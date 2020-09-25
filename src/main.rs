use std::{env, io::Error};

use log::info;
use tokio::net::{TcpListener, TcpStream};

use game_of_estimates::actor::{run_actor, Actor};
use game_of_estimates::game_server::{GameServer, GameServerAddr};
use game_of_estimates::player::Player;
use game_of_estimates::remote::RemoteConnection;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = env_logger::try_init();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:5500".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let mut listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    let game_server = GameServer::new();
    let game_server_addr = game_server.addr();
    tokio::spawn(run_actor(game_server));

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(client_main(stream, game_server_addr.clone()));
    }

    Ok(())
}

async fn client_main(stream: TcpStream, game_server: GameServerAddr) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("New WebSocket connection: {}", addr);

    let conn = RemoteConnection::new(ws_stream);
    Player::new(conn, game_server).run().await;
}
