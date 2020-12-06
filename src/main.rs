use std::{env, fmt, io, io::Error};

use futures_util::TryFutureExt;
use log::{error, info};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Error as WsError;

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

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = env_logger::try_init();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:5500".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let mut listener = try_socket.expect("Failed to bind");
    info!("Listening on: ws://{}", addr);

    let game_server = GameServer::new();
    let game_server_addr = game_server.start();

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(
            client_main(stream, game_server_addr.clone()).map_err(|err| {
                error!("{}", err);
                err
            }),
        );
    }

    Ok(())
}

async fn client_main(stream: TcpStream, game_server: GameServerAddr) -> Result<(), String> {
    let addr = stream
        .peer_addr()
        .in_context("Connected streams should have a peer address")?;
    info!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .in_context("Error during the websocket handshake occurred")?;

    info!("New WebSocket connection: {}", addr);

    let conn = RemoteConnection::new(ws_stream);
    Player::new(conn, game_server).run().await;

    Ok(())
}
