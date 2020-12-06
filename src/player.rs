use std::mem::replace;

use log::{error, info, warn};
use rand::distributions::Alphanumeric;
use rand::Rng;
use tokio::sync::mpsc;

use crate::actor::Addr;
use crate::game::{GameAddr, GameMessage, GamePlayerMessage};
use crate::game_server::{GameServerAddr, GameServerMessage};
use crate::remote::{RemoteConnection, RemoteMessage};

const TO_BE_CREATED: &str = "<to be created>";

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
        let (tx, rx) = mpsc::channel(16);
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
            RemoteMessage::CreateGame { deck } => {
                info!("{}: Wants to create a game", self.id);
                self.leave_old_game().await;
                self.game_id = Some(TO_BE_CREATED.to_string()); // as marker
                self.game_server
                    .send(GameServerMessage::Create {
                        player_id: self.id.clone(),
                        player: self.addr(),
                        deck,
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
            RemoteMessage::Vote { vote } => {
                if let Some(ref mut game) = &mut self.game {
                    info!("{}: Voted {:?}", self.id, &vote);
                    game.send(GameMessage::PlayerVoted(self.id.clone(), vote))
                        .await
                        .unwrap();
                } else {
                    warn!("{}: No room to send vote to", self.id);
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
            GamePlayerMessage::Welcome(id, mut game, game_state, players) => {
                if self.game_id.as_ref() == Some(&id)
                    || self.game_id.as_ref().map(|e| &e as &str) == Some(&TO_BE_CREATED)
                {
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
                    info!(
                        "{}: Reject welcome of game {}, got {:?}",
                        self.id, id, self.game_id
                    );
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
