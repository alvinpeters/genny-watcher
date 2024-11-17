use crate::config::TlsConfig;
use crate::types::util::DateTime;
use crate::{Result, ServerState};
use anyhow::Context;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum::extract::ConnectInfo;
use axum::handler::Handler;
use axum::http::Request;
use axum::middleware::AddExtension;
use axum::routing::get;
use axum::{Router, ServiceExt};
use hyper::body::Incoming;
use hyper::service::HttpService;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn;
use hyper_util::server::conn::auto::UpgradeableConnection;
use hyper_util::server::graceful::GracefulShutdown;
use quinn::crypto::rustls::QuicServerConfig;
use quinn::Endpoint;
use rustls::ServerConfig;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::select;
use tokio::task::JoinSet;
use tokio_rustls::TlsAcceptor;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, info_span, trace, trace_span, Instrument};

pub(crate) struct RustlsConfig {
    tcp_config: Arc<ServerConfig>,
    renew_after: DateTime,
}

impl TryFrom<TlsConfig> for crate::server::RustlsConfig {
    type Error = anyhow::Error;

    fn try_from(value: TlsConfig) -> std::result::Result<Self, Self::Error> {
        let priv_key_file_path = value.tls_priv_key.context("TLS private key not provided")?;
        let pub_cert_file_path = value
            .tls_pub_cert
            .context("TLS public certificate not provided")?;

        todo!()
    }
}

impl RustlsConfig {}

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

        let conn_graceful_shutdown = Arc::new(GracefulShutdown::new());
        let mut http_joinset = JoinSet::new();

        let mut make_service = app.into_make_service_with_connect_info::<SocketAddr>();

        while let Some(listener) = self.http_listeners.pop() {
            let mut service = make_service.clone();
            let shutdown_token = self.cancellation_token.child_token();
            let server_state = self.server_state.clone();
            let bind_address = listener
                .local_addr()
                .map(|s| s.to_string())
                .unwrap_or_else(|_| "unknown address".to_string());

            let listener_span = trace_span!("HTTP listener", bind_address);

            http_joinset.spawn(
                async move {
                    loop {
                        match listener.accept().await {
                            Ok((stream, remote_addr)) => {}
                            Err(err) => {
                                todo!()
                            }
                        }
                    }
                    return ();
                }
                .instrument(listener_span),
            );
        }

        let https_joinset_opt = if let Some(https_server) = self.https_server {
            let mut https_joinset = JoinSet::new();
            let cert_refresh_token = self.cancellation_token.child_token();
            let tls_acceptor = TlsAcceptor::from(https_server.server_config);

            for listener in https_server.https_listeners {
                let conn_tracker = self.conn_tracker.clone();
                let mut service = make_service.clone();
                let local_addr = listener
                    .local_addr()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| "unknown address".to_string());
                let listener_span = trace_span!("HTTP listener", local_addr);
                let tls_acceptor = tls_acceptor.clone();

                https_joinset.spawn(async move {
                    loop {
                        // TODO: Remove unwrap
                        let (tcp_stream, remote_addr) = listener.accept().await.unwrap();
                        let conn_span = info_span!("HTTPS", "remote"=remote_addr.to_string());
                        let tls_acceptor = tls_acceptor.clone();
                        // Infallible
                        use tower::Service;
                        let tower_service = service.call(remote_addr).await.unwrap();

                        conn_tracker.spawn(async move {
                            let tls_stream = match tls_acceptor.accept(tcp_stream).await {
                                Ok(stream) => stream,
                                Err(err) => {
                                    debug!("failed to perform TLS handshake, ending the connection");
                                    trace!("error thrown: {}", err);
                                    return;
                                }
                            };

                            handle_tcp_stream(tls_stream, tower_service).await;
                        }.instrument(conn_span));
                    }
                    return ()
                }.instrument(listener_span));
            }

            Some(https_joinset)
        } else {
            None
        };

        http_joinset.abort_all();
        if let Some(https_joinset) = https_joinset_opt {}
        match Arc::into_inner(conn_graceful_shutdown) {
            None => todo!(),
            Some(_) => todo!(),
        }
    }
}

fn bind_tcp_listeners(socket_addrs: impl ToSocketAddrs) -> Result<Vec<TcpListener>> {
    let mut listeners = vec![];

    for socket_addr in socket_addrs.to_socket_addrs()? {
        let std_listener = std::net::TcpListener::bind(socket_addr)?;
        std_listener.set_nonblocking(true)?;
        let listener = TcpListener::from_std(std_listener).context("unable to bind socket")?;

        listeners.push(listener);
    }

    Ok(listeners)
}

async fn handle_tcp_stream<Stream: AsyncWrite + AsyncRead + Send + Unpin + 'static>(
    stream: Stream,
    service: AddExtension<Router, ConnectInfo<SocketAddr>>,
) {
    use tower::util::Oneshot;
    use tower::ServiceExt;

    let tokio_stream = TokioIo::new(stream);

    // Hyper also has its own `Service` trait and doesn't use tower. We can use
    // `hyper::service::service_fn` to create a hyper `Service` that calls our app through
    // `tower::Service::call`.
    let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
        // We have to clone `tower_service` because hyper's `Service` uses `&self` whereas
        // tower's `Service` requires `&mut self`.
        //
        // We don't need to call `poll_ready` since `Router` is always ready.
        service.clone().oneshot(request)
    });

    match conn::auto::Builder::new(TokioExecutor::new())
        .serve_connection_with_upgrades(tokio_stream, hyper_service)
        .await
    {
        Ok(()) => trace!("HTTP connection gracefully ended"),
        Err(err) => {}
    }
}
