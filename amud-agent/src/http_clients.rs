//! Shared hyper clients for PVE HTTPS and Docker unix socket (avoids per-poll TLS setup).

use http_body_util::Empty;
use hyper::body::Bytes;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use std::sync::{Arc, OnceLock};

// Trust-all certificate verifier for loopback PVE :8006 only.
#[derive(Debug)]
pub(crate) struct NoVerifier;

impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

type PveHttpsClient = Client<
    hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
    Empty<Bytes>,
>;

static PVE_CLIENT: OnceLock<PveHttpsClient> = OnceLock::new();

pub(crate) fn pve_https_client() -> &'static PveHttpsClient {
    PVE_CLIENT.get_or_init(|| {
        let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .expect("TLS protocol versions")
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoVerifier))
        .with_no_client_auth();

        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_tls_config(tls)
            .https_or_http()
            .enable_http1()
            .build();

        Client::builder(TokioExecutor::new()).build(https)
    })
}

#[cfg(unix)]
type DockerUnixClient = Client<hyperlocal::UnixConnector, Empty<Bytes>>;

#[cfg(unix)]
static DOCKER_CLIENT: OnceLock<DockerUnixClient> = OnceLock::new();

#[cfg(unix)]
pub(crate) fn docker_unix_client() -> &'static DockerUnixClient {
    DOCKER_CLIENT.get_or_init(Client::unix)
}
