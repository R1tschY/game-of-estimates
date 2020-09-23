use crate::player::RejectReason;
use futures_util::{SinkExt, StreamExt};
use quick_error::quick_error;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::pin::Pin;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::{Error as WsError, Result as WsResult};
use tokio_tungstenite::WebSocketStream;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum RemoteMessage {
    // upstream
    SetVoter { voter: bool },
    SetName { name: String },
    JoinGame { game: String },
    CreateGame,
    Close,

    // downstream
    Welcome { player_id: String },
    Rejected,
    Joined { game: String },
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
    socket: WebSocketStream<TcpStream>,
}

impl RemoteConnection {
    pub fn new(socket: WebSocketStream<TcpStream>) -> Self {
        Self { socket }
    }

    pub async fn send(&mut self, message: RemoteMessage) -> ConnResult<()> {
        self.socket
            .send(Message::Text(serde_json::to_string(&message)?))
            .await
            .map_err(|err| err.into())
    }

    pub async fn recv(&mut self) -> Option<ConnResult<RemoteMessage>> {
        self.socket.next().await.map(Self::from_message)
    }

    fn from_message(msg: WsResult<Message>) -> ConnResult<RemoteMessage> {
        match msg? {
            Message::Text(text) => Ok(serde_json::from_str(&text)?),
            Message::Close(_reason) => Ok(RemoteMessage::Close), // TODO: log reason
            msg => Err(ConnError::UnsupportedMessageFormat(msg)),
        }
    }
}
