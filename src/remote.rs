use std::convert::TryInto;

use futures_util::{SinkExt, StreamExt};
use quick_error::quick_error;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::time::{Duration, Instant};
use tokio_native_tls::TlsStream;
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use crate::room::{GameState, PlayerState};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum RemoteMessage {
    // upstream
    Vote {
        vote: Option<String>,
    },
    UpdatePlayer {
        voter: bool,
        name: Option<String>,
    },
    ForceOpen,
    Restart,
    SetName {
        name: String,
    },
    JoinRoom {
        room: String,
    },
    CreateRoom {
        deck: String,
    },
    // pseudo
    Ping(Duration),
    Close,

    // downstream
    Welcome {
        player_id: String,
    },
    Rejected,
    Joined {
        room: String,
        state: GameState,
        players: Vec<PlayerState>,
    },
    PlayerJoined {
        player: PlayerState,
    },
    PlayerChanged {
        player: PlayerState,
    },
    PlayerLeft {
        player_id: String,
    },
    GameChanged {
        game_state: GameState,
    },
}

quick_error! {
    #[derive(Debug)]
    pub enum ConnError {
        Ws(err: WsError) {
            display("Web socket err: {}", err)
            from()
        }
        Json(err: serde_json::Error) {
            display("Web socket err: {}", err)
            from()
        }
        UnsupportedMessageFormat(msg: Message) {
            display("Unsupported web socket message: {}", msg)
        }
    }
}

type ConnResult<T> = Result<T, ConnError>;

pub struct RemoteConnection {
    socket: WebSocket,

    last_ping_start: Instant,
    last_ping_id: u16,
}

enum WebSocket {
    Plain(WebSocketStream<TcpStream>),
    Tls(WebSocketStream<TlsStream<TcpStream>>),
}

impl WebSocket {
    pub async fn send(&mut self, msg: Message) -> Result<(), WsError> {
        match self {
            WebSocket::Plain(stream) => stream.send(msg).await,
            WebSocket::Tls(stream) => stream.send(msg).await,
        }
    }

    pub async fn next(&mut self) -> Option<Result<Message, WsError>> {
        match self {
            WebSocket::Plain(stream) => stream.next().await,
            WebSocket::Tls(stream) => stream.next().await,
        }
    }
}

impl RemoteConnection {
    pub fn new(socket: WebSocketStream<TcpStream>) -> Self {
        let now = Instant::now();
        Self {
            socket: WebSocket::Plain(socket),

            last_ping_start: now,
            last_ping_id: 0,
        }
    }

    pub fn new_with_tls(socket: WebSocketStream<TlsStream<TcpStream>>) -> Self {
        let now = Instant::now();
        Self {
            socket: WebSocket::Tls(socket),

            last_ping_start: now,
            last_ping_id: 0,
        }
    }

    pub async fn send(&mut self, message: RemoteMessage) -> ConnResult<()> {
        self.socket
            .send(Message::Text(serde_json::to_string(&message)?))
            .await
            .map_err(|err| err.into())
    }

    pub async fn ping(&mut self) -> ConnResult<()> {
        let now = Instant::now();
        self.last_ping_id = self.last_ping_id.overflowing_add(1).0;
        self.last_ping_start = now;

        self.socket
            .send(Message::Ping(self.last_ping_id.to_le_bytes().to_vec()))
            .await
            .map_err(|err| err.into())
    }

    pub async fn recv(&mut self) -> ConnResult<RemoteMessage> {
        loop {
            if let Some(msg) = self.socket.next().await {
                match msg? {
                    Message::Text(text) => return Ok(serde_json::from_str(&text)?),
                    Message::Close(_reason) => return Ok(RemoteMessage::Close),
                    Message::Pong(payload) => {
                        if payload.try_into().map(u16::from_le_bytes) == Ok(self.last_ping_id) {
                            let duration =
                                Instant::now().checked_duration_since(self.last_ping_start);
                            if let Some(duration) = duration {
                                return Ok(RemoteMessage::Ping(duration));
                            }
                        }
                    }
                    Message::Ping(payload) => {
                        let _ = self.socket.send(Message::Pong(payload)).await;
                    }
                    msg => return Err(ConnError::UnsupportedMessageFormat(msg)),
                }
            } else {
                return Ok(RemoteMessage::Close);
            }
        }
    }
}
