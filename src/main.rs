use std::sync::Arc;
use std::{env, fmt, fs};

use log::{debug, error, info};
use tokio::net::{TcpListener, TcpStream};
use tokio_native_tls::native_tls::Identity;
use tokio_native_tls::TlsAcceptor;

use game_of_estimates::game_server::{GameServer, GameServerAddr};
use game_of_estimates::player::Player;
use game_of_estimates::remote::RemoteConnection;
use uactor::blocking::Actor;

trait ErrorContextExt<T> {
    fn in_context(self, context: &str) -> Result<T, String>;
}

impl<T, E: fmt::Debug> ErrorContextExt<T> for Result<T, E> {
    fn in_context(self, context: &str) -> Result<T, String> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(format!("{}: {:?}", context, err)),
        }
    }
}

#[derive(Clone)]
enum StreamAcceptor {
    Plain,
    Tls(Arc<TlsAcceptor>),
}

impl StreamAcceptor {
    pub async fn accept(&self, stream: TcpStream) -> Result<RemoteConnection, String> {
        let addr = stream
            .peer_addr()
            .in_context("Connected streams should have a peer address")?;
        debug!("Peer address: {}", addr);

        match self {
            StreamAcceptor::Plain => {
                let ws_stream = tokio_tungstenite::accept_async(stream)
                    .await
                    .in_context("Error during the websocket handshake occurred")?;

                debug!("New WebSocket connection: {}", addr);
                Ok(RemoteConnection::new(ws_stream))
            }

            StreamAcceptor::Tls(tls_acceptor) => {
                let tls_stream = tls_acceptor
                    .accept(stream)
                    .await
                    .in_context("Failed to accept TLS connection")?;

                let ws_stream = tokio_tungstenite::accept_async(tls_stream)
                    .await
                    .in_context("Error during the websocket handshake occurred")?;

                debug!("New WebSocket connection: {}", addr);
                Ok(RemoteConnection::new_with_tls(ws_stream))
            }
        }
    }

    pub fn scheme(&self) -> &'static str {
        match self {
            StreamAcceptor::Plain => "ws",
            StreamAcceptor::Tls(_) => "wss",
        }
    }
}

fn create_tls_acceptor(cert_file: String) -> Result<StreamAcceptor, String> {
    let cert_content = fs::read(cert_file).in_context("Failed to read certificate")?;
    let identity = Identity::from_pkcs12(&cert_content, "").in_context("Invalid certificate")?;
    let acceptor: TlsAcceptor = tokio_native_tls::native_tls::TlsAcceptor::new(identity)
        .in_context("Failed to set certificate")?
        .into();
    Ok(StreamAcceptor::Tls(Arc::new(acceptor)))
}

#[chassis::integration]
mod integration {
    use super::*;

    struct Bindings;

    impl Bindings {
        #[singleton]
        pub fn provide_game_server() -> GameServerAddr {
            GameServer::default().start()
        }

        #[singleton]
        pub fn provide_tcp_acceptor() -> StreamAcceptor {
            if let Ok(cert) = env::var("GOE_CERT_PKCS12") {
                create_tls_acceptor(cert).unwrap()
            } else {
                StreamAcceptor::Plain
            }
        }

        #[singleton]
        pub fn provide_listen_addr() -> ListenAddr {
            ListenAddr(
                env::var("GOE_WEBSOCKET_ADDR").unwrap_or_else(|_| "127.0.0.1:5500".to_string()),
            )
        }

        pub fn provide_main(
            addr: ListenAddr,
            acceptor: StreamAcceptor,
            game_server: GameServerAddr,
        ) -> Main {
            Main {
                addr,
                acceptor,
                game_server,
            }
        }
    }

    pub trait Integrator {
        fn provide_main(&self) -> Main;
    }
}

#[derive(Clone)]
struct ListenAddr(String);

pub struct Main {
    addr: ListenAddr,
    acceptor: StreamAcceptor,
    game_server: GameServerAddr,
}

impl Main {
    pub async fn run(&self) -> Result<(), String> {
        // Create the event loop and TCP listener we'll accept connections on.
        let listener = TcpListener::bind(self.addr.0.clone())
            .await
            .in_context("Failed to bind")?;
        info!("Listening on {}://{}", self.acceptor.scheme(), &self.addr.0);

        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(Self::run_player(
                stream,
                self.acceptor.clone(),
                self.game_server.clone(),
            ));
        }

        Ok(())
    }

    async fn run_player(stream: TcpStream, acceptor: StreamAcceptor, game_server: GameServerAddr) {
        let conn = match acceptor.accept(stream).await {
            Ok(conn) => conn,
            Err(err) => {
                error!("Failed to accept connection: {}", err);
                return;
            }
        };

        Player::new(conn, game_server).run().await
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    use crate::integration::Integrator;

    env_logger::try_init().in_context("Failed to init logger")?;

    let integrator = integration::IntegratorImpl::new();
    integrator.provide_main().run().await
}
