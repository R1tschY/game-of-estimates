use std::error::Error;
use std::sync::Arc;

use crate::room::RoomEvent;

#[derive(Debug)]
pub struct DbError(anyhow::Error);

pub type DbResult<T> = Result<T, DbError>;

impl<E> From<E> for DbError
where
    E: Error + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        DbError(error.into())
    }
}

pub struct DatabaseUrl(pub String);

#[async_trait::async_trait]
pub trait DatabaseMigrator {
    async fn migrate(&self) -> DbResult<()>;
}

pub type DatabaseMigratorRef = Arc<dyn DatabaseMigrator + Send + Sync>;

#[async_trait::async_trait]
pub trait RoomRepository {
    async fn append_room_event(&self, id: &str, evt: RoomEvent) -> DbResult<()>;
    async fn get_room_events(&self, id: &str) -> DbResult<Vec<RoomEvent>>;
}

pub type RoomRepositoryRef = Arc<dyn RoomRepository + Send + Sync>;
