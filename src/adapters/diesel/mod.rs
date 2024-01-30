use diesel::prelude::*;
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use diesel_json::Json;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use serde::{Deserialize, Serialize};

use schema::rooms_events;

use crate::ports::{DbResult, RoomRepository};
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

pub async fn create_pool() -> DbResult<Pool<AsyncPgConnection>> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    AsyncConnectionWrapper::<AsyncPgConnection>::establish(&database_url)?
        .run_pending_migrations(MIGRATIONS)
        .expect("migrations should run successfully");

    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool = Pool::builder(config).build()?;
    Ok(pool)
}

pub struct DieselRoomRepository {
    pool: Pool<AsyncPgConnection>,
}

impl DieselRoomRepository {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RoomRepository for DieselRoomRepository {
    async fn append_room_event(&self, room_id: &str, event: RoomEvent) -> DbResult<()> {
        let event = NewRoomEvent {
            room_id,
            event_data: Json(event.into()),
        };

        let mut conn = self.pool.get().await?;
        diesel::insert_into(rooms_events::table)
            .values(&event)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }

    async fn get_room_events(&self, id: &str) -> DbResult<Vec<RoomEvent>> {
        use schema::rooms_events::dsl::*;

        let mut conn = self.pool.get().await?;
        Ok(rooms_events
            .filter(room_id.eq(id))
            .select(SingleRoomEvent::as_select())
            .load::<SingleRoomEvent>(&mut *conn)
            .await?
            .into_iter()
            .map(|evt| evt.event_data.0.into())
            .collect::<Vec<RoomEvent>>())
    }
}
