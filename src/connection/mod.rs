use std::fmt::Display;
use std::io::Cursor;

use async_stream::try_stream;
use async_trait::async_trait;
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_stream::Stream;
use tracing::{debug, warn};

pub use frame::*;

use crate::{HeosError, HeosResult};
use crate::model::event::HeosEvent;

mod discover;
mod frame;

// copied pasted from https://docs.rs/crate/mini-redis/0.4.1/source/src/connection.rs
#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,

    // The buffer for reading frames.
    buffer: BytesMut,
}

impl Connection {
    pub async fn find() -> crate::HeosResult<Connection> {
        let ips = discover::ssdp_discover_heos_devices()?;

        for ip in ips {
            match TcpStream::connect((ip.clone(), 1255)).await {
                Ok(socket) => return Ok(Connection::new(socket)),
                Err(err) => warn!("Failed to connect to {}. cause: {:?}", &ip, err),
            };
        }
        Err(HeosError::NetworkError {
            message: "No devices found.".to_owned(),
        })
    }

    pub async fn connect<T: ToSocketAddrs>(s: T) -> HeosResult<Connection> {
        let stream = TcpStream::connect(s).await?;
        println!("connected woth :{:?}", &stream);
        Ok(Self::new(stream))
    }

    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(socket),
            // Default to a 4KB read buffer.
            buffer: BytesMut::with_capacity(16 * 1024),
        }
    }

    pub async fn try_clone(&mut self) -> crate::HeosResult<Self> {
        let addr = self.stream.get_ref().peer_addr()?;
        let stream = TcpStream::connect(addr).await?;
        Ok(Connection::new(stream))
    }

    pub fn into_event_stream(mut self) -> impl Stream<Item = HeosResult<HeosEvent>> {
        try_stream! {
            let _ = self.write_frame("system/register_for_change_events?enable=on")
                .await?;
            let _response = self.read_command_response().await?;
            println!("Listening for events....");
            loop {
                let event : HeosEvent = self
                    .read_event()
                    .await
                    .and_then(|e| e.try_into())?;
                yield event;
            }
        }
    }

    pub async fn write_frame(&mut self, command: &str) -> crate::HeosResult<()> {
        self.stream
            .write_all(format!("heos://{}\r\n", command).as_bytes())
            .await?;
        self.stream.flush().await?;
        Ok(())
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
                Some(Frame::Response(command)) => return Ok(command),
                Some(Frame::Error(error)) => {
                    return Err(HeosError::InvalidCommand {
                        command: "".to_owned(),
                        message: error,
                    })
                }
                Some(Frame::UnderProcess(command)) => {
                    debug!(">> waiting for {} to finish.", &command);
                }
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
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // The remote closed the components.connection. For this to be a clean
                // shutdown, there should be no data in the read buffer. If
                // there is, this means that the peer closed the socket while
                // sending a frame.
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("components.connection reset by peer".into());
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
                // but should not impact any other connected client.
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
            #[warn(non_snake_case)]
            Err(_Incomplete) => Ok(None),
            // An error was encountered while parsing the frame. The components.connection
            // is now in an invalid state. Returning `Err` from here will result
            // in the components.connection being closed.
            // Err(e) => Err(e.into()),
        }
    }
}

#[async_trait]
pub trait CommandExecutor {
    async fn execute_command<A, B>(&mut self, command: A) -> HeosResult<B>
    where
        A: Display + Send,
        B: TryFrom<CommandResponse, Error = HeosError>;
}

#[async_trait]
impl CommandExecutor for Connection {
    async fn execute_command<A, B>(&mut self, command: A) -> HeosResult<B>
    where
        A: Display + Send,
        B: TryFrom<CommandResponse, Error = HeosError>,
    {
        let command = format!("{}", command);
        tracing::debug!("executing command: {}", &command);
        let _ = self.write_frame(&command).await?;
        let response = self.read_command_response().await?;
        response.try_into()
    }
}
