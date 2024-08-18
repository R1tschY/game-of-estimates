use log::error;
use std::collections::HashMap;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use uactor::blocking::Actor;
use uactor::tokio::blocking::Context;

use crate::player::{PlayerAddr, PlayerInformation};
use crate::ports::RoomRepositoryRef;
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
        reply: oneshot::Sender<String>,
    },
}

pub struct GameServer {
    rooms: HashMap<String, RoomAddr>,
    room_repo: RoomRepositoryRef,
}

pub type GameServerAddr = mpsc::Sender<GameServerMessage>;

impl GameServer {
    pub fn new(room_repo: RoomRepositoryRef) -> Self {
        Self {
            rooms: Default::default(),
            room_repo,
        }
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
                    match self.room_repo.get_room_events(&room).await {
                        Ok(events) => {
                            if events.is_empty() {
                                Self::send_rejection(&player_addr, RejectReason::RoomDoesNotExist)
                                    .await;
                            }

                            if let Some(restored_room) =
                                Room::restore(&room, events, self.room_repo.clone())
                            {
                                self.rooms.insert(room, restored_room.start());
                            } else {
                                error!("Failed to restore room {}", room);
                                Self::send_rejection(&player_addr, RejectReason::JoinGameError)
                                    .await;
                            }
                        }
                        Err(db_err) => {
                            error!("Failed to restore room {}: {:?}", room, db_err);
                            Self::send_rejection(&player_addr, RejectReason::JoinGameError).await;
                        }
                    }
                }
            }

            GameServerMessage::Create { deck, reply } => {
                let room_id = Room::gen_id();
                let room = Room::new(&room_id, deck, self.room_repo.clone());
                self.rooms.insert(room_id.clone(), room.start());
                let _ = reply.send(room_id);
            }
        }
    }
}
