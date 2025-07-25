use std::collections::HashMap;
use std::fmt::Debug;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use uactor::blocking::{Actor, ActorContext, Addr};
use uactor::tokio::blocking::Context;

use crate::player::{PlayerAddr, PlayerInformation};
use crate::ports::{DbError, RoomRepositoryRef};

#[derive(Debug)]
pub enum RoomMessage {
    JoinRequest(PlayerAddr, PlayerInformation),
    PlayerLeft(String),
    PlayerVoted(String, Option<String>),
    UpdatePlayer {
        id: String,
        voter: bool,
        name: Option<String>,
    },
    ForceOpen,
    Restart,
    Close,

    // internal
    CloseWhenEmpty,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum RejectReason {
    RoomDoesNotExist,
    CreateGameError,
    JoinGameError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    deck: String,
    open: bool,
    votes: HashMap<String, Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerState {
    id: String,
    name: Option<String>,
    voter: bool,
}

#[derive(Debug, Clone)]
pub enum GamePlayerMessage {
    // join mgmt
    Welcome(String, RoomAddr, GameState, Vec<PlayerState>),
    Rejected(RejectReason),

    // room state sync
    PlayerJoined(PlayerState),
    PlayerChanged(PlayerState),
    PlayerLeft(String),
    GameStateChanged(GameState),
}

#[derive(Clone)]
struct GamePlayer {
    addr: PlayerAddr,

    vote: Option<String>,
    info: PlayerInformation,
}

impl GamePlayer {
    pub fn new(addr: PlayerAddr, info: PlayerInformation) -> Self {
        Self {
            addr,
            info,
            vote: None,
        }
    }

    fn to_state(&self) -> PlayerState {
        PlayerState {
            id: self.info.id.clone(),
            name: self.info.name.clone(),
            voter: self.info.voter,
        }
    }
}

pub struct Room {
    id: String,
    deck: String,
    players: HashMap<String, GamePlayer>,
    open: bool,
    repo: RoomRepositoryRef,
}

pub enum RoomEvent {
    Created { deck: String },
    PlayerJoined { player_id: String },
    PlayerLeaved { player_id: String },
}

async fn delayed_message<T: Debug>(addr: Addr<T>, msg: T, duration: Duration) {
    sleep(duration).await;
    let _ = addr.send(msg).await;
}

impl Room {
    pub async fn new(id: &str, deck: String, repo: RoomRepositoryRef) -> Result<Self, DbError> {
        let self_ = Self {
            id: id.to_string(),
            players: HashMap::new(),
            open: false,
            deck,
            repo,
        };

        self_
            .repo
            .append_room_event(
                &self_.id,
                RoomEvent::Created {
                    deck: self_.deck.clone(),
                },
            )
            .await?;

        info!("{}: Created room", id);
        Ok(self_)
    }

    pub fn restore(id: &str, events: Vec<RoomEvent>, repo: RoomRepositoryRef) -> Option<Self> {
        let mut iter = events.into_iter();

        let deck = if let Some(RoomEvent::Created { deck }) = iter.next() {
            deck
        } else {
            return None;
        };

        for evt in iter {
            match evt {
                RoomEvent::Created { .. } => {
                    return None;
                }
                RoomEvent::PlayerJoined { .. } | RoomEvent::PlayerLeaved { .. } => {
                    // ignored: player must join again
                }
            }
        }

        Some(Self {
            id: id.to_string(),
            players: HashMap::default(),
            open: false,
            deck,
            repo,
        })
    }

    pub fn gen_id() -> String {
        URL_SAFE_NO_PAD.encode(Uuid::now_v7().as_bytes())
    }

    async fn send_to_player(&mut self, player: &GamePlayer, msg: GamePlayerMessage) {
        let result = player.addr.send(msg).await;
        if result.is_err() {
            error!(
                "{}: Failed to send message to player {}",
                self.id, player.info.id
            );
            // TODO: self.remove_player(player.)
        }
    }

    async fn send_to_players(&mut self, msg: GamePlayerMessage) {
        for player in self.players.values_mut() {
            let result = player.addr.send(msg.clone()).await;
            if result.is_err() {
                error!(
                    "{}: Failed to send message to player {}",
                    self.id, player.info.id
                );
                // TODO: self.remove_player(player.)
            }
        }
    }

    async fn add_player(
        &mut self,
        player_addr: PlayerAddr,
        player: PlayerInformation,
        ctx: &Context<Self>,
    ) {
        let player_id = player.id.clone();
        let game_player = GamePlayer::new(player_addr, player);
        let game_player_state = game_player.to_state();
        self.players.insert(player_id.clone(), game_player.clone());

        if let Err(err) = self
            .repo
            .append_room_event(
                &self.id,
                RoomEvent::PlayerJoined {
                    player_id: player_id.to_string(),
                },
            )
            .await
        {
            warn!("Suppressed database error: {}", err);
        }

        // welcome
        let players_state = self.players.values().map(|p| p.to_state()).collect();
        self.send_to_player(
            &game_player,
            GamePlayerMessage::Welcome(self.id.clone(), ctx.addr(), self.to_state(), players_state),
        )
        .await;

        // introduce
        self.send_to_players(GamePlayerMessage::PlayerJoined(game_player_state.clone()))
            .await;
    }

    async fn remove_player(&mut self, player_id: &str, ctx: &mut Context<Self>) {
        self.players.remove(player_id);

        if let Err(err) = self
            .repo
            .append_room_event(
                &self.id,
                RoomEvent::PlayerLeaved {
                    player_id: player_id.to_string(),
                },
            )
            .await
        {
            warn!("Suppressed database error: {}", err);
        }

        // announce
        self.send_to_players(GamePlayerMessage::PlayerLeft(player_id.to_string()))
            .await;
        self.update_state_and_send().await;

        if self.players.is_empty() {
            info!("{}: room is now empty", self.id);
            <Self as Actor>::Context::spawn(delayed_message(
                ctx.addr(),
                RoomMessage::CloseWhenEmpty,
                Duration::from_secs(60 * 5),
            ));
        }
    }

    async fn set_vote(&mut self, player_id: &str, vote: Option<String>) {
        if self.open {
            warn!(
                "{}: Discared vote of {} because cards are open",
                self.id, player_id
            );
            return;
        }

        if let Some(player) = self.players.get_mut(player_id) {
            if player.info.voter {
                player.vote = vote;
            } else {
                warn!("{}: Non-voter {} voted", self.id, player_id);
                return;
            }
        } else {
            warn!(
                "{}: Failed to set vote for non-existing player {}",
                self.id, player_id
            );
            return;
        }

        self.update_state();
        self.send_game_state().await;
    }

    fn update_state(&mut self) -> bool {
        let mut change = false;

        let all_voted = self
            .players
            .values()
            .all(|player| player.vote.is_some() || !player.info.voter);
        if all_voted {
            // at least 2 voter must exist to make sense
            let voters = self
                .players
                .values()
                .filter(|player| player.info.voter)
                .count();
            if voters > 1 {
                self.open = true;
                change = true
            }
        }

        change
    }

    async fn update_state_and_send(&mut self) {
        if self.update_state() {
            self.send_game_state().await;
        }
    }

    async fn send_game_state(&mut self) {
        self.send_to_players(GamePlayerMessage::GameStateChanged(self.to_state()))
            .await;
    }

    async fn force_open(&mut self) {
        if !self.open {
            self.open = true;
            self.send_game_state().await;
        }
    }

    async fn restart(&mut self) {
        self.open = false;
        for player in self.players.values_mut() {
            player.vote = None;
        }
        self.send_game_state().await;
    }

    async fn update_player(&mut self, id: &str, name: Option<String>, voter: bool) {
        if let Some(player) = self.players.get_mut(id) {
            player.info.voter = voter;
            player.info.name = name;

            let state = player.to_state();
            self.send_to_players(GamePlayerMessage::PlayerChanged(state))
                .await;
            self.update_state_and_send().await;
        } else {
            warn!("{}: Ignoring update on unknown player {}", self.id, id);
        }
    }

    fn to_state(&self) -> GameState {
        GameState {
            deck: self.deck.clone(),
            open: self.open,
            votes: self
                .players
                .values()
                .filter(|p| p.info.voter)
                .map(|p| {
                    let vote = if self.open {
                        p.vote.clone()
                    } else {
                        p.vote.as_ref().map(|_vote| "�".to_string())
                    };

                    (p.info.id.clone(), vote)
                })
                .collect(),
        }
    }
}

pub type RoomAddr = Addr<RoomMessage>;

#[async_trait::async_trait]
impl Actor for Room {
    type Message = RoomMessage;
    type Context = Context<Self>;

    async fn on_message(&mut self, msg: Self::Message, ctx: &mut Context<Self>) {
        match msg {
            RoomMessage::JoinRequest(player_addr, player) => {
                self.add_player(player_addr, player, ctx).await
            }
            RoomMessage::PlayerLeft(player) => self.remove_player(&player, ctx).await,
            RoomMessage::PlayerVoted(player_id, vote) => self.set_vote(&player_id, vote).await,
            RoomMessage::ForceOpen => self.force_open().await,
            RoomMessage::Restart => self.restart().await,
            RoomMessage::UpdatePlayer { id, name, voter } => {
                self.update_player(&id, name, voter).await
            }
            RoomMessage::Close => {
                info!("{}: Forced close", self.id);
                ctx.force_quit()
            }
            RoomMessage::CloseWhenEmpty => {
                if self.players.is_empty() {
                    info!("{}: closed because it's empty", self.id);
                    ctx.force_quit()
                }
            }
        }
    }

    async fn setup(&mut self, ctx: &mut Context<Self>) {
        let players_state: Vec<PlayerState> = self.players.values().map(|p| p.to_state()).collect();
        let game_state = self.to_state();

        self.send_to_players(GamePlayerMessage::Welcome(
            self.id.clone(),
            ctx.addr(),
            game_state.clone(),
            players_state.clone(),
        ))
        .await;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::mpsc;

    use crate::ports::{DbResult, RoomRepository};
    use crate::room::GamePlayerMessage::GameStateChanged;
    use crate::room::RoomMessage::*;

    use super::*;

    struct RoomTester {
        players: Vec<mpsc::Receiver<GamePlayerMessage>>,
        room_addr: Addr<RoomMessage>,
    }

    struct FakeRoomRepository;

    #[async_trait::async_trait]
    impl RoomRepository for FakeRoomRepository {
        async fn append_room_event(&self, _id: &str, _evt: RoomEvent) -> DbResult<()> {
            Ok(())
        }

        async fn get_room_events(&self, _id: &str) -> DbResult<Vec<RoomEvent>> {
            Ok(vec![])
        }
    }

    impl RoomTester {
        pub async fn new_room() -> Self {
            let repo: RoomRepositoryRef = Arc::new(FakeRoomRepository);
            let room = Room::new("TEST-ROOM", "TEST-DECK".to_string(), repo).await;
            let room_addr = room.unwrap().start();
            Self {
                players: vec![],
                room_addr,
            }
        }

        pub async fn send(&self, msg: RoomMessage) {
            self.room_addr.send(msg).await.unwrap();
        }

        pub async fn join_player(&mut self, id: &str, voter: bool) {
            let (player_addr, rx, player_info) = Self::create_player(id, voter);
            self.send(JoinRequest(player_addr, player_info)).await;
            self.players.push(rx);
        }

        pub async fn kick_player(&mut self, id: &str) {
            self.send(PlayerLeft(id.to_string())).await;
        }

        pub async fn force_open(&self) {
            self.send(ForceOpen).await;
        }

        pub async fn send_vote(&self, player: &str, vote: Option<&str>) {
            self.send(PlayerVoted(player.to_string(), vote.map(|v| v.to_string())))
                .await;
        }

        pub async fn close(self) -> Vec<mpsc::Receiver<GamePlayerMessage>> {
            self.send(Close).await;
            self.room_addr.closed().await;
            self.players
        }

        fn create_player(
            id: &str,
            voter: bool,
        ) -> (
            PlayerAddr,
            mpsc::Receiver<GamePlayerMessage>,
            PlayerInformation,
        ) {
            let (player_addr, rx) = mpsc::channel(16);
            let info = PlayerInformation {
                id: id.to_string(),
                voter,
                name: None,
            };

            (player_addr, rx, info)
        }
    }

    #[tokio::test]
    async fn check_open_when_all_voted() {
        let mut tester = RoomTester::new_room().await;
        tester.join_player("1", true).await;
        tester.join_player("2", true).await;

        // ACT
        tester.send_vote("1", Some("VOTE")).await;
        tester.send_vote("2", Some("VOTE")).await;
        let mut rxs = tester.close().await;

        // ASSERT
        test_for_message!(rxs[0], GameStateChanged(state) if state.open == true);
    }

    #[tokio::test]
    async fn check_open_with_non_voter() {
        let mut tester = RoomTester::new_room().await;
        tester.join_player("1", true).await;
        tester.join_player("2", true).await;
        tester.join_player("3", false).await;

        // ACT
        tester.send_vote("1", Some("VOTE")).await;
        tester.send_vote("2", Some("VOTE")).await;
        let mut rxs = tester.close().await;

        // ASSERT
        test_for_message!(rxs[0], GameStateChanged(state) if state.open == true);
    }

    #[tokio::test]
    async fn check_open_after_player_became_non_voter() {
        let mut tester = RoomTester::new_room().await;
        tester.join_player("1", true).await;
        tester.join_player("2", true).await;
        tester.join_player("3", true).await;

        // ACT
        tester.send_vote("1", Some("VOTE")).await;
        tester.send_vote("3", Some("VOTE")).await;
        tester
            .send(UpdatePlayer {
                id: "2".to_string(),
                voter: false,
                name: None,
            })
            .await;
        let mut rxs = tester.close().await;

        // ASSERT
        test_for_message!(rxs[0], GameStateChanged(state) if state.open == true);
    }

    #[tokio::test]
    async fn check_open_after_player_left() {
        let mut tester = RoomTester::new_room().await;
        tester.join_player("1", true).await;
        tester.join_player("2", true).await;
        tester.join_player("3", true).await;

        // ACT
        tester.send_vote("1", Some("VOTE")).await;
        tester.send_vote("3", Some("VOTE")).await;
        tester.kick_player("2").await;
        let mut rxs = tester.close().await;

        // ASSERT
        test_for_message!(rxs[0], GameStateChanged(state) if state.open == true);
    }

    #[tokio::test]
    async fn check_no_voting_when_closed() {
        let mut tester = RoomTester::new_room().await;
        tester.join_player("r1", true).await;
        tester.join_player("p1", true).await;

        // ACT
        tester.force_open().await;
        tester.send_vote("p1", Some("VOTE")).await;
        let mut rxs = tester.close().await;

        // ASSERT
        assert_no_message!(
            rxs[0], GameStateChanged(ref state) if state.votes.get("p1").cloned().flatten().is_some());
    }
}
