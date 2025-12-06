use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::{Context, Result};
use bytes::Bytes;
use pcap::{Capture, Device};
use quinn::{Connection, Endpoint, RecvStream, ServerConfig};
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use tracing::{error, info};

pub fn generate_cert() -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>)> {
    let cert = generate_simple_self_signed(vec!["localhost".to_string()])
        .context( "Failed to generate self-signed certificate")?;
    let cert_der = CertificateDer::from(cert.cert);
    let key_raw = PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der());
    let key:PrivateKeyDer = PrivateKeyDer::Pkcs8(key_raw);

    Ok((cert_der, key))
}

pub fn make_client_endpoint(bind_addr: SocketAddr, server_cert: &[&[u8]]) -> Result<Endpoint> {
    let client_config = configure_client(server_cert)?;
    let mut endpoint = Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
}

pub fn configure_client(server_cert: &[&[u8]]) -> Result<quinn::ClientConfig> {
    let mut root_store = rustls::RootCertStore::empty();
    for cert in server_cert {
        root_store.add(CertificateDer::from(*cert))?;
    };

    Ok(quinn::ClientConfig::with_root_certificates(Arc::new(root_store))?)
}

pub fn make_server_endpoint(bind_addr: SocketAddr) -> Result<(Endpoint, CertificateDer<'static>)> {
    let (server_config, server_cert) = configure_server()?;
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok((endpoint, server_cert))
}

pub fn configure_server() -> Result<(ServerConfig, CertificateDer<'static>)> {
    let (cert, key) = generate_cert()?;

    let server_config = ServerConfig::with_single_cert(vec![cert.clone()], key)?;
    Ok((server_config, cert))
}

pub async fn handle_recv(mut recv_stream: RecvStream) -> Result<String> {
    let buf = recv_stream
        .read_to_end(usize::MAX)
        .await
        .context("Failed to read from the stream")?;

    let package = String::from_utf8(buf)
        .context("Failed to parse the package as UTF-8")?;

    Ok(package)
}

pub async fn handle_send(mut send_stream: quinn::SendStream, package: &str) -> Result<()> {
    send_stream.write_all(package.as_bytes())
        .await
        .context( "Failed to send the package")?;
    Ok(())
}

pub fn listener(dev_name: Arc<String>, filter: String, tx: tokio::sync::mpsc::Sender<Bytes>, label: String) {
    tokio::task::spawn_blocking(move || {
        let dev = Device::from(dev_name.as_str());
        let mut cap = Capture::from_device(dev)
            .unwrap()
            .promisc(true)
            .snaplen(65535)
            .timeout(1000)
            .open()
            .inspect_err(|e| error!("[{}] failed to open the network interface: {}", label, e))
            .unwrap();

        cap
            .filter(filter.as_str(), true)
            .inspect_err(|e| error!("[{}] filter failed: {}", label, e))
            .unwrap();
        info!("[{}] sniffing started", label);

        loop {
            match cap.next_packet() {
                Ok(_packet) => {
                    let data = Bytes::copy_from_slice(_packet.data);

                    if let Err(_) = tx.blocking_send(data) {
                        error!("[{}] failed to send a packet via mpsc channel", label);
                    }
                },
                Err(pcap::Error::TimeoutExpired) => continue,
                Err(e) => error!("[{}] ailed to capture a packet: {}", label, e),
            }
        }
    });
}

pub async fn connector(conn:Arc<Connection>, mut rx: tokio::sync::mpsc::Receiver<Bytes>, label: String) {
    tokio::spawn(async move {
        let mut send_stream = conn.open_uni()
            .await
            .inspect_err(|e| error!("[{}] failed to open two-way stream: {}", label, e))
            .unwrap();
        info!("[{}] stream opened", label);

        while let Some(data) = rx.recv().await {
            let len = data.len();
            let len_bytes = (len as u32).to_be_bytes();

            if let Err(_) = send_stream.write_all(&len_bytes).await {
                error!("[{}] failed to fetch the length of the package", label);
                break;
            }

            if let Err(_) = send_stream.write_all(&data).await {
                error!("[{}] failed to send the package", label);
                break;
            }
            info!("[{}] sending a package", label);
        }
    });
}