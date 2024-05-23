use dorsal::{utility, DefaultReturn, StarterDatabase};
use serde::{Deserialize, Serialize};

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
    pub async fn get_log_by_id(&self, id: String) -> DefaultReturn<Option<Log>> {
        // check in cache
        let cached = self.base.cachedb.get(format!("log:{}", id)).await;

        if cached.is_some() {
            // ...
            let log = serde_json::from_str::<Log>(cached.unwrap().as_str()).unwrap();

            // return
            return DefaultReturn {
                success: true,
                message: String::from("Log exists (cache)"),
                payload: Option::Some(log),
            };
        }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"cr_logs\" WHERE \"id\" = ?"
        } else {
            "SELECT * FROM \"cr_logs\" WHERE \"id\" = $1"
        };

        let c = &self.base.db.client;
        let res = sqlx::query(query).bind::<&String>(&id).fetch_one(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Log does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.base.textify_row(row).data;

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
        return DefaultReturn {
            success: true,
            message: String::from("Log exists"),
            payload: Option::Some(log),
        };
    }

    // SET
    /// Create a log given its type and content
    ///
    /// # Arguments:
    /// * `logtype` - `String` of the log's `logtype`
    /// * `content` - `String` of the log's `content`
    pub async fn create_log(
        &self,
        logtype: String,
        content: String,
    ) -> DefaultReturn<Option<String>> {
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "INSERT INTO \"cr_logs\" VALUES (?, ?, ?, ?)"
        } else {
            "INSERT INTO \"cr_logs\" VALUES ($1, $2, $3, $4)"
        };

        let log_id: String = utility::random_id();

        let c = &self.base.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&log_id)
            .bind::<String>(logtype)
            .bind::<String>(utility::unix_epoch_timestamp().to_string())
            .bind::<String>(content)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Log created!"),
            payload: Option::Some(log_id),
        };
    }

    /// Delete a log given its id
    ///
    /// # Arguments:
    /// * `id` - `String` of the log's `id`
    pub async fn delete_log(&self, id: String) -> DefaultReturn<Option<String>> {
        // make sure log exists
        let existing = &self.get_log_by_id(id.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Log does not exist!"),
                payload: Option::None,
            };
        }

        // update log
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "DELETE FROM \"cr_logs\" WHERE \"id\" = ?"
        } else {
            "DELETE FROM \"cr_logs\" WHERE \"id\" = $1"
        };

        let c = &self.base.db.client;
        let res = sqlx::query(query).bind::<&String>(&id).execute(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // update cache
        self.base.cachedb.remove(format!("log:{}", id)).await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Log deleted!"),
            payload: Option::Some(id),
        };
    }
}
