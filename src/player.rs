use std::mem::replace;

use log::{error, info, warn};
use rand::distributions::Alphanumeric;
use rand::Rng;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration, Interval};

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
    ping_interval: Interval,

    name: Option<String>,
    voter: bool,
}

pub type PlayerAddr = mpsc::Sender<GamePlayerMessage>;

#[derive(Debug, Clone)]
pub struct PlayerInformation {
    pub id: String,
    pub voter: bool,
    pub name: Option<String>,
}

impl Player {
    pub fn new(remote: RemoteConnection, game_server: GameServerAddr) -> Self {
        let (tx, rx) = mpsc::channel(16);
        Self {
            channel: rx,
            addr: tx,

            id: Self::gen_id(),
            game_server,
            room: None,
            room_id: None,

            remote,
            ping_interval: interval(Duration::from_secs(30)),

            name: None,
            voter: true,
        }
    }

    pub fn gen_id() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect::<String>()
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    async fn leave_room(&self, room: &RoomAddr) {
        // ignore result, because error means room does not exist, that we left
        let _ = room
            .send(RoomMessage::PlayerLeft(self.id.to_string()))
            .await;
    }

    async fn leave_old_room(&mut self) {
        if let Some(old_room_id) = &self.room_id {
            info!("{}: Leaves already joined room {}", self.id, old_room_id);
            if let Some(old_room) = replace(&mut self.room, None) {
                self.leave_room(&old_room).await;
            }
        }
    }

    async fn send_to_room(&mut self, msg: RoomMessage) {
        if let Some(ref room) = &self.room {
            if room.send(msg).await.is_err() {
                error!("{}: Room does not exist anymore", self.id);
                self.room = None;
                self.room_id = None;
                self.send_to_remote(RemoteMessage::Rejected).await;
            }
        } else {
            warn!("{}: No room to interact with", self.id);
        }
    }

    async fn send_join_message(&mut self, msg: GameServerMessage) {
        if self.game_server.send(msg).await.is_err() {
            error!("{}: Join room does not exist", self.id);
            self.room = None;
            self.room_id = None;
            self.send_to_remote(RemoteMessage::Rejected).await;
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
                let information = self.get_player_information();
                self.send_join_message(GameServerMessage::Create {
                    player_addr: self.addr(),
                    player: information,
                    deck,
                })
                .await;
            }
            RemoteMessage::JoinRoom { room } => {
                info!("{}: Wants to join {}", self.id, &room);
                if self.room_id.as_ref() == Some(&room) {
                    info!("{}: Already joined {}", self.id, &room);
                    return true;
                }

                self.leave_old_room().await;
                self.room_id = Some(room.clone());
                let player_information = self.get_player_information();
                self.send_join_message(GameServerMessage::Join {
                    room,
                    player_addr: self.addr(),
                    player: player_information,
                })
                .await;
            }
            RemoteMessage::Vote { vote } => {
                info!("{}: Voted {:?}", self.id, &vote);
                self.send_to_room(RoomMessage::PlayerVoted(self.id.clone(), vote))
                    .await;
            }
            RemoteMessage::UpdatePlayer { voter, name } => {
                info!(
                    "{}: Update player: voter={:?} name={:?}",
                    self.id, &voter, &name
                );
                self.voter = voter;
                self.name = name.clone();

                if self.room.is_some() {
                    self.send_to_room(RoomMessage::UpdatePlayer {
                        id: self.id.clone(),
                        voter,
                        name,
                    })
                    .await;
                }
            }
            RemoteMessage::ForceOpen => {
                info!("{}: Force open", self.id);
                self.send_to_room(RoomMessage::ForceOpen).await;
            }
            RemoteMessage::Restart => {
                info!("{}: Restart", self.id);
                self.send_to_room(RoomMessage::Restart).await;
            }
            RemoteMessage::Ping(duration) => {
                info!("{}: Ping {}ms", self.id, duration.as_millis())
            }
            _ => {
                info!("{}: Ignored `{:?}`", self.id, msg);
            }
        }
        true
    }

    fn get_player_information(&mut self) -> PlayerInformation {
        PlayerInformation {
            id: self.id.clone(),
            voter: self.voter,
            name: self.name.clone(),
        }
    }

    async fn send_to_remote(&mut self, msg: RemoteMessage) {
        let result = self.remote.send(msg).await;
        if let Err(err) = result {
            error!("{}: Failed to send message to remote: {:?}", self.id, err);
        }
    }

    pub fn addr(&self) -> Addr<GamePlayerMessage> {
        self.addr.clone()
    }

    pub async fn run(&mut self) {
        self.setup().await;

        loop {
            tokio::select! {
                maybe_recv_res = self.remote.recv() => {
                    match maybe_recv_res {
                        Ok(msg) => if !self.on_remote_message(msg).await {
                            break;
                        }
                        Err(err) => {
                            // TODO
                            error!("{}: Message error: {}", self.id, err)
                        }
                    }
                },
                recv_res = self.channel.recv() => {
                    if let Some(msg) = recv_res {
                        self.on_message(msg).await
                    } else {
                        break;
                    }
                }
                _ = self.ping_interval.tick() => {
                    if let Err(err) = self.remote.ping().await {
                        warn!("{}: Failed to send ping: {:?}", self.id, err);
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
                    self.send_to_remote(RemoteMessage::Joined {
                        room: id,
                        state: game_state,
                        players,
                    })
                    .await;
                } else {
                    info!(
                        "{}: Reject welcome of room {}, got {:?}",
                        self.id, id, self.room_id
                    );
                    self.leave_room(&room).await;
                }
            }
            GamePlayerMessage::Rejected(reason) => {
                warn!("{}: Player was rejected: {:?}", self.id, reason);
                self.send_to_remote(RemoteMessage::Rejected).await;
            }
            GamePlayerMessage::PlayerJoined(player) => {
                self.send_to_remote(RemoteMessage::PlayerJoined { player })
                    .await;
            }
            GamePlayerMessage::PlayerChanged(player) => {
                self.send_to_remote(RemoteMessage::PlayerChanged { player })
                    .await;
            }
            GamePlayerMessage::PlayerLeft(player_id) => {
                self.send_to_remote(RemoteMessage::PlayerLeft { player_id })
                    .await;
            }
            GamePlayerMessage::GameStateChanged(game_state) => {
                self.send_to_remote(RemoteMessage::GameChanged { game_state })
                    .await;
            }
        }
    }

    async fn setup(&mut self) {
        let welcome = RemoteMessage::Welcome {
            player_id: self.id().to_string(),
        };
        self.send_to_remote(welcome).await;
        let _ = self.remote.ping().await;
    }

    async fn tear_down(&mut self) {
        if let Some(room) = self.room.as_mut() {
            let room = room.clone();
            self.leave_room(&room).await;
        }
    }
}
