use serde::{Deserialize, Serialize};
use serde_json::Result;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RemoteMessage {
    // upstream
    SetVoter { voter: bool },
    SetName { name: String },

    // downstream
    Hello { id: String },
}

pub struct Participant {
    id: Option<String>,
    tx: mpsc::Sender<RemoteMessage>,
}

impl Participant {
    pub fn new(tx: mpsc::Sender<RemoteMessage>) -> Self {
        Self { id: None, tx }
    }
}

pub struct Room {
    id: String,
}

impl Room {
    pub fn new(id: impl ToString) -> Self {
        Self { id: id.to_string() }
    }
}

pub struct WaitingRoom {
    clients: Vec<Participant>,
}

impl WaitingRoom {
    pub fn new() -> Self {
        Self { clients: vec![] }
    }

    pub fn add_client(&mut self, client: Participant) {
        self.clients.push(client);
    }
}

pub struct GameServer {
    rooms: Vec<Room>,
    waiting_room: WaitingRoom,
    participants: Mutex<Vec<Participant>>,
}

impl GameServer {
    pub fn new() -> Self {
        Self {
            rooms: vec![],
            waiting_room: WaitingRoom::new(),
            participants: Mutex::new(vec![]),
        }
    }

    pub async fn add_participant(&self, participant: Participant) {
        let mut participants = self.participants.lock().await;
        participants.push(participant);
    }
}
