use std::fmt::{Display, Formatter};
use std::io::Cursor;
use std::net::SocketAddr;

use anyhow::{anyhow, Context};
use bytes::{Buf, BytesMut};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpStream, ToSocketAddrs};
use tracing::{debug, info};

pub use frame::*;

use crate::types::HeosErrorCode;
use crate::HeosResult;

// mod discover;
mod frame;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandResponse {
    pub command_name: String,
    pub message: String,
    pub payload: Value, // can be Null
    pub options: Value, // can be Null
}

impl Display for CommandResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = serde_json::to_string_pretty(&self).unwrap();
        write!(f, "{}", str)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub command_name: String,
    pub eid: HeosErrorCode,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct EventResponse {
    pub event_name: String,
    pub message: String,
}

// copied pasted from https://docs.rs/crate/mini-redis/0.4.1/source/src/connection.rs
#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,

    // The buffer for reading frames.
    buffer: BytesMut,

    peer_addr: SocketAddr,
}

#[allow(dead_code)]
impl Connection {
    // pub async fn find() -> crate::HeosResult<Connection> {
    //     let ips = discover::ssdp_discover_heos_devices()?;
    //
    //     for ip in ips {
    //         match TcpStream::connect((ip.clone(), 1255)).await {
    //             Ok(socket) => return Ok(Connection::new(socket)),
    //             Err(err) => warn!("Failed to connect to {}. cause: {:?}", &ip, err),
    //         };
    //     }
    //     Err(HeosError::NetworkError {
    //         message: "No devices found.".to_owned(),
    //     })
    // }

    pub async fn connect<T: ToSocketAddrs>(s: T) -> HeosResult<Connection> {
        let stream = TcpStream::connect(s)
            .await
            .context("Could not connect to device")?;
        println!("connected to device :{:?}", &stream);
        let peer_addr = stream.peer_addr().context("Failed to ask remote address")?;
        Ok(Connection {
            stream: BufWriter::new(stream),
            // Default to a 4KB read buffer.
            buffer: BytesMut::with_capacity(16 * 1024),
            peer_addr,
        })
    }

    pub fn ip_addr(&self) -> &SocketAddr {
        &self.peer_addr
    }

    pub async fn try_clone(&mut self) -> HeosResult<Self> {
        let addr = self
            .stream
            .get_ref()
            .peer_addr()
            .context("Failed to clone connection to device ")?;
        let stream = TcpStream::connect(addr)
            .await
            .context("Failed to connect to device ")?;
        Ok(Connection {
            stream: BufWriter::new(stream),
            // Default to a 4KB read buffer.
            buffer: BytesMut::with_capacity(16 * 1024),
            peer_addr: self.peer_addr.clone(),
        })
    }

    // TODO check if this is just fancy pancy stuff?! (Does not belong here!)
    // pub fn into_event_stream(mut self) -> impl Stream<Item = HeosResult<HeosEvent>> {
    //     try_stream! {
    //         let _ = self.write_frame("system/register_for_change_events?enable=on")
    //             .await?;
    //         let _response = self.read_command_response().await?;
    //         println!("Listening for events....");
    //         loop {
    //             let event : HeosEvent = self
    //                 .read_event()
    //                 .await
    //                 .and_then(|e| e.try_into())?;
    //             yield event;
    //         }
    //     }
    // }

    pub async fn execute_command<D: Display>(
        &mut self,
        command: D,
    ) -> crate::HeosResult<CommandResponse> {
        let command = command.to_string();
        let payload = format!("heos://{}\r\n", &command);
        info!("Sending command: {}", &command);
        let _ = self
            .stream
            .write_all(payload.as_bytes())
            .await
            .context(format!("Failed to send command '{}'  to device", &payload))?;
        let _ = self
            .stream
            .flush()
            .await
            .context(format!("Failed to send command '{}'  to device", &payload))?;
        self.read_command_response().await
    }

    pub async fn read_event(&mut self) -> crate::HeosResult<EventResponse> {
        loop {
            let response = self.read_frame().await?;
            match response {
                Some(Frame::Event(event)) => return Ok(event),
                _ => { // nop
                }
            }
        }
    }
    pub async fn read_command_response(&mut self) -> crate::HeosResult<CommandResponse> {
        loop {
            let response = self.read_frame().await?;
            match response {
                // this has to be upfront!?
                Some(Frame::UnderProcess(command)) => {
                    debug!(">> waiting for {} to finish.", &command);
                }
                Some(Frame::Response(command)) => return Ok(command),
                Some(Frame::Error(error)) => return Err(error),
                _ => { // nop
                }
            }
        }
    }

    /// Read a single `Frame` value from the underlying stream.
    ///
    /// The function waits until it has retrieved enough data to parse a frame.
    /// Any data remaining in the read buffer after the frame has been parsed is
    /// kept there for the next call to `read_frame`.
    ///
    /// # Returns
    ///
    /// On success, the received frame is returned. If the `TcpStream`
    /// is closed in a way that doesn't break a frame in half, it returns
    /// `None`. Otherwise, an error is returned.
    pub async fn read_frame(&mut self) -> crate::HeosResult<Option<Frame>> {
        loop {
            // Attempt to parse a frame from the buffered data. If enough data
            // has been buffered, the frame is returned.
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            // There is not enough buffered data to read a frame. Attempt to
            // read more data from the socket.
            //
            // On success, the number of bytes is returned. `0` indicates "end
            // of stream".
            if 0 == self
                .stream
                .read_buf(&mut self.buffer)
                .await
                .context("Failed to read buffer")?
            {
                // The remote closed the components.connection. For this to be a clean
                // shutdown, there should be no data in the read buffer. If
                // there is, this means that the peer closed the socket while
                // sending a frame.
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(
                        anyhow!("Failed to read from Heos. Connection reset by peer").into(),
                    );
                }
            }
        }
    }
    fn parse_frame(&mut self) -> crate::HeosResult<Option<Frame>> {
        // Cursor is used to track the "current" location in the
        // buffer. Cursor also implements `Buf` from the `bytes` crate
        // which provides a number of helpful utilities for working
        // with bytes.
        let mut buf = Cursor::new(&self.buffer[..]);

        // The first step is to check if enough data has been buffered to parse
        // a single frame. This step is usually much faster than doing a full
        // parse of the frame, and allows us to skip allocating data structures
        // to hold the frame data unless we know the full frame has been
        // received.
        match Frame::check(&mut buf) {
            Ok(_) => {
                // The `check` function will have advanced the cursor until the
                // end of the frame. Since the cursor had position set to zero
                // before `Frame::check` was called, we obtain the length of the
                // frame by checking the cursor position.
                let len = buf.position() as usize;

                // Reset the position to zero before passing the cursor to
                // `Frame::parse`.
                buf.set_position(0);

                // Parse the frame from the buffer. This allocates the necessary
                // structures to represent the frame and returns the frame
                // value.
                //
                // If the encoded frame representation is invalid, an error is
                // returned. This should terminate the **current** components.connection
                // but should not impact any other connected api.
                let frame = Frame::parse(&mut buf)?;

                // Discard the parsed data from the read buffer.
                //
                // When `advance` is called on the read buffer, all of the data
                // up to `len` is discarded. The details of how this works is
                // left to `BytesMut`. This is often done by moving an internal
                // cursor, but it may be done by reallocating and copying data.
                self.buffer.advance(len);

                // Return the parsed frame to the caller.
                Ok(Some(frame))
            }
            // There is not enough data present in the read buffer to parse a
            // single frame. We must wait for more data to be received from the
            // socket. Reading from the socket will be done in the statement
            // after this `match`.
            //
            // We do not want to return `Err` from here as this "error" is an
            // expected runtime condition.
            #[allow(non_snake_case)]
            Err(_) => Ok(None),
            // An error was encountered while parsing the frame. The components.connection
            // is now in an invalid state. Returning `Err` from here will result
            // in the components.connection being closed.
            // Err(e) => Err(e.into()),
        }
    }
}
