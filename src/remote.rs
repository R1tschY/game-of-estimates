use std::convert::TryInto;

use futures_util::{SinkExt, StreamExt};
use quick_error::quick_error;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, Instant};
use warp::ws::Message;

use crate::room::{GameState, PlayerState};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
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
        Ws(err: warp::Error) {
            display("Web socket error: {}", err)
            from()
        }
        Json(err: serde_json::Error) {
            display("JSON error: {}", err)
            from()
        }
        UnsupportedMessageFormat(msg: Message) {
            display("Unsupported web socket message: {:?}", msg)
        }
    }
}

type ConnResult<T> = Result<T, ConnError>;

pub struct RemoteConnection {
    socket: warp::ws::WebSocket,

    last_ping_start: Instant,
    last_ping_id: u16,
}

impl RemoteConnection {
    pub fn new(socket: warp::ws::WebSocket) -> Self {
        let now = Instant::now();
        Self {
            socket,

            last_ping_start: now,
            last_ping_id: 0,
        }
    }

    pub async fn send(&mut self, message: RemoteMessage) -> ConnResult<()> {
        self.socket
            .send(Message::text(serde_json::to_string(&message)?))
            .await
            .map_err(|err| err.into())
    }

    pub async fn ping(&mut self) -> ConnResult<()> {
        let now = Instant::now();
        self.last_ping_id = self.last_ping_id.overflowing_add(1).0;
        self.last_ping_start = now;

        self.socket
            .send(Message::ping(self.last_ping_id.to_le_bytes().to_vec()))
            .await
            .map_err(|err| err.into())
    }

    pub async fn recv(&mut self) -> ConnResult<RemoteMessage> {
        while let Some(msg) = self.socket.next().await {
            match msg? {
                msg if msg.is_text() => return Ok(serde_json::from_str(msg.to_str().unwrap())?),
                msg if msg.is_close() => return Ok(RemoteMessage::Close),
                msg if msg.is_pong() => {
                    if msg.into_bytes().try_into().map(u16::from_le_bytes) == Ok(self.last_ping_id)
                    {
                        let duration = Instant::now().checked_duration_since(self.last_ping_start);
                        if let Some(duration) = duration {
                            return Ok(RemoteMessage::Ping(duration));
                        }
                    }
                }
                msg if msg.is_ping() => {
                    let _ = self.socket.send(Message::pong(msg.into_bytes())).await;
                }
                msg => return Err(ConnError::UnsupportedMessageFormat(msg)),
            }
        }

        Ok(RemoteMessage::Close)
    }
}
