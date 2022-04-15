use std::collections::HashMap;
use std::fmt;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use uactor::blocking::{Actor, Context};

use crate::player::{PlayerAddr, PlayerInformation};
use crate::room::{GamePlayerMessage, RejectReason, Room, RoomAddr, RoomMessage};

/// Return envelope to send result of an operation back to sender
pub struct ReturnEnvelope<T> {
    channel: oneshot::Sender<T>,
}

impl<T> ReturnEnvelope<T> {
    fn new(channel: oneshot::Sender<T>) -> Self {
        Self { channel }
    }

    pub fn channel() -> (ReturnEnvelope<T>, oneshot::Receiver<T>) {
        let (send, recv) = oneshot::channel();
        (ReturnEnvelope::new(send), recv)
    }

    pub fn send(self, value: T) {
        let _ = self.channel.send(value);
    }
}

impl<T> fmt::Debug for ReturnEnvelope<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ReturnEnvelope")
    }
}

#[derive(Debug)]
pub enum GameServerMessage {
    Join {
        room: String,

        player_addr: PlayerAddr,
        player: PlayerInformation,
    },
    Create {
        deck: String,

        player_addr: PlayerAddr,
        player: PlayerInformation,
    },
    ExternalCreate {
        deck: String,
        ret: ReturnEnvelope<Result<String, RejectReason>>,
    },
}

#[derive(Default)]
pub struct GameServer {
    rooms: HashMap<String, RoomAddr>,
}

pub type GameServerAddr = mpsc::Sender<GameServerMessage>;

impl GameServer {
    pub fn find_new_game_id(&self) -> Option<String> {
        for digits in 6..20 {
            let id = Room::gen_id(digits);
            if !self.rooms.contains_key(&id) {
                return Some(id);
            }
        }

        None
    }

    async fn send_rejection(player: &PlayerAddr, reason: RejectReason) {
        let _ = player.send(GamePlayerMessage::Rejected(reason)).await;
    }
}

#[async_trait::async_trait]
impl Actor for GameServer {
    type Message = GameServerMessage;
    type Context = Context<Self>;

    async fn on_message(&mut self, msg: Self::Message, _ctx: &mut Context<Self>) {
        match msg {
            GameServerMessage::Join {
                room,
                player_addr,
                player,
            } => {
                if let Some(room_addr) = self.rooms.get_mut(&room) {
                    let result = room_addr
                        .send(RoomMessage::JoinRequest(player_addr.clone(), player))
                        .await;
                    if result.is_err() {
                        // room does not exist
                        Self::send_rejection(&player_addr, RejectReason::RoomDoesNotExist).await;
                        self.rooms.remove(&room);
                    }
                } else {
                    Self::send_rejection(&player_addr, RejectReason::RoomDoesNotExist).await;
                }
            }

            GameServerMessage::Create {
                player_addr,
                player,
                deck,
            } => {
                if let Some(room_id) = self.find_new_game_id() {
                    let room = Room::new_with_creator(&room_id, (player_addr, player), deck);
                    self.rooms.insert(room_id, room.start());
                } else {
                    Self::send_rejection(&player_addr, RejectReason::CreateGameError).await;
                }
            }

            GameServerMessage::ExternalCreate { deck, ret } => {
                ret.send(if let Some(room_id) = self.find_new_game_id() {
                    let room = Room::new(&room_id, deck);
                    self.rooms.insert(room_id.clone(), room.start());
                    Ok(room_id)
                } else {
                    Err(RejectReason::CreateGameError)
                });
            }
        }
    }
}
