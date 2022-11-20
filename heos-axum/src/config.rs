use clap::Parser;
use heos_api::HeosResult;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Parser, Debug)]
pub struct Config {
    #[clap(long, env)]
    pub rust_log: Option<String>,

    #[clap(long, env, default_value_t = 8080)]
    pub port: u16,

    #[clap(long, env)]
    pub host: Option<IpAddr>,

    #[clap(long, env)]
    pub heos_device_addr: Option<IpAddr>,

    #[clap(long, env)]
    pub base_url: String,
}

impl Config {
    pub fn get_local_addr(&self) -> SocketAddr {
        let host = self.host.unwrap_or(Ipv4Addr::new(127, 0, 0, 1).into());
        SocketAddr::new(host, self.port)
    }
}
