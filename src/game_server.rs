use std::collections::HashMap;

use tokio::sync::mpsc;

use uactor::blocking::{Actor, Context};

use crate::player::{PlayerAddr, PlayerInformation};
use crate::room::{GamePlayerMessage, RejectReason, Room, RoomAddr, RoomMessage};

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
                    let room = Room::new(&room_id, (player_addr, player), deck);
                    self.rooms.insert(room_id, room.start());
                } else {
                    Self::send_rejection(&player_addr, RejectReason::CreateGameError).await;
                }
            }
        }
    }
}
