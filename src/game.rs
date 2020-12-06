use std::collections::HashMap;

use log::warn;
use rand::distributions::Uniform;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::actor::{Actor, ActorContext, Addr, Context};
use crate::player::PlayerAddr;

#[derive(Debug)]
pub enum GameMessage {
    JoinRequest(String, PlayerAddr),
    PlayerLeft(String),
    PlayerVoted(String, Option<String>),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum RejectReason {
    GameNotFound,
    CreateGameError,
    JoinGameError,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    deck: String,
    open: bool,
    votes: HashMap<String, Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerState {
    id: String,
    voter: bool,
}

#[derive(Debug, Clone)]
pub enum GamePlayerMessage {
    // join mgmt
    Welcome(String, GameAddr, GameState, Vec<PlayerState>),
    Rejected(RejectReason),

    // room state sync
    OtherPlayerJoined(PlayerState),
    OtherPlayerChanged(PlayerState),
    OtherPlayerLeft(String),
    GameStateChanged(GameState),
}

struct GamePlayer {
    addr: PlayerAddr,

    id: String,
    voter: bool,
    vote: Option<String>,
}

impl GamePlayer {
    pub fn new(id: &str, addr: PlayerAddr, voter: bool) -> Self {
        Self {
            id: id.to_string(),
            addr,
            voter,
            vote: None,
        }
    }

    fn to_state(&self) -> PlayerState {
        PlayerState {
            id: self.id.clone(),
            voter: self.voter,
        }
    }
}

pub struct Game {
    id: String,
    deck: String,
    players: HashMap<String, GamePlayer>,
    open: bool,
}

impl Game {
    pub fn new(id: &str, creator: (String, PlayerAddr), deck: String) -> Self {
        let mut players = HashMap::new();
        let game_player = GamePlayer::new(&creator.0, creator.1, false);
        players.insert(creator.0, game_player);

        Self {
            id: id.to_string(),
            players,
            open: false,
            deck,
        }
    }

    pub fn gen_id(digits: u8) -> String {
        rand::thread_rng()
            .sample(&Uniform::from(0..10u32.pow(digits as u32)))
            .to_string()
    }

    async fn add_player(&mut self, player_id: String, mut player: PlayerAddr, ctx: &Context<Self>) {
        let game_player = GamePlayer::new(&player_id, player.clone(), true);
        let game_player_state = game_player.to_state();
        self.players.insert(player_id, game_player);

        // welcome
        let players_state = self.players.values().map(|p| p.to_state()).collect();
        player
            .send(GamePlayerMessage::Welcome(
                self.id.clone(),
                ctx.addr(),
                self.to_state(),
                players_state,
            ))
            .await
            .unwrap();

        // introduce
        for player in self.players.values_mut() {
            if player.id != game_player_state.id {
                player
                    .addr
                    .send(GamePlayerMessage::OtherPlayerJoined(
                        game_player_state.clone(),
                    ))
                    .await
                    .unwrap();
            }
        }
    }

    async fn remove_player(&mut self, player_id: &str) {
        self.players.remove(player_id);

        // announce
        for player in self.players.values_mut() {
            player
                .addr
                .send(GamePlayerMessage::OtherPlayerLeft(player_id.to_string()))
                .await
                .unwrap();
        }
    }

    async fn set_vote(&mut self, player_id: &str, vote: Option<String>) {
        if let Some(mut player) = self.players.get_mut(player_id) {
            if player.voter {
                player.vote = vote;
            } else {
                warn!("{}: Non-voter voted", player_id);
                return;
            }
        } else {
            warn!("{}: Failed to set vote for non-existing player", player_id);
            return;
        }

        self.send_game_state().await;
    }

    async fn send_game_state(&mut self) {
        let state = self.to_state();
        for player in self.players.values_mut() {
            player
                .addr
                .send(GamePlayerMessage::GameStateChanged(state.clone()))
                .await
                .unwrap();
        }
    }

    fn to_state(&self) -> GameState {
        GameState {
            deck: self.deck.clone(),
            open: self.open,
            votes: self
                .players
                .values()
                .filter(|p| p.voter)
                .map(|p| (p.id.clone(), p.vote.clone()))
                .collect(),
        }
    }
}

pub type GameAddr = Addr<GameMessage>;

#[async_trait::async_trait]
impl Actor for Game {
    type Message = GameMessage;
    type Context = Context<Self>;

    async fn on_message(&mut self, msg: Self::Message, ctx: &Context<Self>) {
        match msg {
            GameMessage::JoinRequest(player_id, player) => {
                self.add_player(player_id, player, ctx).await
            }
            GameMessage::PlayerLeft(player) => self.remove_player(&player).await,
            GameMessage::PlayerVoted(player_id, vote) => self.set_vote(&player_id, vote).await,
        }
    }

    async fn setup(&mut self, ctx: &Context<Self>) {
        let players_state: Vec<PlayerState> = self.players.values().map(|p| p.to_state()).collect();
        let game_state = self.to_state();

        for player in self.players.values_mut() {
            player
                .addr
                .send(GamePlayerMessage::Welcome(
                    self.id.clone(),
                    ctx.addr(),
                    game_state.clone(),
                    players_state.clone(),
                ))
                .await
                .unwrap(); // TODO: Result
        }
    }
}
