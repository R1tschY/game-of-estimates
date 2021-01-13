use std::collections::HashMap;

use tokio::sync::mpsc;

use uactor::blocking::{Actor, Context};

use crate::player::PlayerAddr;
use crate::room::{GamePlayerMessage, RejectReason, Room, RoomAddr, RoomMessage};

#[derive(Debug)]
pub enum GameServerMessage {
    Join {
        room: String,
        player_id: String,
        player: PlayerAddr,
    },
    Create {
        player_id: String,
        player: PlayerAddr,
        deck: String,
    },
}

pub struct GameServer {
    rooms: HashMap<String, RoomAddr>,
}

pub type GameServerAddr = mpsc::Sender<GameServerMessage>;

impl GameServer {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }

    pub fn find_new_game_id(&self) -> Option<String> {
        for digits in 6..20 {
            let id = Room::gen_id(digits);
            if !self.rooms.contains_key(&id) {
                return Some(id);
            }
        }

        None
    }
}

#[async_trait::async_trait]
impl Actor for GameServer {
    type Message = GameServerMessage;
    type Context = Context<Self>;

    async fn on_message(&mut self, msg: Self::Message, _ctx: &Context<Self>) {
        match msg {
            GameServerMessage::Join {
                room,
                player_id,
                player,
            } => {
                if let Some(room_addr) = self.rooms.get_mut(&room) {
                    room_addr
                        .send(RoomMessage::JoinRequest(player_id, player))
                        .await
                        .unwrap(); // TODO: Result
                } else {
                    player
                        .send(GamePlayerMessage::Rejected(RejectReason::GameNotFound))
                        .await
                        .unwrap(); // TODO: Result
                }
            }

            GameServerMessage::Create {
                player_id,
                player,
                deck,
            } => {
                if let Some(room_id) = self.find_new_game_id() {
                    let room = Room::new(&room_id, (player_id, player), deck);
                    self.rooms.insert(room_id, room.start());
                } else {
                    player
                        .send(GamePlayerMessage::Rejected(RejectReason::CreateGameError))
                        .await
                        .unwrap(); // TODO: Result
                }
            }
        }
    }
}
