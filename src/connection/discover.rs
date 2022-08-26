use std::net::IpAddr;

use ssdp::header::{HeaderMut, Man, MX, ST};
use ssdp::message::{Config, Multicast, SearchRequest, SearchResponse};
use ssdp::{FieldMap, SSDPError, SSDPReceiver};
use tokio::runtime;
use tokio::sync::mpsc;
use tracing::debug;

use crate::HeosError;

// todo this would make the entire app async. Don't know if I need it.
pub fn device_channel() -> mpsc::Receiver<IpAddr> {
    let (sender, recsiver) = mpsc::channel(12);

    // Build the runtime for the new thread.
    //
    // The runtime is created before spawning the thread
    // to more cleanly forward errors if the `unwrap()`
    // panics.
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    std::thread::spawn(move || {
        let query_result = ssdp_discover_heos_devices();

        match query_result {
            Ok(devices) => {
                rt.block_on(async move {
                    for ip_addr in devices {
                        sender.send(ip_addr).await;
                    }
                });
            }
            Err(err) => {
                tracing::warn!("Failed to find upd devices, {:?}", &err);
            }
        }
    });
    recsiver
}

pub fn ssdp_discover_heos_devices() -> Result<Vec<IpAddr>, HeosError> {
    debug!("Searching for heos devices");
    let mut result = vec![];

    let mut request = SearchRequest::new();

    request.set(Man);
    request.set(MX(5));
    request.set(ST::Target(FieldMap::URN(String::from(
        "schemas-denon-com:device:ACT-Denon:1",
    ))));
    let config = Config::default(); //.set_port(9876);
    let results: SSDPReceiver<SearchResponse> = request.multicast_with_config(&config)?;
    for (_msg, src) in results {
        debug!("Found device at {:?}", &src.ip());
        result.push(src.ip());
    }
    return Ok(result);
}

impl From<SSDPError> for HeosError {
    fn from(error: SSDPError) -> Self {
        HeosError::NetworkError {
            message: format!("SSDP failed. {}", error),
        }
    }
}
