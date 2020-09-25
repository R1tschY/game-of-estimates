use std::mem::replace;

use log::{info, warn};
use rand::distributions::Alphanumeric;
use rand::Rng;
use tokio::sync::mpsc;

use crate::actor::{Actor, Addr};
use crate::game::{GameAddr, GameMessage, GamePlayerMessage};
use crate::game_server::{GameServerAddr, GameServerMessage};
use crate::remote::{RemoteConnection, RemoteMessage};

pub struct Player {
    channel: mpsc::Receiver<GamePlayerMessage>,
    addr: mpsc::Sender<GamePlayerMessage>,

    id: String,
    game_id: Option<String>,
    game: Option<GameAddr>,
    game_server: GameServerAddr,
    remote: RemoteConnection,
}

pub type PlayerAddr = mpsc::Sender<GamePlayerMessage>;

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

    async fn leave_old_game(&mut self) {
        if let Some(old_game_id) = &self.game_id {
            info!("{}: Leaves already joined game {}", self.id, old_game_id);
            if let Some(mut old_game) = replace(&mut self.game, None) {
                old_game
                    .send(GameMessage::PlayerLeft(self.id.to_string()))
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
            RemoteMessage::CreateGame => {
                info!("{}: Wants to create a game", self.id);
                self.leave_old_game().await;
                self.game_id = Some("<to be created>".to_string()); // as marker
                self.game_server
                    .send(GameServerMessage::Create {
                        player_id: self.id.clone(),
                        player: self.addr(),
                    })
                    .await
                    .unwrap(); // TODO: Result
            }
            RemoteMessage::JoinGame { game } => {
                info!("{}: Wants to join {}", self.id, &game);
                if self.game_id.as_ref() == Some(&game) {
                    info!("{}: Already joined {}", self.id, &game);
                    return true;
                }

                self.leave_old_game().await;
                self.game_id = Some(game.clone());
                self.game_server
                    .send(GameServerMessage::Join {
                        game,
                        player_id: self.id.clone(),
                        player: self.addr(),
                    })
                    .await
                    .unwrap(); // TODO: Result
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
    type Message = GamePlayerMessage;

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
                            Err(_err) => {
                                // TODO
                                panic!()
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

    async fn on_message(&mut self, msg: GamePlayerMessage) {
        match msg {
            GamePlayerMessage::Welcome(id, mut game, game_state, players) => {
                if self.game_id.as_ref() == Some(&id) {
                    info!("{}: Joined {}", self.id, id);
                    self.game = Some(game);
                    self.remote
                        .send(RemoteMessage::Joined {
                            game: id,
                            state: game_state,
                            players,
                        })
                        .await
                        .unwrap(); // TODO: Result
                } else {
                    info!("{}: Reject welcome of game {}", self.id, id);
                    game.send(GameMessage::PlayerLeft(self.id.to_string()))
                        .await
                        .unwrap();
                }
            }
            GamePlayerMessage::Rejected(reason) => {
                warn!("{}: Player was rejected: {:?}", self.id, reason);
            }
            GamePlayerMessage::OtherPlayerJoined(player) => {
                self.remote
                    .send(RemoteMessage::PlayerJoined { player })
                    .await
                    .unwrap();
            }
            GamePlayerMessage::OtherPlayerChanged(player) => {
                self.remote
                    .send(RemoteMessage::PlayerChanged { player })
                    .await
                    .unwrap();
            }
            GamePlayerMessage::OtherPlayerLeft(player_id) => {
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
        if let Some(game) = self.game.as_mut() {
            game.send(GameMessage::PlayerLeft(self.id.to_string()))
                .await
                .unwrap();
        }
    }
}
