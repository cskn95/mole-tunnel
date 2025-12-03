use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use anyhow::Result;
use quinn::{ServerConfig, Endpoint};

#[path = "../cert_utils.rs"]
mod cert_utils;
const LOCALHOST_V4: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
const SERVER_ADDR: SocketAddr = SocketAddr::new(LOCALHOST_V4, 5000);

#[tokio::main]
async fn main() -> Result<()> {
    let (cert, key) = cert_utils::generate_cert()?;
    let server_config = ServerConfig::with_single_cert(vec![cert], key)?;
    let endpoint = Endpoint::server(server_config, SERVER_ADDR)?;

    while let Some(conn) = endpoint.accept().await {
        let connection = conn.await?;

    }

    Ok(())
}