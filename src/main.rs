use std::sync::Arc;
use std::{env, fmt, fs};

use log::{error, info};
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
        info!("Peer address: {}", addr);

        match self {
            StreamAcceptor::Plain => {
                let ws_stream = tokio_tungstenite::accept_async(stream)
                    .await
                    .in_context("Error during the websocket handshake occurred")?;

                info!("New WebSocket connection: {}", addr);
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

                info!("New WebSocket connection: {}", addr);
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

#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::try_init().in_context("Failed to init logger")?;

    let addr = env::var("GOE_WEBSOCKET_ADDR").unwrap_or_else(|_| "127.0.0.1:5500".to_string());

    let acceptor = if let Ok(cert) = env::var("GOE_CERT_PKCS12") {
        create_tls_acceptor(cert)?
    } else {
        StreamAcceptor::Plain
    };

    // Create the event loop and TCP listener we'll accept connections on.
    let listener = TcpListener::bind(addr.clone())
        .await
        .in_context("Failed to bind")?;
    info!("Listening on {}://{}", acceptor.scheme(), &addr);

    let game_server = GameServer::default();
    let game_server_addr = game_server.start();

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(run_player(
            stream,
            acceptor.clone(),
            game_server_addr.clone(),
        ));
    }

    Ok(())
}

fn create_tls_acceptor(cert_file: String) -> Result<StreamAcceptor, String> {
    let cert_content = fs::read(cert_file).in_context("Failed to read certificate")?;
    let identity = Identity::from_pkcs12(&cert_content, "").in_context("Invalid certificate")?;
    let acceptor: TlsAcceptor = tokio_native_tls::native_tls::TlsAcceptor::new(identity)
        .in_context("Failed to set certificate")?
        .into();
    Ok(StreamAcceptor::Tls(Arc::new(acceptor)))
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
