use anyhow::{anyhow, Context};
use clap::{Args, Parser};
use rkyv::{Archive, Deserialize, Serialize};
use rustls::internal::msgs::handshake::CertificateChain;
use rustls::ServerConfig;
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{pem, CertificateDer, PrivateKeyDer};
use std::convert::Infallible;
use std::fs::File;
use std::net::SocketAddr;
use std::path::PathBuf;

use crate::Result;

/// Temporary. Will remove once ACME is ready.
#[derive(Args, Archive, Serialize, Deserialize)]
pub(super) struct TlsConfig {
    #[arg(long, requires = "tls_pub_cert", value_parser = get_priv_key_from_file)]
    pub(crate) tls_priv_key: Option<Vec<u8>>,
    #[arg(long, requires = "tls_priv_key", value_parser = get_pub_cert_chain_from_file)]
    pub(crate) tls_pub_cert: Option<Vec<Vec<u8>>>,
}

impl TlsConfig {
    pub(super) fn is_some(&self) -> bool {
        self.tls_priv_key.is_some() && self.tls_pub_cert.is_some()
    }
}

fn get_priv_key_from_file(path: &str) -> Result<Vec<u8>> {
    // For validation
    let der = PrivateKeyDer::from_pem_file(path)
        .map_err(|_| anyhow!("failed to get TLS private key from file: {}", path))?;

    Ok(der.secret_der().to_vec())
}

fn get_pub_cert_chain_from_file(path: &str) -> Result<Vec<Vec<u8>>> {
    // For validation
    let certs = CertificateDer::pem_file_iter(path).map_err(|_| {
        anyhow!(
            "failed to get TLS public certificate chain from file: {}",
            path
        )
    })?;
    let mut cert_der_vec = vec![];

    for cert_res in certs {
        let cert = cert_res
            .map_err(|_| anyhow!("failed to parse a public certificate from file: {}", path))?;
        cert_der_vec.push(cert.to_vec());
    }

    Ok(cert_der_vec)
}

#[derive(Args)]
pub(super) struct BindConfig {
    #[arg(long, requires = "tls_pub_cert")]
    pub(super) https_socket: Vec<SocketAddr>,
    #[arg(long)]
    pub(super) http_socket: Vec<SocketAddr>,
}

impl TryFrom<TlsConfig> for ServerConfig {
    type Error = anyhow::Error;

    fn try_from(tls_config: TlsConfig) -> Result<Self, Self::Error> {
        let Some(priv_key_der_bytes) = tls_config.tls_priv_key else {
            todo!()
        };
        let Some(pub_cert_der_bytes) = tls_config.tls_pub_cert else {
            todo!()
        };

        let priv_key = PrivateKeyDer::try_from(priv_key_der_bytes).map_err(|err| anyhow!(err))?;

        let mut cert_chain = vec![];
        for cert_bytes in pub_cert_der_bytes {
            // Infallible
            let cert = CertificateDer::try_from(cert_bytes)?;
            cert_chain.push(cert);
        }

        let server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, priv_key)?;

        Ok(server_config)
    }
}

#[derive(Parser)]
pub(super) struct CliConfig {
    #[command(flatten)]
    pub(super) bind_config: BindConfig,
    #[command(flatten)]
    pub(super) tls_config: TlsConfig,
}
