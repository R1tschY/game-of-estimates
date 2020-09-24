use crate::actor::{Actor, Addr};
use crate::game::{GameAddr, GameMessage};
use crate::game_server::{GameServerAddr, GameServerMessage};
use crate::player::RejectReason::CreateGameError;
use crate::remote::{RemoteConnection, RemoteMessage};
use async_trait::async_trait;
use log::{error, info, warn};
use rand::distributions::Alphanumeric;
use rand::Rng;
use tokio::sync::mpsc;

#[derive(Debug, PartialEq)]
pub enum RejectReason {
    GameNotFound,
    CreateGameError,
    JoinGameError,
}

#[derive(Debug)]
pub enum PlayerMessage {
    Invite(String, GameAddr),
    Rejected(RejectReason),
}

pub struct Player {
    channel: mpsc::Receiver<PlayerMessage>,
    addr: mpsc::Sender<PlayerMessage>,

    id: String,
    game_id: Option<String>,
    game: Option<GameAddr>,
    game_server: GameServerAddr,
    remote: RemoteConnection,
}

pub type PlayerAddr = mpsc::Sender<PlayerMessage>;

impl Player {
    pub fn new(remote: RemoteConnection, game_server: GameServerAddr) -> Self {
        let (tx, rx) = mpsc::channel(8);
        Self {
            channel: rx,
            addr: tx,

            id: Self::gen_id(),
            remote,
            game_server,
            game: None,
            game_id: None,
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

    pub async fn on_remote_message(&mut self, msg: RemoteMessage) -> bool {
        match msg {
            RemoteMessage::Close => {
                info!("{}: Player disconnected friendly", self.id);
                return false;
            }
            RemoteMessage::CreateGame => {
                info!("{}: Wants to create a room", self.id);
                if self.game_id.is_some() {
                    error!("{}: Already joined a room", self.id);
                    self.remote.send(RemoteMessage::Rejected).await;
                } else {
                    self.game_id = Some("<to be created>".to_string()); // as marker
                    self.game_server
                        .send(GameServerMessage::Create {
                            player_id: self.id.clone(),
                            player: self.addr(),
                        })
                        .await; // TODO: Result
                }
            }
            RemoteMessage::JoinGame { game } => {
                info!("{}: Wants to join {}", self.id, &game);
                if self.game_id.is_some() {
                    error!("{}: Already joined a room", self.id);
                    self.remote.send(RemoteMessage::Rejected).await;
                } else {
                    self.game_id = Some(game.clone());
                    self.game_server
                        .send(GameServerMessage::Join {
                            game,
                            player_id: self.id.clone(),
                            player: self.addr(),
                        })
                        .await; // TODO: Result
                }
            }
            _ => {
                info!("{}: Ignored `{:?}`", self.id, msg);
            }
        }
        true
    }
}

#[async_trait::async_trait]
impl Actor for Player {
    type Message = PlayerMessage;

    fn addr(&self) -> Addr<Self::Message> {
        self.addr.clone()
    }

    async fn recv(&mut self) -> Option<Self::Message> {
        self.channel.recv().await
    }

    async fn run(&mut self) {
        self.setup().await;

        let this = self; // needed because #[async_trait] ignores macros
        loop {
            tokio::select! {
                maybe_recv_res = this.remote.recv() => {
                    if let Some(recv_res) = maybe_recv_res {
                        match recv_res {
                            Ok(msg) => if !this.on_remote_message(msg).await {
                                break;
                            }
                            Err(err) => {
                                // TODO
                            }
                        };
                    } else {
                        info!("{}: Player disconnected", this.id);
                        break;
                    }
                },
                recv_res = this.channel.recv() => {
                    if let Some(msg) = recv_res {
                        this.on_message(msg).await
                    } else {
                        break;
                    }
                }
            }
        }

        this.tear_down().await;
    }

    async fn setup(&mut self) {
        let welcome = RemoteMessage::Welcome {
            player_id: self.id().to_string(),
        };
        self.remote.send(welcome).await.unwrap();
    }

    async fn tear_down(&mut self) {
        if let Some(mut game) = self.game.as_mut() {
            game.send(GameMessage::PlayerLeft(self.id.to_string()))
                .await;
        }
    }

    async fn on_message(&mut self, msg: PlayerMessage) {
        match msg {
            PlayerMessage::Invite(id, game) => {
                if self.game.is_some() {
                    // TODO: check id
                    error!("{}: Player invited multiple times", self.id);
                } else {
                    info!("{}: Joined {}", self.id, id);
                    self.game = Some(game);
                    self.remote.send(RemoteMessage::Joined { game: id }).await; // TODO: Result
                }
            }
            PlayerMessage::Rejected(reason) => {
                warn!("{}: Player was rejected: {:?}", self.id, reason);
            }
        }
    }
}
