use dorsal::{utility, StarterDatabase};
use serde::{Deserialize, Serialize};

use crate::db::Result;
use crate::model::DatabaseError;

#[derive(Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Log {
    // selectors
    pub id: String,
    pub logtype: String,
    // dates
    pub timestamp: u128,
    // ...
    pub content: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogIdentifier {
    pub id: String,
}

// database
#[derive(Clone)]
pub struct LogDatabase {
    pub base: StarterDatabase,
}

impl LogDatabase {
    // logs

    // GET
    /// Get a log by its id
    ///
    /// # Arguments:
    /// * `id` - `String` of the log's `id`
    pub async fn get_log_by_id(&self, id: String) -> Result<Log> {
        // check in cache
        let cached = self.base.cachedb.get(format!("log:{}", id)).await;

        if cached.is_some() {
            // ...
            let log = serde_json::from_str::<Log>(cached.unwrap().as_str()).unwrap();

            // return
            return Ok(log);
        }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"cr_logs\" WHERE \"id\" = ?"
        } else {
            "SELECT * FROM \"cr_logs\" WHERE \"id\" = $1"
        };

        let c = &self.base.db.client;
        let row = match sqlx::query(query).bind::<&String>(&id).fetch_one(c).await {
            Ok(r) => self.base.textify_row(r).data,
            Err(_) => return Err(DatabaseError::Other),
        };

        // store in cache
        let log = Log {
            id: row.get("id").unwrap().to_string(),
            logtype: row.get("logtype").unwrap().to_string(),
            timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
            content: row.get("content").unwrap().to_string(),
        };

        self.base
            .cachedb
            .set(
                format!("log:{}", id),
                serde_json::to_string::<Log>(&log).unwrap(),
            )
            .await;

        // return
        return Ok(log);
    }

    // SET
    /// Create a log given its type and content
    ///
    /// # Arguments:
    /// * `logtype` - `String` of the log's `logtype`
    /// * `content` - `String` of the log's `content`
    pub async fn create_log(&self, logtype: String, content: String) -> Result<()> {
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "INSERT INTO \"cr_logs\" VALUES (?, ?, ?, ?)"
        } else {
            "INSERT INTO \"cr_logs\" VALUES ($1, $2, $3, $4)"
        };

        let log_id: String = utility::random_id();

        let c = &self.base.db.client;
        match sqlx::query(query)
            .bind::<&String>(&log_id)
            .bind::<String>(logtype)
            .bind::<String>(utility::unix_epoch_timestamp().to_string())
            .bind::<String>(content)
            .execute(c)
            .await
        {
            Ok(_) => return Ok(()),
            Err(_) => return Err(DatabaseError::Other),
        };
    }

    /// Delete a log given its id
    ///
    /// # Arguments:
    /// * `id` - `String` of the log's `id`
    pub async fn delete_log(&self, id: String) -> Result<()> {
        // make sure log exists
        if let Err(e) = self.get_log_by_id(id.clone()).await {
            return Err(e);
        };

        // update log
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "DELETE FROM \"cr_logs\" WHERE \"id\" = ?"
        } else {
            "DELETE FROM \"cr_logs\" WHERE \"id\" = $1"
        };

        let c = &self.base.db.client;
        match sqlx::query(query).bind::<&String>(&id).execute(c).await {
            Ok(_) => {
                // update cache
                self.base.cachedb.remove(format!("log:{}", id)).await;

                // return
                return Ok(());
            }
            Err(_) => return Err(DatabaseError::Other),
        }
    }
}
