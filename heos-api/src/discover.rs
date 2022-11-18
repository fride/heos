use std::time::Duration;
use tokio_stream::StreamExt;
use std::net::IpAddr;
use std::str::FromStr;
use anyhow::{anyhow, Context};
use ssdp_client::{SearchTarget, URN};
use tracing::info;
use crate::error::HeosError;
use crate::HeosResult;
use url::{Url, ParseError};

pub async fn find_heos_devices() -> HeosResult<IpAddr>{
    info!("Searching for heos devices");
    let search_target = SearchTarget::from_str("urn:schemas-denon-com:device:ACT-Denon:1").unwrap();
    let mut responses = ssdp_client::search(&search_target, Duration::from_secs(3), 2)
        .await
        .context("Failed to query for upnp devices")?;

    while let Some(device) = responses.try_next().await
        .context("Failed to query for upnp devices")? {
        // wow, so much parser nonsense!
        match device.search_target() {
            SearchTarget::URN(urn)
                if urn.domain_name() == "schemas-denon-com" && urn.typ() == "ACT-Denon" => {
                info!("Found a heos device");
                let url = Url::parse(device.location())
                    .context("UPNP URL not parseable")?;
                let host = url.host().ok_or(anyhow!("Url without host"))?;
                let ip =  IpAddr::from_str(&host.to_string())
                    .with_context(||"Failed to parse ip address")?;
                return Ok(ip)
            }
            _ => {
                info!("Found something else");
                continue
            }
        };
    }
    Err(HeosError::NoDeviceFound)
}


#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    pub async fn test_stuff() {
        let devices = find_heos_devices().await.unwrap();
    }
}
