//! # CacheDB
//!
//! In-memory shared cache SQLite database connection for caching.
//! This should be relatively easy to convert to a Redis cache if needed since this is already essentially just a key-value store.
//!
//! Identifiers should be a string following this format: `TYPE_OF_OBJECT:OBJECT_ID`. For pastes this would look like: `paste:{custom_url}`
use super::sql::{self, Database};
use sqlx::Row;

#[derive(Clone)]
pub struct CacheDB {
    pub db: Database<sqlx::SqlitePool>,
}

impl CacheDB {
    pub async fn new() -> CacheDB {
        return CacheDB {
            db: sql::create_db_sqlite("sqlite://:memory:?cache=shared").await,
        };
    }

    pub async fn init(&self) {
        // create tables
        let c = &self.db.client;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"CacheObjects\" (
                id VARCHAR(1000000),
                content VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;
    }

    // GET
    /// Get a cache object by its identifier
    ///
    /// # Arguments:
    /// * `id` - `String` of the object's id
    pub async fn get(&self, id: String) -> Option<String> {
        // fetch from database
        let c = &self.db.client;
        let res = sqlx::query("SELECT * FROM \"CacheObjects\" WHERE \"id\" = ?")
            .bind::<&String>(&id)
            .fetch_one(c)
            .await;

        if res.is_err() {
            return Option::None;
        }

        // get content
        let row = res.unwrap();
        let content = row.get::<String, &str>("content");

        // return
        Option::Some(content)
    }

    // SET
    /// Set a cache object by its identifier and content
    ///
    /// # Arguments:
    /// * `id` - `String` of the object's id
    /// * `content` - `String` of the object's content
    pub async fn set(&self, id: String, content: String) -> bool {
        // set
        let c = &self.db.client;
        let res = sqlx::query("INSERT INTO \"CacheObjects\" VALUES (?, ?)")
            .bind::<&String>(&id)
            .bind::<&String>(&content)
            .execute(c)
            .await;

        if res.is_err() {
            return false;
        }

        // return
        true
    }

    /// Update a cache object by its identifier and content
    ///
    /// # Arguments:
    /// * `id` - `String` of the object's id
    /// * `content` - `String` of the object's content
    pub async fn update(&self, id: String, content: String) -> bool {
        // update
        let c = &self.db.client;
        let res = sqlx::query("UPDATE \"CacheObjects\" SET \"content\" = ? WHERE \"id\" = ?")
            .bind::<&String>(&content)
            .bind::<&String>(&id)
            .execute(c)
            .await;

        if res.is_err() {
            return false;
        }

        // return
        true
    }

    /// Remove a cache object by its identifier
    ///
    /// # Arguments:
    /// * `id` - `String` of the object's id
    pub async fn remove(&self, id: String) -> bool {
        // remove
        let c = &self.db.client;
        let res = sqlx::query("DELETE FROM \"CacheObjects\" WHERE \"id\" = ?")
            .bind::<&String>(&id)
            .execute(c)
            .await;

        if res.is_err() {
            return false;
        }

        // return
        true
    }
}
