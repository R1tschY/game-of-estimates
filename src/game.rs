use crate::actor::{Actor, Addr};
use crate::player::{PlayerAddr, PlayerMessage};
use rand::distributions::{Alphanumeric, Uniform};
use rand::Rng;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum GameMessage {
    InvitePlayer(PlayerAddr),
}

pub struct Game {
    channel: mpsc::Receiver<GameMessage>,
    addr: mpsc::Sender<GameMessage>,

    id: String,
    players: Vec<PlayerAddr>,
}

impl Game {
    pub fn new(id: &str, creator: PlayerAddr) -> Self {
        let (addr, channel) = mpsc::channel(100);
        Self {
            channel,
            addr,
            id: id.to_string(),
            players: vec![creator],
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

    async fn setup(&mut self) {
        for mut player in &mut self.players {
            player
                .send(PlayerMessage::Invite(self.id.clone(), self.addr.clone()))
                .await; // TODO: Result
        }
    }

    async fn on_message(&mut self, msg: Self::Message) {
        match msg {
            GameMessage::InvitePlayer(mut player) => {
                player
                    .send(PlayerMessage::Invite(self.id.clone(), self.addr()))
                    .await;
                self.players.push(player)
            }
        }
    }
}
