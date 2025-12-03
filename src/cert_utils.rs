use anyhow::{Context, Result};
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime};

pub fn generate_cert() -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>)> {
    let cert = generate_simple_self_signed(vec!["localhost".to_string()])
        .context( "Failed to generate self-signed certificate")?;
    let cert_der = CertificateDer::from(cert.cert);
    let key_raw = PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der());
    let key:PrivateKeyDer = PrivateKeyDer::Pkcs8(key_raw);

    Ok((cert_der, key))
}