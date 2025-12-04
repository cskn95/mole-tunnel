use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use anyhow::Result;
use quinn::{ServerConfig, Endpoint};

#[path = "../cert_utils.rs"]
mod cert_utils;
const LOCALHOST_V4: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
const SERVER_ADDR: SocketAddr = SocketAddr::new(LOCALHOST_V4, 5000);

#[tokio::main]
async fn main() -> Result<()> {
    let (endpoint, cert) = cert_utils::make_server_endpoint(SERVER_ADDR)?;

    while let Some(conn) = endpoint.accept().await {


    }

    Ok(())
}