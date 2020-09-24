use crate::actor::{Actor, Addr};
use crate::player::{PlayerAddr, PlayerMessage};
use futures_util::SinkExt;
use rand::distributions::{Alphanumeric, Uniform};
use rand::Rng;
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum GameMessage {
    InvitePlayer(String, PlayerAddr),
    PlayerLeft(String),
}

pub struct Game {
    channel: mpsc::Receiver<GameMessage>,
    addr: mpsc::Sender<GameMessage>,

    id: String,
    players: HashMap<String, PlayerAddr>,
}

impl Game {
    pub fn new(id: &str, creator: (String, PlayerAddr)) -> Self {
        let (addr, channel) = mpsc::channel(100);

        let mut players = HashMap::new();
        players.insert(creator.0, creator.1);

        Self {
            channel,
            addr,
            id: id.to_string(),
            players,
        }
    }

    pub fn gen_id(digits: u8) -> String {
        rand::thread_rng()
            .sample(&Uniform::from(0..10u32.pow(digits as u32)))
            .to_string()
    }
}

pub type GameAddr = mpsc::Sender<GameMessage>;

#[async_trait::async_trait]
impl Actor for Game {
    type Message = GameMessage;

    fn addr(&self) -> Addr<Self::Message> {
        self.addr.clone()
    }

    async fn recv(&mut self) -> Option<Self::Message> {
        self.channel.recv().await
    }

    async fn on_message(&mut self, msg: Self::Message) {
        match msg {
            GameMessage::InvitePlayer(player_id, mut player) => {
                player
                    .send(PlayerMessage::Invite(self.id.clone(), self.addr()))
                    .await;
                self.players.insert(player_id, player);
            }
            GameMessage::PlayerLeft(player) => {
                self.players.remove(&player);
            }
        }
    }

    async fn setup(&mut self) {
        for mut player in self.players.values_mut() {
            player
                .send(PlayerMessage::Invite(self.id.clone(), self.addr.clone()))
                .await; // TODO: Result
        }
    }
}
