use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use anyhow::Result;
use tracing::{info, error};
//use tracing_subscriber::EnvFilter;
use the_tunnel::cert_utils;
const LOCALHOST_V4: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
const SERVER_ADDR: SocketAddr = SocketAddr::new(LOCALHOST_V4, 5000);


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .init();
    info!("Server initializing");

    let (endpoint, cert) = cert_utils::make_server_endpoint(SERVER_ADDR)
        .inspect_err(|e| error!("Failed to initialize server endpoint: {}", e))?;
    info!("Server listening on {}", SERVER_ADDR);

    std::fs::write("cert.der", &cert)
        .inspect_err(|e| error!("Failed to write certificate: {}", e))?;
    info!("Certificate successfully written to cert.der");

    while let Some(conn) = endpoint.accept().await {
        info!("New connection: {}", conn.remote_address());

        tokio::spawn(async move {
            let client = conn
                .await
                .inspect_err(|e| error!("I have no idea about what the fuck is going on but here's the error: {}", e))
                .unwrap();

            let mut recv_stream = client
                .accept_uni()
                .await
                .inspect_err(|e| error!("Connection failed: {}", e))
                .unwrap();
            info!("Connection established");

            loop {
                let mut len_buf = [0u8; 4];
                match recv_stream.read_exact(&mut len_buf).await {
                    Ok(_) => {
                        let msg_len = u32::from_be_bytes(len_buf) as usize;
                        let mut packet_buf = vec![0u8; msg_len];

                        match recv_stream.read_exact(&mut packet_buf).await {
                            Ok(_) => {
                                info!("Received a package: {} bytes", msg_len);
                            }
                            Err(_) => {
                                error!("Failed to receive a package");
                            }
                        }
                    },
                    Err(e) => error!("{}", e)
                }
            }
        });
    }

    Ok(())
}