use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use anyhow::Context;
use axum::{Router, ServiceExt};
use axum::extract::ConnectInfo;
use axum::handler::Handler;
use axum::routing::get;
use hyper::service::HttpService;
use tower::{Service};
use hyper_util::rt::TokioIo;
use quinn::crypto::rustls::QuicServerConfig;
use quinn::Endpoint;
use rustls::ServerConfig;
use tokio::net::TcpListener;
use tokio::select;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, info_span, trace, trace_span, Instrument};
use crate::config::TlsConfig;
use crate::{Result, ServerState};
use crate::types::util::DateTime;

pub(crate) struct RustlsConfig {
    tcp_config: Arc<ServerConfig>,
    renew_after: DateTime
}

impl TryFrom<TlsConfig> for crate::server::RustlsConfig {
    type Error = anyhow::Error;

    fn try_from(value: TlsConfig) -> std::result::Result<Self, Self::Error> {
        let priv_key_file_path = value.tls_priv_key
            .context("TLS private key not provided")?;
        let pub_cert_file_path = value.tls_pub_cert
            .context("TLS public certificate not provided")?;

        todo!()
    }
}

impl RustlsConfig {
}


struct HttpsServer {
    server_config: Arc<ServerConfig>,
    https_listeners: Vec<TcpListener>,
}

struct Server {
    http_listeners: Vec<TcpListener>,
    https_server: Option<HttpsServer>,
    cancellation_token: CancellationToken,
    server_state: ServerState,
    conn_tracker: TaskTracker,
}

impl Server {
    pub(super) fn new(server_state: ServerState) -> Self {
        Self {
            http_listeners: vec![],
            https_server: None,
            cancellation_token: CancellationToken::new(),
            server_state,
            conn_tracker: TaskTracker::new(),
        }
    }

    pub(super) fn bind_http(mut self, socket_addr: impl ToSocketAddrs) -> Result<Self> {
        let mut listeners = bind_tcp_listeners(socket_addr)?;
        self.http_listeners.append(&mut listeners);

        Ok(self)
    }

    pub(super) fn bind_https_tcp(mut self, socket_addr: impl ToSocketAddrs) -> Result<Self> {
        let listeners = bind_tcp_listeners(socket_addr)?;
        // for listener in listeners {
        //     self.https_listeners.push(HttpsListener::TCP(listener))
        // }

        Ok(self)
    }

    pub(super) async fn serve(mut self) -> Self {
        let app = Router::new().route(
            "/",
            get(
                |ConnectInfo(remote_addr): ConnectInfo<SocketAddr>| async move {
                    format!("Hello {remote_addr}")
                },
            ),
        );
        let mut http_joinset = JoinSet::new();

        let mut make_service = app.into_make_service_with_connect_info::<SocketAddr>();

        while let Some(listener) = self.http_listeners.pop() {
            let mut service = make_service.clone();
            let shutdown_token = self.cancellation_token.child_token();
            let server_state = self.server_state.clone();
            let bind_address = listener.local_addr()
                .map(|s| s.to_string())
                .unwrap_or_else(|_| "unknown address".to_string());

            let listener_span = trace_span!("HTTP listener", bind_address);

            http_joinset.spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((stream, remote_addr)) => {
                            todo!()
                        }
                        Err(err) => {
                            todo!()
                        }
                    }
                }
                return ();
            }.instrument(listener_span));
        }

        if let Some(https_server) = self.https_server {
            let mut https_joinset = JoinSet::new();
            let cert_refresh_token = self.cancellation_token.child_token();

            for listener in https_server.https_listeners {
                let conn_tracker = self.conn_tracker.clone();
                let mut service = make_service.clone();
                let local_addr = listener.local_addr()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| "unknown address".to_string());
                let listener_span = trace_span!("HTTP listener", local_addr);

                https_joinset.spawn(async move {
                    loop {
                        // TODO: Remove unwrap
                        let (tcp_stream, remote_addr) = listener.accept().await.unwrap();

                        info_span!("HTTPS", "remote"=remote_addr.to_string());
                        conn_tracker.spawn(async move {

                        });
                    }
                    return ()
                }.instrument(listener_span));
            }
        }


        todo!()
    }

    pub(super) fn bind_https_quic(mut self, socket_addrs: impl ToSocketAddrs) -> Result<Self> {
        todo!()
        // let server_config = self.server_config.context("br")?;
        // for socket_addr in socket_addrs.to_socket_addrs()? {
        //     let quic_sc = QuicServerConfig::try_from(server_config.clone())?;
        //     let quic_sc= quinn::ServerConfig::with_crypto(quic_sc);
        //     let endpoint = Endpoint::server(quic_sc, socket_addr)?;
        // }
        //
        // Ok(self)
    }
}

fn bind_tcp_listeners(socket_addrs: impl ToSocketAddrs) -> Result<Vec<TcpListener>> {
    let mut listeners = vec![];
    for socket_addr in socket_addrs.to_socket_addrs()? {
        let std_listener = std::net::TcpListener::bind(socket_addr)?;
        std_listener.set_nonblocking(true)?;
        let listener = TcpListener::from_std(std_listener)
            .context("unable to bind socket")?;

        listeners.push(listener);
    }

    Ok(listeners)
}

async fn handle_tcp_stream() {

}