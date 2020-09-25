use std::collections::HashMap;

use tokio::sync::mpsc;

use crate::actor::{run_actor, Actor};
use crate::game::{Game, GameAddr, GameMessage, GamePlayerMessage, RejectReason};
use crate::player::PlayerAddr;

#[derive(Debug)]
pub enum GameServerMessage {
    Join {
        game: String,
        player_id: String,
        player: PlayerAddr,
    },
    Create {
        player_id: String,
        player: PlayerAddr,
    },
}

pub struct GameServer {
    channel: mpsc::Receiver<GameServerMessage>,
    addr: mpsc::Sender<GameServerMessage>,

    games: HashMap<String, GameAddr>,
}

pub type GameServerAddr = mpsc::Sender<GameServerMessage>;

impl GameServer {
    pub fn new() -> Self {
        let (addr, channel) = mpsc::channel(100);
        Self {
            channel,
            addr,

            games: HashMap::new(),
        }
    }

    pub fn find_new_game_id(&self) -> Option<String> {
        for digits in 6..20 {
            let id = Game::gen_id(digits);
            if !self.games.contains_key(&id) {
                return Some(id);
            }
        }

        None
    }
}

#[async_trait::async_trait]
impl Actor for GameServer {
    type Message = GameServerMessage;

    fn addr(&self) -> GameServerAddr {
        self.addr.clone()
    }

    async fn recv(&mut self) -> Option<GameServerMessage> {
        self.channel.recv().await
    }

    async fn on_message(&mut self, msg: Self::Message) {
        match msg {
            GameServerMessage::Join {
                game,
                player_id,
                mut player,
            } => {
                if let Some(game_addr) = self.games.get_mut(&game) {
                    game_addr
                        .send(GameMessage::JoinRequest(player_id, player))
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
                mut player,
            } => {
                if let Some(game_id) = self.find_new_game_id() {
                    let game = Game::new(&game_id, (player_id, player));
                    self.games.insert(game_id, game.addr());
                    tokio::spawn(run_actor(game));
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
