//! # BundlesDB
//! Database handler for all database types
use super::{
    cache::CacheStore,
    sql::{self, Database, DatabaseOpts},
};
use sqlx::{Column, Row};

use crate::utility;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Clone)]
pub struct AppData {
    pub db: BundlesDB,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
/// Default API return value
pub struct DefaultReturn<T> {
    pub success: bool,
    pub message: String,
    pub payload: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseReturn {
    pub data: HashMap<String, String>,
}

// Paste and Group require the type of their metadata to be specified so it can be converted if needed
#[derive(Debug, Default, PartialEq, sqlx::FromRow, Clone, Serialize, Deserialize)]
pub struct Paste<M> {
    // selectors
    pub custom_url: String,
    pub id: String,
    pub group_name: String,
    // passwords
    pub edit_password: String,
    // dates
    pub pub_date: u128,
    pub edit_date: u128,
    // ...
    pub content: String,
    pub content_html: String, // rendered paste content
    //                           storing the rendered content in the database will save like 100ms when loading pastes!
    // ...
    pub metadata: M,
    pub views: usize,
}

#[derive(Debug, Default, sqlx::FromRow, Clone, Serialize, Deserialize, PartialEq)]
pub struct PasteIdentifier {
    pub custom_url: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasteMetadata {
    pub owner: String,
    pub private_source: String,
    // optionals
    pub title: Option<String>,
    pub description: Option<String>,
    pub favicon: Option<String>,
    pub embed_color: Option<String>,
    pub view_password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A paste content structure containing an array of [files](AtomicPasteFSFile)
pub struct AtomicPaste {
    // atomic pastes are a plain JSON file system storing HTML, CSS, and JS files only
    // they have the least amount of boilerplate for rendering!
    pub _is_atomic: bool, // this must exist so we know a paste's content is for an atomic paste
    pub files: Vec<AtomicPasteFSFile>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
/// A structure representing a single text file
pub struct AtomicPasteFSFile {
    // store only the bare minimum for the required file types
    pub path: String,
    pub content: String,
}

#[derive(Default, PartialEq, sqlx::FromRow, Clone, Serialize, Deserialize)]
pub struct Group<M> {
    // selectors
    pub name: String,
    // passwords
    pub submit_password: String,
    // ...
    pub metadata: M, // JSON Object
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GroupMetadata {
    pub owner: String, // username of owner
}

#[derive(Default, PartialEq, sqlx::FromRow, Clone, Serialize, Deserialize)]
/// A user object
pub struct UserState {
    // selectors
    pub username: String,
    pub id_hashed: String, // users use their UNHASHED id to login, it is used as their session id too!
    //                        the hashed id is the only id that should ever be public!
    pub role: String,
    // dates
    pub timestamp: u128,
}

#[derive(Default, PartialEq, sqlx::FromRow, Clone, Serialize, Deserialize)]
pub struct Log {
    // selectors
    pub id: String,
    pub logtype: String,
    // dates
    pub timestamp: u128,
    // ...
    pub content: String,
}

#[derive(Debug, Default, sqlx::FromRow, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogIdentifier {
    pub id: String,
}

#[derive(Default, PartialEq, sqlx::FromRow, Clone, Serialize, Deserialize)]
pub struct Board<M> {
    // selectors
    pub name: String,
    // dates
    pub timestamp: u128,
    // ...
    pub metadata: M,
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardMetadata {
    pub owner: String,                      // username of owner
    pub is_private: String, // if the homepage of the board is shown to other users (not owner)
    pub allow_anonymous: Option<String>, // if anonymous users can post
    pub allow_open_posting: Option<String>, // if all users can post on the board (not just owner)
    pub about: Option<String>, // welcome message
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardPostLog {
    pub author: String, // username of owner
    pub content: String,
    pub content_html: String,
    pub board: String, // name of board the post is located in
    pub is_hidden: bool,
    pub reply: Option<String>,  // the ID of the post we're replying to
    pub pinned: Option<bool>,   // pin the post to the top of the board feed
    pub replies: Option<usize>, // not really managed in the log, just used to show the number of replies this post has
}

#[derive(Debug, Default, sqlx::FromRow, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardIdentifier {
    pub name: String,
}

// ...
#[derive(Clone)]
#[cfg(feature = "postgres")]
pub struct BundlesDB {
    pub db: Database<sqlx::PgPool>,
    pub options: DatabaseOpts,
}

#[derive(Clone)]
#[cfg(feature = "mysql")]
pub struct BundlesDB {
    pub db: Database<sqlx::MySqlPool>,
    pub options: DatabaseOpts,
}

#[derive(Clone)]
#[cfg(feature = "sqlite")]
pub struct BundlesDB {
    pub db: Database<sqlx::SqlitePool>,
    pub options: DatabaseOpts,
}

static PASTE_CACHE: Lazy<Mutex<CacheStore<Paste<String>>>> =
    Lazy::new(|| Mutex::new(CacheStore::new()));
static AUTH_CACHE: Lazy<Mutex<CacheStore<UserState>>> = Lazy::new(|| Mutex::new(CacheStore::new()));

impl BundlesDB {
    pub async fn new(options: DatabaseOpts) -> BundlesDB {
        return BundlesDB {
            db: sql::create_db(options.clone()).await,
            options,
        };
    }

    pub async fn init(&self) {
        // ...

        // create tables
        let c = &self.db.client;
        // MAX = 1000000
        // we're just using the same max length for everything because lengths are checked before being sent to db

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Pastes\" (
                custom_url VARCHAR(1000000),
                id VARCHAR(1000000),
                group_name VARCHAR(1000000),
                edit_password VARCHAR(1000000),
                pub_date VARCHAR(1000000),
                edit_date VARCHAR(1000000),
                content VARCHAR(1000000),
                content_html VARCHAR(1000000),
                metadata VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Groups\" (
                name VARCHAR(1000000),
                submit_password VARCHAR(1000000),
                metadata VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Users\" (
                username VARCHAR(1000000),
                id_hashed VARCHAR(1000000),
                role VARCHAR(1000000),
                timestamp VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Logs\" (
                id VARCHAR(1000000),
                logtype VARCHAR(1000000),
                timestamp  VARCHAR(1000000),
                content VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Boards\" (
                name VARCHAR(1000000),
                timestamp VARCHAR(1000000),
                metadata VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;
    }

    #[cfg(feature = "sqlite")]
    fn textify_row(&self, row: sqlx::sqlite::SqliteRow) -> DatabaseReturn {
        // get all columns
        let columns = row.columns();

        // create output
        let mut out: HashMap<String, String> = HashMap::new();

        for column in columns {
            let value = row.get(column.name());
            out.insert(column.name().to_string(), value);
        }

        // return
        return DatabaseReturn { data: out };
    }

    #[cfg(feature = "postgres")]
    fn textify_row(&self, row: sqlx::postgres::PgRow) -> DatabaseReturn {
        // get all columns
        let columns = row.columns();

        // create output
        let mut out: HashMap<String, String> = HashMap::new();

        for column in columns {
            let value = row.get(column.name());
            out.insert(column.name().to_string(), value);
        }

        // return
        return DatabaseReturn { data: out };
    }

    #[cfg(feature = "mysql")]
    fn textify_row(&self, row: sqlx::mysql::MySqlRow) -> DatabaseReturn {
        // get all columns
        let columns = row.columns();

        // create output
        let mut out: HashMap<String, String> = HashMap::new();

        for column in columns {
            let value = row.try_get::<Vec<u8>, _>(column.name());

            if value.is_ok() {
                // returned bytes instead of text :(
                // we're going to convert this to a string and then add it to the output!
                out.insert(
                    column.name().to_string(),
                    std::str::from_utf8(value.unwrap().as_slice())
                        .unwrap()
                        .to_string(),
                );
            } else {
                // already text
                let value = row.get(column.name());
                out.insert(column.name().to_string(), value);
            }
        }

        // return
        return DatabaseReturn { data: out };
    }

    // users

    // GET
    /// Get a user by their hashed ID
    ///
    /// # Arguments:
    /// * `hashed` - `String` of the user's hashed ID
    pub async fn get_user_by_hashed(&self, hashed: String) -> DefaultReturn<Option<UserState>> {
        // ...
        if (&self.options.cache_enabled.is_none())
            | (&self.options.cache_enabled.as_ref().unwrap() != &"false")
        {
            let paste_cache = PASTE_CACHE.lock().unwrap();
            let auth_cache = AUTH_CACHE.lock().unwrap();

            let exists_in_cache = paste_cache.load(&hashed).is_some();

            if exists_in_cache == true {
                // get views
                // if allow_cache is true, `selector` should ALWAYS be the custom_url since the cache stores by that, not ID
                let user = auth_cache.load(&hashed).unwrap();

                // return
                return DefaultReturn {
                    success: true,
                    message: String::from("User exists"),
                    payload: Option::Some(UserState {
                        username: user.username.to_string(),
                        id_hashed: user.id_hashed.to_string(),
                        role: user.role.to_string(),
                        timestamp: user.timestamp,
                    }),
                };
            }
        }

        // fetch from database
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Users\" WHERE \"id_hashed\" = ?"
        } else {
            "SELECT * FROM \"Users\" WHERE \"id_hashed\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&hashed)
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("User does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.textify_row(row).data;

        // store in cache
        let user = UserState {
            username: row.get("username").unwrap().to_string(),
            id_hashed: row.get("id_hashed").unwrap().to_string(),
            role: row.get("role").unwrap().to_string(),
            timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
        };

        if (&self.options.cache_enabled.is_none())
            | (&self.options.cache_enabled.as_ref().unwrap() != &"false")
        {
            let mut auth_cache = AUTH_CACHE.lock().unwrap();
            auth_cache.store(row.get("id_hashed").unwrap().to_string(), user.clone());
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("User exists"),
            payload: Option::Some(user),
        };
    }

    /// Get a user by their username
    ///
    /// # Arguments:
    /// * `username` - `String` of the user's username
    pub async fn get_user_by_username(&self, username: String) -> DefaultReturn<Option<UserState>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Users\" WHERE \"username\" = ?"
        } else {
            "SELECT * FROM \"Users\" WHERE \"username\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&username)
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("User does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.textify_row(row).data;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("User exists"),
            payload: Option::Some(UserState {
                username: row.get("username").unwrap().to_string(),
                id_hashed: row.get("id_hashed").unwrap().to_string(),
                role: row.get("role").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
            }),
        };
    }

    // SET
    /// Create a new user given their username. Returns their hashed ID
    ///
    /// # Arguments:
    /// * `username` - `String` of the user's `username`
    pub async fn create_user(&self, username: String) -> DefaultReturn<Option<String>> {
        // make sure user doesn't already exists
        let existing = &self.get_user_by_username(username.clone()).await;
        if existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("User already exists!"),
                payload: Option::None,
            };
        }

        // check username
        let regex = regex::RegexBuilder::new("^[\\w\\_\\-\\.\\!]+$")
            .multi_line(true)
            .build()
            .unwrap();

        if regex.captures(&username).iter().len() < 1 {
            return DefaultReturn {
                success: false,
                message: String::from("Username is invalid"),
                payload: Option::None,
            };
        }

        if (username.len() < 2) | (username.len() > 500) {
            return DefaultReturn {
                success: false,
                message: String::from("Username is invalid"),
                payload: Option::None,
            };
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "INSERT INTO \"Users\" VALUES (?, ?, ?, ?)"
        } else {
            "INSERT INTO \"Users\" VALUES ($1, $2, $3, $4)"
        };

        let user_id_unhashed: String = utility::uuid();
        let user_id_hashed: String = utility::hash(user_id_unhashed.clone());
        let timestamp = utility::unix_epoch_timestamp().to_string();

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&username)
            .bind::<&String>(&user_id_hashed)
            .bind::<&String>(&String::from("member")) // default role
            .bind::<&String>(&timestamp)
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
            message: user_id_unhashed,
            payload: Option::Some(user_id_hashed),
        };
    }

    // logs

    // GET
    /// Get a log by its id
    ///
    /// # Arguments:
    /// * `id` - `String` of the log's `id`
    pub async fn get_log_by_id(&self, id: String) -> DefaultReturn<Option<Log>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"id\" = ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"id\" = $1"
        };

        let c = &self.db.client;
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
        let row = self.textify_row(row).data;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste exists"),
            payload: Option::Some(Log {
                id: row.get("id").unwrap().to_string(),
                logtype: row.get("logtype").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                content: row.get("content").unwrap().to_string(),
            }),
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
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "INSERT INTO \"Logs\" VALUES (?, ?, ?, ?)"
        } else {
            "INSERT INTO \"Logs\" VALUES ($1, $2, $3, $4)"
        };

        let log_id: String = utility::random_id();

        let c = &self.db.client;
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

    /// Edit a log given its ID
    ///
    /// # Arguments:
    /// * `id` - `String` of the log's `id`
    /// * `content` - `String` of the log's new content
    pub async fn edit_log(&self, id: String, content: String) -> DefaultReturn<Option<String>> {
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
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "UPDATE \"Logs\" SET \"content\" = ? WHERE \"id\" = ?"
        } else {
            "UPDATE \"Logs\" SET (\"content\") = ($1) WHERE \"id\" = $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&content)
            .bind::<&String>(&id)
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
            message: String::from("Log updated!"),
            payload: Option::Some(id),
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
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "DELETE FROM \"Logs\" WHERE \"id\" = ?"
        } else {
            "DELETE FROM \"Logs\" WHERE \"id\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind::<&String>(&id).execute(c).await;

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
            message: String::from("Log deleted!"),
            payload: Option::Some(id),
        };
    }

    // pastes

    /// Count the `view_paste` logs for a specific [`Paste`]
    async fn count_paste_views(&self, custom_url: String) -> usize {
        let c = &self.db.client;

        // count views
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT \"ID\" FROM \"Logs\" WHERE \"logtype\" = 'view_paste' AND \"content\" LIKE ?"
        } else {
            "SELECT \"ID\" FROM \"Logs\" WHERE \"logtype\" = 'view_paste' AND \"content\" LIKE $1"
        };

        let views_res = sqlx::query(query)
            .bind::<&String>(&format!("{}::%", &custom_url))
            .fetch_all(c)
            .await;

        if views_res.is_err() {
            return 0;
        }

        return views_res.unwrap().len();
    }

    /// Build a [`Paste`] query with information about it
    async fn build_result_from_query(
        &self,
        query: &str,
        selector: &str,
        allow_cache: bool,
    ) -> DefaultReturn<Option<Paste<String>>> {
        // ...
        if (&self.options.cache_enabled.is_none())
            | (&self.options.cache_enabled.as_ref().unwrap() != &"false")
        {
            let paste_cache = PASTE_CACHE.lock().unwrap();
            let exists_in_cache = paste_cache.load(selector).is_some();

            if (exists_in_cache == true) && allow_cache {
                // get views
                // if allow_cache is true, `selector` should ALWAYS be the custom_url since the cache stores by that, not ID
                let views = &self.count_paste_views(selector.to_owned()).await;
                let paste = paste_cache.load(selector).unwrap();

                // return
                return DefaultReturn {
                    success: true,
                    message: String::from("Paste exists (cache)"),
                    payload: Option::Some(Paste {
                        custom_url: paste.custom_url.to_string(),
                        id: paste.id.to_string(),
                        group_name: paste.group_name.to_string(),
                        edit_password: paste.edit_password.to_string(),
                        pub_date: paste.pub_date,
                        edit_date: paste.edit_date,
                        content: paste.content.to_string(),
                        content_html: paste.content_html.to_string(),
                        metadata: paste.metadata.to_string(),
                        views: views.to_owned(),
                    }),
                };
            }
        }

        // fetch from db
        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&selector.to_lowercase())
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Paste does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.textify_row(row).data;

        // get views
        let views = &self
            .count_paste_views(row.get("custom_url").unwrap().to_owned())
            .await;

        // add to cache
        let paste = Paste {
            custom_url: row.get("custom_url").unwrap().to_string(),
            id: row.get("id").unwrap().to_string(),
            group_name: row.get("group_name").unwrap().to_string(),
            edit_password: row.get("edit_password").unwrap().to_string(),
            pub_date: row.get("pub_date").unwrap().parse::<u128>().unwrap(),
            edit_date: row.get("edit_date").unwrap().parse::<u128>().unwrap(),
            content: row.get("content").unwrap().to_string(),
            content_html: row.get("content_html").unwrap().to_string(),
            metadata: row.get("metadata").unwrap().to_string(),
            views: views.to_owned(),
        };

        if allow_cache
            && (&self.options.cache_enabled.is_none())
                | (&self.options.cache_enabled.as_ref().unwrap() != &"false")
        {
            let mut paste_cache = PASTE_CACHE.lock().unwrap();
            paste_cache.store(row.get("custom_url").unwrap().to_string(), paste.clone());
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste exists (new)"),
            payload: Option::Some(paste),
        };
    }

    // GET
    /// Get a [`Paste`] given its `custom_url`
    ///
    /// # Arguments:
    /// * `url` - `String` of the paste's `custom_url`
    pub async fn get_paste_by_url(&self, url: String) -> DefaultReturn<Option<Paste<String>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Pastes\" WHERE \"custom_url\" = ?"
        } else {
            "SELECT * FROM \"Pastes\" WHERE \"custom_url\" = $1"
        };

        return self.build_result_from_query(query, &url, true).await;
    }

    /// Get a [`Paste`] given its `id`
    ///
    /// # Arguments:
    /// * `id` - `String` of the paste's `id`
    pub async fn get_paste_by_id(&self, id: String) -> DefaultReturn<Option<Paste<String>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Pastes\" WHERE \"id\" = ?"
        } else {
            "SELECT * FROM \"Pastes\" WHERE \"id\" = $1"
        };

        return self.build_result_from_query(query, &id, false).await;
    }

    /// Get all [pastes](Paste) owned by a specific user
    ///
    /// # Arguments:
    /// * `owner` - `String` of the owner's `username`
    pub async fn get_pastes_by_owner(
        &self,
        owner: String,
    ) -> DefaultReturn<Option<Vec<PasteIdentifier>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Pastes\" WHERE \"metadata\" LIKE ?"
        } else {
            "SELECT * FROM \"Pastes\" WHERE \"metadata\" LIKE $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"owner\":\"{}\"%", &owner))
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // build res
        let mut full_res: Vec<PasteIdentifier> = Vec::new();

        for row in res.unwrap() {
            let row = self.textify_row(row).data;
            full_res.push(PasteIdentifier {
                custom_url: row.get("custom_url").unwrap().to_string(),
                id: row.get("id").unwrap().to_string(),
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: owner,
            payload: Option::Some(full_res),
        };
    }

    /// Get all atomic [pastes](Paste) owned by a specific user
    ///
    /// # Arguments:
    /// * `owner` - `String` of the owner's `username`
    pub async fn get_atomic_pastes_by_owner(
        &self,
        owner: String,
    ) -> DefaultReturn<Option<Vec<PasteIdentifier>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Pastes\" WHERE \"metadata\" LIKE ? AND \"content\" LIKE ?"
        } else {
            "SELECT * FROM \"Pastes\" WHERE \"metadata\" LIKE $1 AND \"content\" LIKE $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"owner\":\"{}\"%", &owner))
            .bind("%\"_is_atomic\":true%")
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // build res
        let mut full_res: Vec<PasteIdentifier> = Vec::new();

        for row in res.unwrap() {
            let row = self.textify_row(row).data;
            full_res.push(PasteIdentifier {
                custom_url: row.get("custom_url").unwrap().to_string(),
                id: row.get("id").unwrap().to_string(),
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: owner,
            payload: Option::Some(full_res),
        };
    }

    // SET
    /// Create a new [`Paste`] given various properties
    ///
    /// # Arguments:
    /// * `props` - [`Paste<String>`](Paste)
    /// * `as_user` - The ID of the user creating the paste
    pub async fn create_paste(
        &self,
        props: &mut Paste<String>,
        as_user: Option<String>, // id of paste owner
    ) -> DefaultReturn<Option<Paste<String>>> {
        let p: &mut Paste<String> = props; // borrowed props

        // create default metadata
        let metadata: PasteMetadata = PasteMetadata {
            owner: if as_user.is_some() {
                as_user.clone().unwrap()
            } else {
                String::new()
            },
            private_source: String::from("off"),
            // optionals
            title: Option::Some(String::new()),
            description: Option::Some(String::new()),
            favicon: Option::None,
            embed_color: Option::Some(String::from("#ff9999")),
            view_password: Option::None,
        };

        // check values

        // (check empty)
        if p.custom_url.is_empty() {
            p.custom_url = utility::random_id().chars().take(10).collect();
        }

        if p.edit_password.is_empty() {
            p.edit_password = utility::random_id().chars().take(10).collect();
        }

        // (check length)
        if (p.custom_url.len() < 2) | (p.custom_url.len() > 500) {
            return DefaultReturn {
                success: false,
                message: String::from("Custom URL is invalid"),
                payload: Option::None,
            };
        }

        if !p.group_name.is_empty() && (p.group_name.len() < 2) | (p.group_name.len() > 500) {
            return DefaultReturn {
                success: false,
                message: String::from("Group Name is invalid"),
                payload: Option::None,
            };
        }

        // check content
        if (p.content.len() < 1) | (p.content.len() > 400_000) {
            return DefaultReturn {
                success: false,
                message: String::from("Content is invalid"),
                payload: Option::None,
            };
        }

        // (characters used)
        let regex = regex::RegexBuilder::new("^[\\w\\_\\-\\.\\!]+$")
            .multi_line(true)
            .build()
            .unwrap();

        if regex.captures(&p.custom_url).iter().len() < 1 {
            return DefaultReturn {
                success: false,
                message: String::from("Custom URL is invalid"),
                payload: Option::None,
            };
        }

        // if we're trying to create a paste in a group, make sure the group exists
        // (create it if it doesn't)
        if !p.group_name.is_empty() {
            let n = &p.group_name;
            let e = &p.edit_password;
            let o = &p.custom_url;

            let existing_group = self.get_group_by_name(n.to_string()).await;

            if !existing_group.success {
                let res = self
                    .create_group(Group {
                        name: n.to_string(),
                        submit_password: e.to_string(), // groups will have the same password as their first paste
                        metadata: GroupMetadata {
                            owner: metadata.clone().owner,
                        },
                    })
                    .await;

                if !res.success {
                    return DefaultReturn {
                        success: false,
                        message: res.message,
                        payload: Option::None,
                    };
                }
            } else {
                // check group password
                if utility::hash(e.to_string()) != existing_group.payload.unwrap().submit_password {
                    return DefaultReturn {
                        success: false,
                        message: String::from("The paste edit password must match the group submit password during creation."),
                        payload: Option::None,
                    };
                }
            }

            // append to custom_url
            p.custom_url = format!("{}/{}", n, o);
        }

        // make sure paste does not exist
        let existing: DefaultReturn<Option<Paste<String>>> =
            self.get_paste_by_url(p.custom_url.to_owned()).await;

        if existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Paste already exists!"),
                payload: Option::None,
            };
        }

        // create paste
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "INSERT INTO \"Pastes\" VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        } else {
            "INSERT INTO \"Pastes\" VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        };

        let c = &self.db.client;
        let p: &mut Paste<String> = &mut props.clone();
        p.id = utility::random_id();

        let edit_password = &p.edit_password;
        let edit_password_hash = utility::hash(edit_password.to_string());

        let edit_date = &p.edit_date;
        let pub_date = &p.pub_date;

        let res = sqlx::query(query)
            .bind::<&String>(&p.custom_url)
            .bind::<&String>(&p.id)
            .bind::<&String>(&p.group_name)
            .bind::<&String>(&edit_password_hash)
            .bind::<&String>(&pub_date.to_string())
            .bind::<&String>(&edit_date.to_string())
            .bind::<&String>(&p.content)
            .bind::<&String>(&p.content_html)
            .bind::<&String>(&serde_json::to_string(&metadata).unwrap())
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: res.err().unwrap().to_string(),
                payload: Option::None,
            };
        }

        // return
        let pass = &p.edit_password;
        return DefaultReturn {
            success: true,
            message: pass.to_string(),
            payload: Option::Some(p.to_owned()),
        };
    }

    /// Edit an existing [`Paste`] given its `custom_url`
    pub async fn edit_paste_by_url(
        &self,
        url: String,
        content: String,
        edit_password: String,
        new_url: Option<String>,
        new_edit_password: Option<String>,
        edit_as: Option<String>, // username of account that is editing this paste
    ) -> DefaultReturn<Option<String>> {
        // make sure paste exists
        let existing = &self.get_paste_by_url(url.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Paste does not exist!"),
                payload: Option::None,
            };
        }

        // (parse metadata from existing)
        let existing_metadata =
            serde_json::from_str::<PasteMetadata>(&existing.payload.as_ref().unwrap().metadata);

        // verify password
        // if password hash doesn't match AND edit_as is none OR edit_as != existing_metadata's owner value
        let paste = &existing.payload.clone().unwrap();

        let skip_password_check =
            edit_as.is_some() && edit_as.unwrap() == existing_metadata.unwrap().owner;

        if !skip_password_check && utility::hash(edit_password) != paste.edit_password {
            return DefaultReturn {
                success: false,
                message: String::from("Password invalid"),
                payload: Option::None,
            };
        }

        // ...
        let edit_password_hash = if new_edit_password.is_some() {
            utility::hash(new_edit_password.unwrap())
        } else {
            // get old password
            let edit_password = &paste.edit_password;
            edit_password.to_owned()
        };

        let custom_url = if new_url.is_some() {
            new_url.as_ref().unwrap()
        } else {
            // get old custom url
            &paste.custom_url
        };

        // if we're changing url, make sure this paste doesn't already exist
        if new_url.is_some() {
            let existing = &self.get_paste_by_url(new_url.clone().unwrap()).await;
            if existing.success {
                return DefaultReturn {
                    success: false,
                    message: String::from("A paste with this URL already exists!"),
                    payload: Option::None,
                };
            }
        }

        // update paste
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "UPDATE \"Pastes\" SET \"content\" = ?, \"content_html\" = ?, \"edit_password\" = ?, \"custom_url\" = ?, \"edit_date\" = ? WHERE \"custom_url\" = ?"
        } else {
            "UPDATE \"Pastes\" SET (\"content\", \"content_html\", \"edit_password\", \"custom_url\", \"edit_date\") = ($1, $2, $3, $4, $5) WHERE \"custom_url\" = $6"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&content)
            .bind::<&String>(&crate::markdown::render::parse_markdown(&content))
            .bind::<&String>(&edit_password_hash)
            .bind::<&String>(&custom_url)
            .bind::<&String>(&utility::unix_epoch_timestamp().to_string()) // update edit_date
            .bind::<&String>(&url)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // we're not even going to update the cache, just purge the paste from the cache
        // this also means we don't have to handle any decisions on if the paste custom_url changed or not
        if (&self.options.cache_enabled.is_none())
            | (&self.options.cache_enabled.as_ref().unwrap() != &"false")
        {
            let mut paste_cache = PASTE_CACHE.lock().unwrap();
            paste_cache.clear(&custom_url);
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste updated!"),
            payload: Option::Some(custom_url.to_string()),
        };
    }

    /// Update a [`Paste`]'s metadata by its `custom_url`
    pub async fn edit_paste_metadata_by_url(
        &self,
        url: String,
        metadata: PasteMetadata,
        edit_password: String,
        edit_as: Option<String>, // username of account that is editing this paste
    ) -> DefaultReturn<Option<String>> {
        // make sure paste exists
        let existing = &self.get_paste_by_url(url.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Paste does not exist!"),
                payload: Option::None,
            };
        }

        // (parse metadata from existing)
        let existing_metadata =
            serde_json::from_str::<PasteMetadata>(&existing.payload.as_ref().unwrap().metadata);

        // get edit_as user account
        let ua = if edit_as.is_some() {
            Option::Some(
                self.get_user_by_username(edit_as.clone().unwrap())
                    .await
                    .payload,
            )
        } else {
            Option::None
        };

        // verify password
        // if password hash doesn't match AND edit_as is none OR edit_as != existing_metadata's owner value
        let paste = &existing.payload.clone().unwrap();

        // ...skip password check IF the user is the paste owner!
        let skip_password_check = (edit_as.is_some()
            && edit_as.unwrap() == existing_metadata.unwrap().owner)
            // OR if the user has the "staff" role
            | (ua.as_ref().is_some()
                && ua.as_ref().unwrap().is_some()
                && ua.unwrap().unwrap().role == "staff");

        if !skip_password_check && utility::hash(edit_password) != paste.edit_password {
            return DefaultReturn {
                success: false,
                message: String::from("Password invalid"),
                payload: Option::None,
            };
        }

        // update paste
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "UPDATE \"Pastes\" SET \"metadata\" = ? WHERE \"custom_url\" = ?"
        } else {
            "UPDATE \"Pastes\" SET (\"metadata\") = ($1) WHERE \"custom_url\" = $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&serde_json::to_string(&metadata).unwrap())
            .bind::<&String>(&url)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // we're not even going to update the cache, just purge the paste from the cache
        // this also means we don't have to handle any decisions on if the paste custom_url changed or not
        if (&self.options.cache_enabled.is_none())
            | (&self.options.cache_enabled.as_ref().unwrap() != &"false")
        {
            let mut paste_cache = PASTE_CACHE.lock().unwrap();
            paste_cache.clear(&url);
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste updated!"),
            payload: Option::Some(url),
        };
    }

    /// Count a view to a [`Paste`] given its `custom_url`
    ///
    /// # Arguments:
    /// * `view_as` - The username of the account that viewed the paste
    pub async fn add_view_to_url(
        &self,
        url: &String,
        view_as: &String, // username of account that is viewing this paste
    ) -> DefaultReturn<Option<String>> {
        // make sure paste exists
        let existing = &self.get_paste_by_url(url.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Paste does not exist!"),
                payload: Option::None,
            };
        }

        // check for existing view log
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'view_paste' AND \"content\" LIKE ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'view_paste' AND \"content\" LIKE $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("{}::{}", &url, &view_as))
            .fetch_one(c)
            .await;

        if res.is_err() {
            let err = res.err().unwrap();
            let err_message = err.to_string();

            // count view if message says no rows were returned
            if err_message.starts_with("no rows returned") {
                self.create_log(
                    String::from("view_paste"),
                    format!("{}::{}", &url, &view_as),
                )
                .await;

                // return
                return DefaultReturn {
                    success: true,
                    message: String::from("View counted!"),
                    payload: Option::Some(url.to_string()),
                };
            }

            // default error return
            return DefaultReturn {
                success: false,
                message: String::from("Failed to check for existing view!"),
                payload: Option::None,
            };
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("View counted!"),
            payload: Option::Some(url.to_string()),
        };
    }

    /// Delete a [`Paste`] given its `custom_url` and `edit_password`
    pub async fn delete_paste_by_url(
        &self,
        url: String,
        edit_password: String,
        delete_as: Option<String>,
    ) -> DefaultReturn<Option<String>> {
        // make sure paste exists
        let existing = &self.get_paste_by_url(url.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Paste does not exist!"),
                payload: Option::None,
            };
        }

        // (parse metadata from existing)
        let existing_metadata =
            serde_json::from_str::<PasteMetadata>(&existing.payload.as_ref().unwrap().metadata);

        // get edit_as user account
        let ua = if delete_as.is_some() {
            Option::Some(
                self.get_user_by_username(delete_as.clone().unwrap())
                    .await
                    .payload,
            )
        } else {
            Option::None
        };

        // verify password
        let paste = &existing.payload.clone().unwrap();

        // ...skip password check IF the user is the paste owner!
        let skip_password_check = (delete_as.is_some()
                && delete_as.unwrap() == existing_metadata.unwrap().owner)
                // OR if the user has the "staff" role
                | (ua.as_ref().is_some()
                    && ua.as_ref().unwrap().is_some()
                    && ua.unwrap().unwrap().role == "staff");

        if !skip_password_check && utility::hash(edit_password) != paste.edit_password {
            return DefaultReturn {
                success: false,
                message: String::from("Password invalid"),
                payload: Option::None,
            };
        }

        // delete paste
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "DELETE FROM \"Pastes\" WHERE \"custom_url\" = ?"
        } else {
            "DELETE FROM \"Pastes\" WHERE \"custom_url\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind::<&String>(&url).execute(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // delete paste views
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "DELETE FROM \"Logs\" WHERE \"content\" LIKE ?"
        } else {
            "DELETE FROM \"Logs\" WHERE \"content\" LIKE $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("{}::%", &url))
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to delete paste"),
                payload: Option::None,
            };
        }

        // remove from cache
        if (&self.options.cache_enabled.is_none())
            | (&self.options.cache_enabled.as_ref().unwrap() != &"false")
        {
            let mut paste_cache = PASTE_CACHE.lock().unwrap();
            paste_cache.clear(&url);
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste deleted!"),
            payload: Option::Some(url),
        };
    }

    // groups

    // GET
    /// Get a [`Group`] by its name
    ///
    /// # Arguments:
    /// * `url` - group name
    pub async fn get_group_by_name(&self, url: String) -> DefaultReturn<Option<Group<String>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Groups\" WHERE \"name\" = ?"
        } else {
            "SELECT * FROM \"Groups\" WHERE \"name\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind::<&String>(&url).fetch_one(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Group does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Group exists"),
            payload: Option::Some(Group {
                name: row.get("name"),
                submit_password: row.get("submit_password"),
                metadata: row.get("metadata"),
            }),
        };
    }

    // SET
    /// Create a new [`Group`] by its name
    ///
    /// # Arguments:
    /// * `props` - [`Group<GroupMetadata>`](Group)
    pub async fn create_group(&self, props: Group<GroupMetadata>) -> DefaultReturn<Option<String>> {
        let p: &Group<GroupMetadata> = &props; // borrowed props

        // make sure group does not exist
        let existing: DefaultReturn<Option<Group<String>>> =
            self.get_group_by_name(p.name.to_owned()).await;

        if existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Group already exists!"),
                payload: Option::None,
            };
        }

        // create group
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "INSERT INTO \"Groups\" VALUES (?, ?, ?)"
        } else {
            "INSERT INTO \"Groups\" VALUES ($1, $2, $3)"
        };

        let c = &self.db.client;
        let p: &mut Group<GroupMetadata> = &mut props.clone();

        p.submit_password = utility::hash(p.submit_password.clone());
        let res = sqlx::query(query)
            .bind::<&String>(&p.name)
            .bind::<&String>(&p.submit_password)
            .bind::<&String>(&serde_json::to_string(&p.metadata).unwrap())
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: res.err().unwrap().to_string(),
                payload: Option::None,
            };
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste created"),
            payload: Option::Some(p.name.to_string()),
        };
    }

    // boards

    // GET
    /// Get a [`Board`] by its name
    ///
    /// # Arguments:
    /// * `url` - board name
    pub async fn get_board_by_name(&self, url: String) -> DefaultReturn<Option<Board<String>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Boards\" WHERE \"name\" = ?"
        } else {
            "SELECT * FROM \"Boards\" WHERE \"name\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind::<&String>(&url).fetch_one(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Board does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.textify_row(row).data;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Board exists"),
            payload: Option::Some(Board {
                name: row.get("name").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                metadata: row.get("metadata").unwrap().to_string(),
            }),
        };
    }

    /// Get all posts in a [`Board`] by its name
    ///
    /// # Arguments:
    /// * `url` - board name
    /// * `offset` - optional value representing the SQL fetch offset
    pub async fn get_board_posts(
        &self,
        url: String,
        offset: Option<i32>,
    ) -> DefaultReturn<Option<Vec<Log>>> {
        // make sure board exists
        let existing: DefaultReturn<Option<Board<String>>> =
            self.get_board_by_name(url.to_owned()).await;

        if existing.success == false {
            return DefaultReturn {
                success: false,
                message: String::from("Board does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"content\" NOT LIKE '%\"reply\":\"%' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"content\" NOT LIKE '%\"reply\":\"%' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"board\":\"{}\"%", url))
            .bind(if offset.is_some() { offset.unwrap() } else { 0 })
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch posts"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<Log> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;
            output.push(Log {
                id: row.get("id").unwrap().to_string(),
                logtype: row.get("logtype").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                content: row.get("content").unwrap().to_string(),
            });
        }

        let mut true_output: Vec<Log> = Vec::new();
        for mut post in output {
            let mut parsed = serde_json::from_str::<BoardPostLog>(&post.content).unwrap();

            // get replies
            let replies = &self.get_post_replies_limited(post.clone().id, false).await;

            if replies.payload.is_some() {
                parsed.replies = Option::Some(replies.payload.as_ref().unwrap().len());

                // update
                post.content = serde_json::to_string::<BoardPostLog>(&parsed).unwrap();
                true_output.push(post);

                continue;
            }

            continue;
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Successfully fetched posts"),
            payload: Option::Some(true_output),
        };
    }

    /// Get all pinned posts in a [`Board`] by its name
    ///
    /// # Arguments:
    /// * `url` - board name
    pub async fn get_pinned_board_posts(&self, url: String) -> DefaultReturn<Option<Vec<Log>>> {
        // make sure board exists
        let existing: DefaultReturn<Option<Board<String>>> =
            self.get_board_by_name(url.to_owned()).await;

        if existing.success == false {
            return DefaultReturn {
                success: false,
                message: String::from("Board does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"content\" NOT LIKE '%\"reply\":\"%' AND \"content\" LIKE '%\"pinned\":true%' ORDER BY \"timestamp\" DESC LIMIT 50"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"content\" NOT LIKE '%\"reply\":\"%' AND \"content\" LIKE '%\"pinned\":true%' ORDER BY \"timestamp\" DESC LIMIT 50"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"board\":\"{}\"%", url))
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch posts"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<Log> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;
            output.push(Log {
                id: row.get("id").unwrap().to_string(),
                logtype: row.get("logtype").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                content: row.get("content").unwrap().to_string(),
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Successfully fetched posts (pinned)"),
            payload: Option::Some(output),
        };
    }

    /// Get all posts in a [`Board`] by its name that are replying to another [`BoardPostLog`]
    ///
    /// # Arguments:
    /// * `id` - post id
    /// * `run_existing_check` - if we should check that the log exists first
    pub async fn get_post_replies(
        &self,
        id: String,
        run_existing_check: bool,
    ) -> DefaultReturn<Option<Vec<Log>>> {
        // make sure message exists
        if run_existing_check != false {
            let existing: DefaultReturn<Option<Log>> = self.get_log_by_id(id.to_owned()).await;

            if existing.success == false {
                return DefaultReturn {
                    success: false,
                    message: String::from("Post does not exist"),
                    payload: Option::None,
                };
            }
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? ORDER BY \"timestamp\" DESC LIMIT 50"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 ORDER BY \"timestamp\" DESC LIMIT 50"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"reply\":\"{}\"%", id))
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch posts"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<Log> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;
            output.push(Log {
                id: row.get("id").unwrap().to_string(),
                logtype: row.get("logtype").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                content: row.get("content").unwrap().to_string(),
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Successfully fetched posts (replies)"),
            payload: Option::Some(output),
        };
    }

    /// Get all posts in a [`Board`] by its name that are replying to another [`BoardPostLog`] (limited form)
    ///
    /// - only includes post id
    ///
    /// # Arguments:
    /// * `id` - post id
    /// * `run_existing_check` - if we should check that the log exists first
    pub async fn get_post_replies_limited(
        &self,
        id: String,
        run_existing_check: bool,
    ) -> DefaultReturn<Option<Vec<LogIdentifier>>> {
        // make sure message exists
        if run_existing_check != false {
            let existing: DefaultReturn<Option<Log>> = self.get_log_by_id(id.to_owned()).await;

            if existing.success == false {
                return DefaultReturn {
                    success: false,
                    message: String::from("Post does not exist"),
                    payload: Option::None,
                };
            }
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT \"ID\" FROM \"Logs\" WHERE \"content\" LIKE ? ORDER BY \"timestamp\" DESC LIMIT 50"
        } else {
            "SELECT \"ID\" FROM \"Logs\" WHERE \"content\" LIKE $1 ORDER BY \"timestamp\" DESC LIMIT 50"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"reply\":\"{}\"%", id))
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch posts"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<LogIdentifier> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;
            output.push(LogIdentifier {
                id: row.get("id").unwrap_or(&String::new()).to_string(),
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Successfully fetched posts (limited)"),
            payload: Option::Some(output),
        };
    }

    /// Get all [boards](Board) owned by a specific user
    ///
    /// # Arguments:
    /// * `owner` - `String` of the owner's `username`
    pub async fn get_boards_by_owner(
        &self,
        owner: String,
    ) -> DefaultReturn<Option<Vec<BoardIdentifier>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Boards\" WHERE \"metadata\" LIKE ?"
        } else {
            "SELECT * FROM \"Boards\" WHERE \"metadata\" LIKE $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"owner\":\"{}\"%", &owner))
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // build res
        let mut full_res: Vec<BoardIdentifier> = Vec::new();

        for row in res.unwrap() {
            let row = self.textify_row(row).data;
            full_res.push(BoardIdentifier {
                name: row.get("name").unwrap().to_string(),
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: owner,
            payload: Option::Some(full_res),
        };
    }

    /// Get most recent posts from all [`Boards`](Board)
    ///
    /// # Arguments:
    /// * `offset` - optional value representing the SQL fetch offset
    pub async fn fetch_most_recent_posts(
        &self,
        offset: Option<i32>,
    ) -> DefaultReturn<Option<Vec<Log>>> {
        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            // TODO: flexible LIMIT (pagination)
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'board_post' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'board_post' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind(if offset.is_some() { offset.unwrap() } else { 0 })
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch posts"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<Log> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;
            output.push(Log {
                id: row.get("id").unwrap().to_string(),
                logtype: row.get("logtype").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                content: row.get("content").unwrap().to_string(),
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Successfully fetched posts"),
            payload: Option::Some(output),
        };
    }

    // SET
    /// Create a new [`Board`] given various properties
    ///
    /// # Arguments:
    /// * `props` - [`Board<String>`](Board)
    /// * `as_user` - The ID of the user creating the board
    pub async fn create_board(
        &self,
        props: &mut Board<String>,
        as_user: Option<String>, // id of board owner
    ) -> DefaultReturn<Option<Board<String>>> {
        let p: &mut Board<String> = props; // borrowed props

        // create default metadata
        let metadata: BoardMetadata = BoardMetadata {
            owner: as_user.clone().unwrap(),
            is_private: String::from("no"),
            allow_anonymous: Option::Some(String::from("yes")),
            allow_open_posting: Option::Some(String::from("yes")),
            about: Option::None,
        };

        // check values

        // (check length)
        if (p.name.len() < 2) | (p.name.len() > 250) {
            return DefaultReturn {
                success: false,
                message: String::from("Name is invalid"),
                payload: Option::None,
            };
        }

        // (characters used)
        let regex = regex::RegexBuilder::new("^[\\w\\_\\-\\.\\!]+$")
            .multi_line(true)
            .build()
            .unwrap();

        if regex.captures(&p.name).iter().len() < 1 {
            return DefaultReturn {
                success: false,
                message: String::from("Name is invalid"),
                payload: Option::None,
            };
        }

        // make sure board does not exist
        let existing: DefaultReturn<Option<Board<String>>> =
            self.get_board_by_name(p.name.to_owned()).await;

        if existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Board already exists!"),
                payload: Option::None,
            };
        }

        // create board
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "INSERT INTO \"Boards\" VALUES (?, ?, ?)"
        } else {
            "INSERT INTO \"Boards\" VALUES ($1, $2, $3)"
        };

        let c = &self.db.client;
        let p: &mut Board<String> = &mut props.clone();
        p.timestamp = utility::unix_epoch_timestamp();

        let res = sqlx::query(query)
            .bind::<&String>(&p.name)
            .bind::<&String>(&p.timestamp.to_string())
            .bind::<&String>(&serde_json::to_string(&metadata).unwrap())
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: res.err().unwrap().to_string(),
                payload: Option::None,
            };
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Created board"),
            payload: Option::Some(p.to_owned()),
        };
    }

    /// Create a new post in a given [`Board`]
    ///
    /// # Arguments:
    /// * `props` - [`BoardPostLog`]
    /// * `as_user` - The ID of the user creating the post
    pub async fn create_board_post(
        &self,
        props: &mut BoardPostLog,
        as_user: Option<String>, // username of user posting
        as_role: Option<String>, // role of user posting
    ) -> DefaultReturn<Option<String>> {
        let p: &mut BoardPostLog = props; // borrowed props

        // check values

        // (check length)
        if (p.content.len() < 2) | (p.content.len() > 200_000) {
            return DefaultReturn {
                success: false,
                message: String::from("Content is invalid"),
                payload: Option::None,
            };
        }

        // make sure this board exists
        let existing: DefaultReturn<Option<Board<String>>> =
            self.get_board_by_name(p.board.to_owned()).await;

        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Board does not exist!"),
                payload: Option::None,
            };
        }

        let board =
            serde_json::from_str::<BoardMetadata>(&existing.payload.unwrap().metadata).unwrap();

        // check board "allow_anonymous" setting
        if board.allow_anonymous.is_some()
            && board.allow_anonymous.unwrap() == String::from("no")
            && as_user.is_none()
        {
            return DefaultReturn {
                success: false,
                message: String::from("An account is required to do this"),
                payload: Option::None,
            };
        }

        // check board "allow_open_posting" setting
        if board.allow_open_posting.is_some()
            && board.allow_open_posting.unwrap() == String::from("no")
        {
            let can_post = as_user.is_some()
                && ((as_user.as_ref().unwrap() == &board.owner) | (as_role.unwrap() == "staff"));

            if can_post == false {
                return DefaultReturn {
                    success: false,
                    message: String::from("You do not have permission to do this"),
                    payload: Option::None,
                };
            }
        }

        // create post
        let post = BoardPostLog {
            author: if as_user.is_some() {
                as_user.unwrap()
            } else {
                String::from("Anonymous")
            },
            content: p.content.clone(),
            content_html: crate::markdown::render::parse_markdown(&p.content),
            board: p.board.clone(),
            is_hidden: false,
            reply: p.reply.clone(),
            pinned: Option::Some(false),
            replies: Option::None,
        };

        // return
        self.create_log(
            String::from("board_post"),
            serde_json::to_string::<BoardPostLog>(&post).unwrap(),
        )
        .await
    }

    /// Update a [`Paste`]'s metadata by its `custom_url`
    pub async fn edit_board_metadata_by_name(
        &self,
        name: String,
        metadata: BoardMetadata,
        edit_as: Option<String>, // username of account that is editing this board
    ) -> DefaultReturn<Option<String>> {
        // make sure board exists
        let existing = &self.get_board_by_name(name.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Board does not exist!"),
                payload: Option::None,
            };
        }

        // get edit_as user account
        let ua = if edit_as.is_some() {
            Option::Some(
                self.get_user_by_username(edit_as.clone().unwrap())
                    .await
                    .payload,
            )
        } else {
            Option::None
        };

        if ua.is_none() {
            return DefaultReturn {
                success: false,
                message: String::from("An account is required to do this"),
                payload: Option::None,
            };
        }

        // make sure we can do this
        let user = ua.unwrap().unwrap();
        let can_edit: bool =
            (user.username == metadata.owner) | (user.role == String::from("staff"));

        if can_edit == false {
            return DefaultReturn {
                success: false,
                message: String::from(
                    "You do not have permission to manage this board's contents.",
                ),
                payload: Option::None,
            };
        }

        // update paste
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "UPDATE \"Boards\" SET \"metadata\" = ? WHERE \"name\" = ?"
        } else {
            "UPDATE \"Boards\" SET (\"metadata\") = ($1) WHERE \"name\" = $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&serde_json::to_string(&metadata).unwrap())
            .bind::<&String>(&name)
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
            message: String::from("Board updated!"),
            payload: Option::Some(name),
        };
    }

    /// Delete a board given its name
    ///
    /// # Arguments:
    /// * `name` - `String` of the board's `name`
    pub async fn delete_board(&self, name: String) -> DefaultReturn<Option<String>> {
        // make sure log exists
        let existing = &self.get_board_by_name(name.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Board does not exist!"),
                payload: Option::None,
            };
        }

        // delete board
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "DELETE FROM \"Boards\" WHERE \"name\" = ?"
        } else {
            "DELETE FROM \"Boards\" WHERE \"name\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind::<&String>(&name).execute(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // delete board messages
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "DELETE FROM \"Logs\" WHERE \"content\" LIKE ?"
        } else {
            "DELETE FROM \"Logs\" WHERE \"content\" LIKE $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"board\":\"{}\"%", name))
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
            message: String::from("Board deleted!"),
            payload: Option::Some(name),
        };
    }
}
