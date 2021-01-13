use std::mem::replace;

use log::{error, info, warn};
use rand::distributions::Alphanumeric;
use rand::Rng;
use tokio::sync::mpsc;

use uactor::blocking::Addr;

use crate::game_server::{GameServerAddr, GameServerMessage};
use crate::remote::{RemoteConnection, RemoteMessage};
use crate::room::{GamePlayerMessage, RoomAddr, RoomMessage};

const TO_BE_CREATED: &str = "<to be created>";

pub struct Player {
    channel: mpsc::Receiver<GamePlayerMessage>,
    addr: mpsc::Sender<GamePlayerMessage>,

    id: String,
    room_id: Option<String>,
    room: Option<RoomAddr>,
    game_server: GameServerAddr,
    remote: RemoteConnection,
}

pub type PlayerAddr = mpsc::Sender<GamePlayerMessage>;

impl Player {
    pub fn new(remote: RemoteConnection, game_server: GameServerAddr) -> Self {
        let (tx, rx) = mpsc::channel(16);
        Self {
            channel: rx,
            addr: tx,

            id: Self::gen_id(),
            remote,
            game_server,
            room: None,
            room_id: None,
        }
    }

    pub fn gen_id() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .collect::<String>()
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    async fn leave_old_room(&mut self) {
        if let Some(old_room_id) = &self.room_id {
            info!("{}: Leaves already joined room {}", self.id, old_room_id);
            if let Some(old_room) = replace(&mut self.room, None) {
                old_room
                    .send(RoomMessage::PlayerLeft(self.id.to_string()))
                    .await
                    .unwrap();
            }
        }
    }

    async fn on_remote_message(&mut self, msg: RemoteMessage) -> bool {
        match msg {
            RemoteMessage::Close => {
                info!("{}: Player disconnected friendly", self.id);
                return false;
            }
            RemoteMessage::CreateRoom { deck } => {
                info!("{}: Wants to create a room", self.id);
                self.leave_old_room().await;
                self.room_id = Some(TO_BE_CREATED.to_string()); // as marker
                self.game_server
                    .send(GameServerMessage::Create {
                        player_id: self.id.clone(),
                        player: self.addr(),
                        deck,
                    })
                    .await
                    .unwrap(); // TODO: Result
            }
            RemoteMessage::JoinRoom { room: room } => {
                info!("{}: Wants to join {}", self.id, &room);
                if self.room_id.as_ref() == Some(&room) {
                    info!("{}: Already joined {}", self.id, &room);
                    return true;
                }

                self.leave_old_room().await;
                self.room_id = Some(room.clone());
                self.game_server
                    .send(GameServerMessage::Join {
                        room,
                        player_id: self.id.clone(),
                        player: self.addr(),
                    })
                    .await
                    .unwrap(); // TODO: Result
            }
            RemoteMessage::Vote { vote } => {
                if let Some(ref mut room) = &mut self.room {
                    info!("{}: Voted {:?}", self.id, &vote);
                    room.send(RoomMessage::PlayerVoted(self.id.clone(), vote))
                        .await
                        .unwrap();
                } else {
                    warn!("{}: No room to send vote to", self.id);
                }
            }
            RemoteMessage::ForceOpen => {
                if let Some(ref mut room) = &mut self.room {
                    info!("{}: Force open", self.id);
                    room.send(RoomMessage::ForceOpen).await.unwrap()
                } else {
                    warn!("{}: No room to force open", self.id);
                }
            }
            RemoteMessage::Restart => {
                if let Some(ref mut room) = &mut self.room {
                    info!("{}: Restart", self.id);
                    room.send(RoomMessage::Restart).await.unwrap()
                } else {
                    warn!("{}: No room to restart", self.id);
                }
            }
            _ => {
                info!("{}: Ignored `{:?}`", self.id, msg);
            }
        }
        true
    }

    pub fn addr(&self) -> Addr<GamePlayerMessage> {
        self.addr.clone()
    }

    pub async fn run(&mut self) {
        self.setup().await;

        loop {
            tokio::select! {
                maybe_recv_res = self.remote.recv() => {
                    if let Some(recv_res) = maybe_recv_res {
                        match recv_res {
                            Ok(msg) => if !self.on_remote_message(msg).await {
                                break;
                            }
                            Err(err) => {
                                // TODO
                                error!("{}: Message error: {}", self.id, err)
                            }
                        };
                    } else {
                        info!("{}: Player disconnected", self.id);
                        break;
                    }
                },
                recv_res = self.channel.recv() => {
                    if let Some(msg) = recv_res {
                        self.on_message(msg).await
                    } else {
                        break;
                    }
                }
            }
        }

        self.tear_down().await;
    }

    async fn on_message(&mut self, msg: GamePlayerMessage) {
        match msg {
            GamePlayerMessage::Welcome(id, room, game_state, players) => {
                if self.room_id.as_ref() == Some(&id)
                    || self.room_id.as_ref().map(|e| &e as &str) == Some(&TO_BE_CREATED)
                {
                    info!("{}: Joined {}", self.id, id);
                    self.room = Some(room);
                    self.remote
                        .send(RemoteMessage::Joined {
                            room: id,
                            state: game_state,
                            players,
                        })
                        .await
                        .unwrap(); // TODO: Result
                } else {
                    info!(
                        "{}: Reject welcome of room {}, got {:?}",
                        self.id, id, self.room_id
                    );
                    room.send(RoomMessage::PlayerLeft(self.id.to_string()))
                        .await
                        .unwrap();
                }
            }
            GamePlayerMessage::Rejected(reason) => {
                warn!("{}: Player was rejected: {:?}", self.id, reason);
            }
            GamePlayerMessage::PlayerJoined(player) => {
                self.remote
                    .send(RemoteMessage::PlayerJoined { player })
                    .await
                    .unwrap();
            }
            GamePlayerMessage::PlayerChanged(player) => {
                self.remote
                    .send(RemoteMessage::PlayerChanged { player })
                    .await
                    .unwrap();
            }
            GamePlayerMessage::PlayerLeft(player_id) => {
                self.remote
                    .send(RemoteMessage::PlayerLeft { player_id })
                    .await
                    .unwrap();
            }
            GamePlayerMessage::GameStateChanged(game_state) => {
                self.remote
                    .send(RemoteMessage::GameChanged { game_state })
                    .await
                    .unwrap();
            }
        }
    }

    async fn setup(&mut self) {
        let welcome = RemoteMessage::Welcome {
            player_id: self.id().to_string(),
        };
        self.remote.send(welcome).await.unwrap();
    }

    async fn tear_down(&mut self) {
        if let Some(room) = self.room.as_mut() {
            room.send(RoomMessage::PlayerLeft(self.id.to_string()))
                .await
                .unwrap();
        }
    }
}
