use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use anyhow::Result;
use quinn::Connection;
use tokio::net::TcpListener;
use tracing::{info, error};

use the_tunnel::cert_utils;

const LOCALHOST_V4: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
const CLIENT_ADDR: SocketAddr = SocketAddr::new(LOCALHOST_V4, 3131);
const SERVER_ADDR: SocketAddr = SocketAddr::new(LOCALHOST_V4, 5000);
const PACKAGE: &str = "Hello, server!";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .init();
    info!("Client initializing");

    let cert = std::fs::read("cert.der")
        .inspect_err(|e| error!("Failed to read the certificate: {}", e))?;
    info!("Certificate successfully read");

    let endpoint = cert_utils::make_client_endpoint(CLIENT_ADDR, &[&cert])
        .inspect_err(|e| error!("Failed to initialize the client endpoint: {}", e))?;
    info!("Client endpoint initialized");

    let connection = endpoint.connect(SERVER_ADDR, "localhost")
        .inspect_err(|e| error!("Failed to connect to the server: {}", e))?
        .await?;
    info!("Connected to the server");

    let mut send_stream = connection.open_uni()
        .await
        .inspect_err(|e| error!("Failed to open two-way stream: {}", e))?;
    info!("Streams opened");

    send_stream.write_all(PACKAGE.as_bytes())
        .await
        .inspect_err(|e| error!("Failed to send the package: {}", e))?;
    info!("Sending the package");

    send_stream.finish()?;
    info!("Package sent");
    
    endpoint.wait_idle().await;
    connection.close(0u32.into(), b"done");
    info!("Client shutting down gracefully");
    
    Ok(())
}