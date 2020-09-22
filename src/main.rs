use std::{env, io::Error};

use futures_util::{SinkExt, StreamExt};
use game_of_estimates::server::{GameServer, Participant, RemoteMessage};
use log::info;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = env_logger::try_init();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let mut listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    let game_server = Arc::new(GameServer::new());

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(client_main(stream, game_server.clone()));
    }

    Ok(())
}

async fn client_main(stream: TcpStream, game_server: Arc<GameServer>) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("New WebSocket connection: {}", addr);

    let (mut write, mut read) = ws_stream.split();
    let (mut tx, mut rx) = mpsc::channel(10);

    let participant = Participant::new(tx);
    game_server.add_participant(participant).await;

    let client_id = RemoteMessage::Hello {
        id: "abcABC".to_string(),
    };
    write
        .send(Message::Text(serde_json::to_string(&client_id).unwrap()))
        .await;

    loop {
        tokio::select! {
            remote_msg = read.next() => {
                if let Some(Ok(Message::Close(_))) = remote_msg {
                    info!("Remote disconnected friendly: {}", addr);
                    break;
                }
                if remote_msg.is_none() {
                    info!("Remote disconnected: {}", addr);
                    break;
                }
                info!("Remote recv: {:?}", remote_msg);
            },
            internal_msg = rx.recv() => {

            }
        }
    }
}
