use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use anyhow::Result;
use bytes::Bytes;
use pcap::{Capture, Device};
use tracing::{info, error};

use the_tunnel::cert_utils;
use the_tunnel::cert_utils::{connector, listener};

const LOCALHOST_V4: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
const CLIENT_ADDR: SocketAddr = SocketAddr::new(LOCALHOST_V4, 3131);
const SERVER_ADDR: SocketAddr = SocketAddr::new(LOCALHOST_V4, 5000);


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .init();
    info!("Client initializing");

    let (tx_http, rx_http) = tokio::sync::mpsc::channel::<Bytes>(4096);
    let (tx_dns, rx_dns) = tokio::sync::mpsc::channel::<Bytes>(4096);

    let device = Device::lookup()
        .expect("Failed to find the network interface")
        .expect("Failed to open the network interface");
    let device_name = Arc::new(device.name);

    let cert = std::fs::read("cert.der")
        .inspect_err(|e| error!("Failed to read the certificate: {}", e))?;
    info!("Certificate successfully read");

    let endpoint = cert_utils::make_client_endpoint(CLIENT_ADDR, &[&cert])
        .inspect_err(|e| error!("Failed to initialize the client endpoint: {}", e))?;
    info!("Client endpoint initialized");

    let connection = Arc::new(
        endpoint.connect(SERVER_ADDR, "localhost")
            .inspect_err(|e| error!("Failed to connect to the server: {}", e))?
            .await?
    );
    info!("Connected to the server");

    let dev_name_http = device_name.clone();
    let dev_name_dns = device_name.clone();

    let conn_http = connection.clone();
    let conn_dns = connection.clone();

    listener(dev_name_http, "tcp port 80 or tcp port 443 or udp port 443".to_string(), tx_http, "HTTP".to_string());
    listener(dev_name_dns, "tcp port 53 or udp port 53".to_string(), tx_dns, "DNS".to_string());

    connector(conn_http, rx_http, "HTTP".to_string()).await;
    connector(conn_dns, rx_dns, "DNS".to_string()).await;

    tokio::signal::ctrl_c().await?;

    info!("Closing the connection");
    connection.close(0u32.into(), b"bye");
    
    Ok(())
}