use std::sync::Mutex;

use diesel::connection::DefaultLoadingMode;
use diesel::prelude::*;
use diesel::{Connection, PgConnection};
use diesel_json::Json;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use serde::{Deserialize, Serialize};

use schema::rooms_events;

use crate::ports::RoomRepository;
use crate::room::RoomEvent;

mod schema;

#[derive(Queryable, Selectable)]
#[diesel(table_name = rooms_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SingleRoomEvent {
    pub event_data: Json<DbRoomEvent>,
}

#[derive(Insertable)]
#[diesel(table_name = rooms_events)]
pub struct NewRoomEvent<'a> {
    pub room_id: &'a str,
    pub event_data: Json<DbRoomEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DbRoomEvent {
    Created { id: String, deck: Vec<String> },
    PlayerJoined { player_id: String },
    PlayerLeaved { player_id: String },
}

impl From<RoomEvent> for DbRoomEvent {
    fn from(value: RoomEvent) -> Self {
        match value {
            RoomEvent::Created { id, deck } => DbRoomEvent::Created { id, deck },
            RoomEvent::PlayerJoined { player_id } => DbRoomEvent::PlayerJoined { player_id },
            RoomEvent::PlayerLeaved { player_id } => DbRoomEvent::PlayerLeaved { player_id },
        }
    }
}

impl From<DbRoomEvent> for RoomEvent {
    fn from(value: DbRoomEvent) -> Self {
        match value {
            DbRoomEvent::Created { id, deck } => RoomEvent::Created { id, deck },
            DbRoomEvent::PlayerJoined { player_id } => RoomEvent::PlayerJoined { player_id },
            DbRoomEvent::PlayerLeaved { player_id } => RoomEvent::PlayerLeaved { player_id },
        }
    }
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn establish_connection() -> PgConnection {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut connection = PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("migrations should run successfully");
    connection
}

pub struct DieselRoomRepository {
    conn: Mutex<PgConnection>,
}

impl DieselRoomRepository {
    pub fn new(conn: Mutex<PgConnection>) -> Self {
        Self { conn }
    }
}

impl RoomRepository for DieselRoomRepository {
    fn append_room_event(&self, room_id: &str, event: RoomEvent) {
        let event = NewRoomEvent {
            room_id,
            event_data: Json(event.into()),
        };

        let mut conn = self.conn.lock().unwrap();
        diesel::insert_into(rooms_events::table)
            .values(&event)
            .execute(&mut *conn)
            .expect("New room event not saved");
    }

    fn get_room_events(&self, id: &str) -> Vec<RoomEvent> {
        use schema::rooms_events::dsl::*;

        let mut conn = self.conn.lock().unwrap();
        rooms_events
            .filter(room_id.eq(id))
            .select(SingleRoomEvent::as_select())
            .load_iter::<SingleRoomEvent, DefaultLoadingMode>(&mut *conn)
            .expect("Select failed")
            .map(|evt| evt.map(|evt| evt.event_data.0.into()))
            .collect::<QueryResult<Vec<RoomEvent>>>()
            .expect("Select failed")
    }
}
