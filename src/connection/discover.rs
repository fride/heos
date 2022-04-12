use std::net::IpAddr;

use ssdp::{FieldMap, SSDPError, SSDPReceiver};
use ssdp::header::{HeaderMut, Man, MX, ST};
use ssdp::message::{Config, Multicast, SearchRequest, SearchResponse};
use tracing::info;

use crate::HeosError;

pub fn ssdp_discover_heos_devices() -> Result<Vec<IpAddr>, HeosError> {
    info!("Searching for heos devices");
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
        info!("Found device at {:?}", &src);
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
