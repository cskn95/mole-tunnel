use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::{Context, Result};
use quinn::{Endpoint, ServerConfig};
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ClientConfig, ProtocolVersion};

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

pub fn configure_server() -> Result<(quinn::ServerConfig, CertificateDer<'static>)> {
    let (cert, key) = generate_cert()?;

    let mut server_config = ServerConfig::with_single_cert(vec![cert.clone()], key)?;
    Ok((server_config, cert))
}