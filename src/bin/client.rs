use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use anyhow::Result;
use quinn::{Endpoint};

#[path = "../cert_utils.rs"]
mod cert_utils;

const SERVER_NAME: &str = "localhost";
const LOCALHOST_V4: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
const CLIENT_ADDR: SocketAddr = SocketAddr::new(LOCALHOST_V4, 4443);
const SERVER_ADDR: SocketAddr = SocketAddr::new(LOCALHOST_V4, 5000);

#[tokio::main]
async fn main() -> Result<()> {
    let endpoint = Endpoint::client(CLIENT_ADDR)?;

    let connection = endpoint.connect(SERVER_ADDR, SERVER_NAME)?.await?;

    Ok(())
}