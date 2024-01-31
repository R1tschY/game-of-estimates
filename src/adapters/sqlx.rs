use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use sqlx::{PgPool, Row};
use time::OffsetDateTime;

use crate::ports::{DbResult, RoomRepository};
use crate::room::RoomEvent;

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

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn create_pool() -> DbResult<PgPool> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

pub struct DieselRoomRepository {
    pool: PgPool,
}

impl DieselRoomRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RoomRepository for DieselRoomRepository {
    async fn append_room_event(&self, room_id: &str, event: RoomEvent) -> DbResult<()> {
        sqlx::query("INSERT INTO room_events (occurred_at, room_id, event_data) VALUES (?, ?, ?)")
            .bind(OffsetDateTime::now_utc())
            .bind(room_id)
            .bind(Json(DbRoomEvent::from(event)))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_room_events(&self, id: &str) -> DbResult<Vec<RoomEvent>> {
        let mut rows = sqlx::query("SELECT event_data FROM room_events WHERE room_id = ?")
            .bind(id)
            .fetch(&self.pool);

        let mut res = vec![];
        while let Some(row) = rows.try_next().await? {
            let json: Json<DbRoomEvent> = row.get(0);
            res.push(RoomEvent::from(json.0));
        }
        Ok(res)
    }
}
