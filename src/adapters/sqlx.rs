use std::sync::Arc;

use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use sqlx::{PgPool, Row};
use time::OffsetDateTime;

use crate::ports::{
    DatabaseMigrator, DatabaseMigratorRef, DatabaseUrl, DbResult, RoomRepository, RoomRepositoryRef,
};
use crate::room::RoomEvent;

#[derive(Default)]
pub struct SqlxModule;

#[chassis::module]
impl SqlxModule {
    #[chassis(singleton)]
    pub fn provide_pool(database_url: DatabaseUrl) -> PgPool {
        PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy(&database_url.0)
            .expect("Database configuration should be valid")
    }

    #[chassis(singleton)]
    pub fn provide_db_migrator(pool: PgPool) -> DatabaseMigratorRef {
        Arc::new(SqlxMigrator::new(pool))
    }

    #[chassis(singleton)]
    pub fn provide_room_repo(pool: PgPool) -> RoomRepositoryRef {
        Arc::new(SqlxRoomRepository::new(pool))
    }
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

static MIGRATOR: Migrator = sqlx::migrate!();

pub struct SqlxMigrator {
    pool: PgPool,
}

impl SqlxMigrator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl DatabaseMigrator for SqlxMigrator {
    async fn migrate(&self) -> DbResult<()> {
        Ok(MIGRATOR.run(&self.pool).await?)
    }
}

pub struct SqlxRoomRepository {
    pool: PgPool,
}

impl SqlxRoomRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RoomRepository for SqlxRoomRepository {
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
