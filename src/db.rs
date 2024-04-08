//! # Database
//! Database handler for all database types
use crate::utility;
use serde::{Deserialize, Serialize};

use dorsal::query as sqlquery;

#[derive(Clone)]
pub struct AppData {
    pub db: Database,
    pub http_client: awc::Client,
}

pub use dorsal::db::special::auth_db::{FullUser, UserMetadata, UserState};

pub use dorsal::db::special::log_db::Log;
pub use dorsal::DefaultReturn;

// Paste and Group require the type of their metadata to be specified so it can be converted if needed
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct PasteIdentifier {
    pub custom_url: String,
    pub id: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PasteMetadata {
    pub owner: String,
    pub private_source: String,
    // optionals
    pub title: Option<String>,
    pub description: Option<String>,
    pub favicon: Option<String>,
    pub embed_color: Option<String>,
    pub view_password: Option<String>,
    pub page_template: Option<String>, // handlebars formatted page template
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct FullPaste<M, U> {
    pub paste: Paste<M>,
    pub user: Option<FullUser<U>>,
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

#[derive(Default, PartialEq, Clone, Serialize, Deserialize)]
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

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Notification {
    pub user: String,    // the user that is being notified
    pub content: String, // notification text
    pub address: String, // notification redirect url
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
// Takes the place of "about" in BoardMetadata, identifies a board as a user mail stream
pub struct UserMailStreamIdentifier {
    pub _is_user_mail_stream: bool, // always going to be true ... cannot be edited into board about ANYWHERE
    pub user1: String,              // username of first user
    pub user2: String,              // username of second user
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardMetadata {
    pub owner: String,                      // username of owner
    pub is_private: String, // if the homepage of the board is shown to other users (not owner)
    pub allow_anonymous: Option<String>, // if anonymous users can post
    pub allow_open_posting: Option<String>, // if all users can post on the board (not just owner)
    pub topic_required: Option<String>, // if posts are required to include a topic value
    pub about: Option<String>, // welcome message
    pub tags: Option<String>, // SPACE separated list of tags that identify the board for searches
                            // TODO: we could likely export a list of "valid" tags at some point in the future
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardPostLog {
    pub author: String, // username of owner
    pub content: String,
    pub content_html: String,
    pub topic: Option<String>, // post topic, content is hidden unless expanded if provided
    pub board: String,         // name of board the post is located in
    pub is_hidden: bool,       // if the post is hidden in the feed (does nothing right now)
    pub reply: Option<String>, // the ID of the post we're replying to
    pub pinned: Option<bool>,  // pin the post to the top of the board feed
    pub replies: Option<usize>, // not really managed in the log, just used to show the number of replies this post has
    pub tags: Option<String>,   // same as board tags, just for posts specifically
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardIdentifier {
    pub name: String,
    pub tags: String,
}

// ...
#[derive(Clone)]
pub struct Database {
    pub base: dorsal::StarterDatabase,
    pub auth: dorsal::AuthDatabase,
    pub logs: dorsal::LogDatabase,
}

impl Database {
    pub async fn new(opts: dorsal::DatabaseOpts) -> Database {
        let db = dorsal::StarterDatabase::new(opts).await;

        Database {
            base: db.clone(),
            auth: dorsal::AuthDatabase { base: db.clone() },
            logs: dorsal::LogDatabase { base: db },
        }
    }

    pub async fn init(&self) {
        // create tables
        let c = &self.base.db.client;
        // MAX = 1000000
        // we're just using the same max length for everything because lengths are checked before being sent to db

        let _ = sqlquery(
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

        let _ = sqlquery(
            "CREATE TABLE IF NOT EXISTS \"Groups\" (
                name VARCHAR(1000000),
                submit_password VARCHAR(1000000),
                metadata VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        let _ = sqlquery(
            "CREATE TABLE IF NOT EXISTS \"Users\" (
                username VARCHAR(1000000),
                id_hashed VARCHAR(1000000),
                role VARCHAR(1000000),
                timestamp VARCHAR(1000000),
                metadata VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        let _ = sqlquery(
            "CREATE TABLE IF NOT EXISTS \"Logs\" (
                id VARCHAR(1000000),
                logtype VARCHAR(1000000),
                timestamp  VARCHAR(1000000),
                content VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        let _ = sqlquery(
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
    /// Get a user by their unhashed ID (hashes ID and then calls [`Database::get_user_by_hashed()`])
    ///
    /// Calls [`Database::get_user_by_unhashed_st()`] if user is invalid.
    ///
    /// # Arguments:
    /// * `unhashed` - `String` of the user's unhashed ID
    pub async fn get_user_by_unhashed(
        &self,
        unhashed: String,
    ) -> DefaultReturn<Option<FullUser<String>>> {
        self.auth.get_user_by_unhashed(unhashed).await
    }

    /// Get a user by their username
    ///
    /// # Arguments:
    /// * `username` - `String` of the user's username
    pub async fn get_user_by_username(
        &self,
        username: String,
    ) -> DefaultReturn<Option<FullUser<String>>> {
        self.auth.get_user_by_username(username).await
    }

    // SET
    /// Ban a [`UserState`] by its `username`
    pub async fn ban_user_by_name(&self, name: String) -> DefaultReturn<Option<String>> {
        // make sure user exists
        let existing = &self.get_user_by_username(name.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("User does not exist!"),
                payload: Option::None,
            };
        }

        // make sure user role is "member"
        let user = &existing.payload.as_ref().unwrap().user;
        if user.role != "member" {
            return DefaultReturn {
                success: false,
                message: String::from("User must be of role \"member\""),
                payload: Option::None,
            };
        }

        // update user
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "UPDATE \"Users\" SET \"role\" = ? WHERE \"username\" = ?"
        } else {
            "UPDATE \"Users\" SET (\"role\") = ($1) WHERE \"username\" = $2"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
            .bind::<&str>("banned")
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

        // lock user assets
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "UPDATE \"Pastes\" SET \"metadata\" = ? WHERE \"metadata\" LIKE ?"
        } else {
            "UPDATE \"Pastes\" SET (\"metadata\") = ($1) WHERE \"metadata\" LIKE $2"
        };

        let c = &self.base.db.client;
        // TODO: some kind of bulk cache update to handle this
        let res = sqlquery(query)
            .bind::<&String>(
                &serde_json::to_string::<PasteMetadata>(&PasteMetadata {
                    // lock editors out
                    owner: String::new(),
                    private_source: String::from("on"),
                    // optionals
                    title: Option::Some(String::new()),
                    description: Option::Some(String::new()),
                    favicon: Option::None,
                    embed_color: Option::None,
                    view_password: Option::Some(format!(
                        "LOCKED(USER_BANNED)-{}",
                        crate::utility::random_id()
                    )),
                    page_template: Option::None,
                })
                .unwrap(),
            )
            .bind::<&String>(&format!("%\"owner\":\"{name}\"%"))
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // update cache
        let existing_in_cache = self.base.cachedb.get(format!("user:{}", name)).await;

        if existing_in_cache.is_some() {
            let mut user =
                serde_json::from_str::<UserState<String>>(&existing_in_cache.unwrap()).unwrap();
            user.role = String::from("banned"); // update role

            // update cache
            self.base
                .cachedb
                .update(
                    format!("user:{}", name),
                    serde_json::to_string::<UserState<String>>(&user).unwrap(),
                )
                .await;
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("User banned!"),
            payload: Option::Some(name),
        };
    }

    // pastes

    /// Count the `view_paste` logs for a specific [`Paste`]
    async fn count_paste_views(&self, custom_url: String) -> usize {
        let c = &self.base.db.client;

        // count views
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT \"ID\" FROM \"Logs\" WHERE \"logtype\" = 'view_paste' AND \"content\" LIKE ?"
        } else {
            "SELECT \"ID\" FROM \"Logs\" WHERE \"logtype\" = 'view_paste' AND \"content\" LIKE $1"
        };

        let views_res = sqlquery(query)
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
    ) -> DefaultReturn<Option<FullPaste<PasteMetadata, String>>> {
        // check in cache
        let cached = self.base.cachedb.get(format!("paste:{}", selector)).await;

        if cached.is_some() {
            // ...
            let paste =
                serde_json::from_str::<Paste<PasteMetadata>>(cached.unwrap().as_str()).unwrap();

            // get user
            let user = if paste.metadata.owner.len() > 0 {
                // TODO: maybe don't clone here
                (self
                    .get_user_by_username(paste.clone().metadata.owner)
                    .await)
                    .payload
            } else {
                Option::None
            };

            // return
            return DefaultReturn {
                success: true,
                message: String::from("Paste exists (cache)"),
                payload: Option::Some(FullPaste { paste, user }),
            };
        }

        // fetch from db
        let c = &self.base.db.client;
        let res = sqlquery(query)
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
        let row = self.base.textify_row(row).data;

        // get views
        let views = &self
            .count_paste_views(row.get("custom_url").unwrap().to_owned())
            .await;

        // ...
        let metadata = serde_json::from_str::<PasteMetadata>(row.get("metadata").unwrap()).unwrap();

        let paste = Paste {
            custom_url: row.get("custom_url").unwrap().to_string(),
            id: row.get("id").unwrap().to_string(),
            group_name: row.get("group_name").unwrap().to_string(),
            edit_password: row.get("edit_password").unwrap().to_string(),
            pub_date: row.get("pub_date").unwrap().parse::<u128>().unwrap(),
            edit_date: row.get("edit_date").unwrap().parse::<u128>().unwrap(),
            content: row.get("content").unwrap().to_string(),
            content_html: row.get("content_html").unwrap().to_string(),
            metadata,
            views: views.to_owned(),
        };

        // store in cache
        self.base
            .cachedb
            .set(
                format!("paste:{}", paste.custom_url),
                serde_json::to_string::<Paste<PasteMetadata>>(&paste).unwrap(),
            )
            .await;

        // get user
        let user = if paste.metadata.owner.len() > 0 {
            // TODO: maybe don't clone here
            (self
                .get_user_by_username(paste.clone().metadata.owner)
                .await)
                .payload
        } else {
            Option::None
        };

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste exists (new)"),
            payload: Option::Some(FullPaste { paste, user }),
        };
    }

    // GET
    /// Get a [`Paste`] given its `custom_url`
    ///
    /// # Arguments:
    /// * `url` - `String` of the paste's `custom_url`
    pub async fn get_paste_by_url(
        &self,
        url: String,
    ) -> DefaultReturn<Option<FullPaste<PasteMetadata, String>>> {
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Pastes\" WHERE \"custom_url\" = ?"
        } else {
            "SELECT * FROM \"Pastes\" WHERE \"custom_url\" = $1"
        };

        return self.build_result_from_query(query, &url).await;
    }

    /// Get a [`Paste`] given its `id`
    ///
    /// # Arguments:
    /// * `id` - `String` of the paste's `id`
    pub async fn get_paste_by_id(
        &self,
        id: String,
    ) -> DefaultReturn<Option<FullPaste<PasteMetadata, String>>> {
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Pastes\" WHERE \"id\" = ?"
        } else {
            "SELECT * FROM \"Pastes\" WHERE \"id\" = $1"
        };

        return self.build_result_from_query(query, &id).await;
    }

    /// Get all [pastes](Paste) owned by a specific user (limited)
    ///
    /// # Arguments:
    /// * `owner` - `String` of the owner's `username`
    /// * `offset` - optional value representing the SQL fetch offset
    pub async fn get_pastes_by_owner_limited(
        &self,
        owner: String,
        offset: Option<i32>,
    ) -> DefaultReturn<Option<Vec<PasteIdentifier>>> {
        let offset = if offset.is_some() { offset.unwrap() } else { 0 };

        // check in cache
        let cached = self
            .base
            .cachedb
            .get(format!("pastes-by-owner:{}:offset{}", owner, offset))
            .await;

        if cached.is_some() {
            // ...
            let pastes =
                serde_json::from_str::<Vec<PasteIdentifier>>(cached.unwrap().as_str()).unwrap();

            // return
            return DefaultReturn {
                success: true,
                message: owner,
                payload: Option::Some(pastes),
            };
        }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Pastes\" WHERE \"metadata\" LIKE ? ORDER BY \"pub_date\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Pastes\" WHERE \"metadata\" LIKE $1 ORDER BY \"pub_date\" DESC LIMIT 50 OFFSET $2"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
            .bind::<&String>(&format!("%\"owner\":\"{}\"%", &owner))
            .bind(offset)
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
            let row = self.base.textify_row(row).data;
            full_res.push(PasteIdentifier {
                custom_url: row.get("custom_url").unwrap().to_string(),
                id: row.get("id").unwrap().to_string(),
            });
        }

        // store in cache
        self.base
            .cachedb
            .set(
                format!("pastes-by-owner:{}:offset{}", owner, offset),
                serde_json::to_string::<Vec<PasteIdentifier>>(&full_res).unwrap(),
            )
            .await;

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
        // check in cache
        let cached = self
            .base
            .cachedb
            .get(format!("pastes-by-owner-atomic:{}:atomic", owner))
            .await;

        if cached.is_some() {
            // ...
            let pastes =
                serde_json::from_str::<Vec<PasteIdentifier>>(cached.unwrap().as_str()).unwrap();

            // return
            return DefaultReturn {
                success: true,
                message: owner,
                payload: Option::Some(pastes),
            };
        }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Pastes\" WHERE \"metadata\" LIKE ? AND \"content\" LIKE ?"
        } else {
            "SELECT * FROM \"Pastes\" WHERE \"metadata\" LIKE $1 AND \"content\" LIKE $2"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
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
            let row = self.base.textify_row(row).data;
            full_res.push(PasteIdentifier {
                custom_url: row.get("custom_url").unwrap().to_string(),
                id: row.get("id").unwrap().to_string(),
            });
        }

        // store in cache
        self.base
            .cachedb
            .set(
                format!("pastes-by-owner:{}:atomic", owner),
                serde_json::to_string::<Vec<PasteIdentifier>>(&full_res).unwrap(),
            )
            .await;

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
            page_template: Option::None,
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
        let regex = regex::RegexBuilder::new("^[\\w\\_\\-\\.\\!\\p{Extended_Pictographic}]+$")
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
        let existing: DefaultReturn<Option<FullPaste<PasteMetadata, String>>> =
            self.get_paste_by_url(p.custom_url.to_owned()).await;

        if existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Paste already exists!"),
                payload: Option::None,
            };
        }

        // create paste
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "INSERT INTO \"Pastes\" VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        } else {
            "INSERT INTO \"Pastes\" VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        };

        let c = &self.base.db.client;
        let p: &mut Paste<String> = &mut props.clone();
        p.id = utility::random_id();

        let edit_password = &p.edit_password;
        let edit_password_hash = utility::hash(edit_password.to_string());

        let edit_date = &p.edit_date;
        let pub_date = &p.pub_date;

        let res = sqlquery(query)
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

        // update cache
        if as_user.is_some() {
            self.base
                .cachedb
                .remove_starting_with(format!("pastes-by-owner:{}*", as_user.unwrap()))
                .await;
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
        let existing_metadata = &existing.payload.as_ref().unwrap().paste.metadata;

        // verify password
        // if password hash doesn't match AND edit_as is none OR edit_as != existing_metadata's owner value
        let paste = &existing.payload.clone().unwrap().paste;

        let skip_password_check = edit_as.is_some() && edit_as.unwrap() == existing_metadata.owner;

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
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "UPDATE \"Pastes\" SET \"content\" = ?, \"content_html\" = ?, \"edit_password\" = ?, \"custom_url\" = ?, \"edit_date\" = ? WHERE \"custom_url\" = ?"
        } else {
            "UPDATE \"Pastes\" SET (\"content\", \"content_html\", \"edit_password\", \"custom_url\", \"edit_date\") = ($1, $2, $3, $4, $5) WHERE \"custom_url\" = $6"
        };

        let content_html = &crate::markdown::render::parse_markdown(&content);
        let edit_date = &utility::unix_epoch_timestamp().to_string();

        let c = &self.base.db.client;
        let res = sqlquery(query)
            .bind::<&String>(&content)
            .bind::<&String>(content_html)
            .bind::<&String>(&edit_password_hash)
            .bind::<&String>(&custom_url)
            .bind::<&String>(edit_date) // update edit_date
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

        // update cache
        let existing_in_cache = self.base.cachedb.get(format!("paste:{}", url)).await;

        if existing_in_cache.is_some() {
            let mut paste =
                serde_json::from_str::<Paste<PasteMetadata>>(&existing_in_cache.unwrap()).unwrap();

            paste.content = content; // update content
            paste.content_html = content_html.to_string(); // update content_html
            paste.edit_password = edit_password_hash; // update edit_password
            paste.edit_date = edit_date.parse::<u128>().unwrap(); // update edit_date
            paste.custom_url = custom_url.to_string(); // update custom_url

            // update cache
            self.base
                .cachedb
                .update(
                    format!("paste:{}", url),
                    serde_json::to_string::<Paste<PasteMetadata>>(&paste).unwrap(),
                )
                .await;
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
        let existing_metadata = &existing.payload.as_ref().unwrap().paste.metadata;

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
        let paste = &existing.payload.clone().unwrap().paste;

        // ...skip password check IF the user is the paste owner!
        let skip_password_check = (edit_as.is_some()
            && edit_as.unwrap() == existing_metadata.owner)
            // OR if the user has the "ManagePastes" permission
            | (ua.as_ref().is_some()
                && ua.as_ref().unwrap().is_some()
                && ua.unwrap().unwrap().level.permissions.contains(&String::from("ManagePastes")));

        if !skip_password_check && utility::hash(edit_password) != paste.edit_password {
            return DefaultReturn {
                success: false,
                message: String::from("Password invalid"),
                payload: Option::None,
            };
        }

        // update paste
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "UPDATE \"Pastes\" SET \"metadata\" = ? WHERE \"custom_url\" = ?"
        } else {
            "UPDATE \"Pastes\" SET (\"metadata\") = ($1) WHERE \"custom_url\" = $2"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
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

        // update cache
        let existing_in_cache = self.base.cachedb.get(format!("paste:{}", url)).await;

        if existing_in_cache.is_some() {
            let mut paste =
                serde_json::from_str::<Paste<PasteMetadata>>(&existing_in_cache.unwrap()).unwrap();
            paste.metadata = metadata; // update metadata

            // update cache
            self.base
                .cachedb
                .update(
                    format!("paste:{}", url),
                    serde_json::to_string::<Paste<PasteMetadata>>(&paste).unwrap(),
                )
                .await;
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
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'view_paste' AND \"content\" LIKE ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'view_paste' AND \"content\" LIKE $1"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
            .bind::<&String>(&format!("{}::{}", &url, &view_as))
            .fetch_one(c)
            .await;

        if res.is_err() {
            let err = res.err().unwrap();
            let err_message = err.to_string();

            // count view if message says no rows were returned
            if err_message.starts_with("no rows returned") {
                self.logs
                    .create_log(
                        String::from("view_paste"),
                        format!("{}::{}", &url, &view_as),
                    )
                    .await;

                // update cache
                let existing_in_cache = self.base.cachedb.get(format!("paste:{}", url)).await;

                if existing_in_cache.is_some() {
                    let mut paste =
                        serde_json::from_str::<Paste<PasteMetadata>>(&existing_in_cache.unwrap())
                            .unwrap();
                    paste.views += 1;

                    // update cache
                    self.base
                        .cachedb
                        .update(
                            format!("paste:{}", url),
                            serde_json::to_string::<Paste<PasteMetadata>>(&paste).unwrap(),
                        )
                        .await;
                }

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
        let existing_metadata = &existing.payload.as_ref().unwrap().paste.metadata;

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
        let paste = &existing.payload.clone().unwrap().paste;

        // ...skip password check IF the user is the paste owner!
        let skip_password_check = (delete_as.is_some()
                && delete_as.unwrap() == existing_metadata.owner)
                // OR if the user has the "ManagePastes" permission
                | (ua.as_ref().is_some()
                    && ua.as_ref().unwrap().is_some()
                    && ua.unwrap().unwrap().level.permissions.contains(&String::from("ManagePastes")));

        if !skip_password_check && utility::hash(edit_password) != paste.edit_password {
            return DefaultReturn {
                success: false,
                message: String::from("Password invalid"),
                payload: Option::None,
            };
        }

        // delete paste
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "DELETE FROM \"Pastes\" WHERE \"custom_url\" = ?"
        } else {
            "DELETE FROM \"Pastes\" WHERE \"custom_url\" = $1"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query).bind::<&String>(&url).execute(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // delete paste views
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "DELETE FROM \"Logs\" WHERE \"content\" LIKE ?"
        } else {
            "DELETE FROM \"Logs\" WHERE \"content\" LIKE $1"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
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

        // update cache
        self.base.cachedb.remove(format!("paste:{}", url)).await;

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
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Groups\" WHERE \"name\" = ?"
        } else {
            "SELECT * FROM \"Groups\" WHERE \"name\" = $1"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query).bind::<&String>(&url).fetch_one(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Group does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.base.textify_row(row).data;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Group exists"),
            payload: Option::Some(Group {
                name: row.get("name").unwrap().to_string(),
                submit_password: row.get("submit_password").unwrap().to_string(),
                metadata: row.get("metadata").unwrap().to_string(),
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
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "INSERT INTO \"Groups\" VALUES (?, ?, ?)"
        } else {
            "INSERT INTO \"Groups\" VALUES ($1, $2, $3)"
        };

        let c = &self.base.db.client;
        let p: &mut Group<GroupMetadata> = &mut props.clone();

        p.submit_password = utility::hash(p.submit_password.clone());
        let res = sqlquery(query)
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

    // notifications

    // GET
    /// Get the [`Notification`]s that belong to the given `user`
    ///
    /// # Arguments:
    /// * `user` - username of user to check
    pub async fn get_user_notifications(
        &self,
        user: String,
        offset: Option<i32>,
    ) -> DefaultReturn<Option<Vec<Log>>> {
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"logtype\" = 'notification' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"logtype\" = 'notification' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $2"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
            .bind::<&String>(&format!("%\"user\":\"{user}\"%"))
            .bind(if offset.is_some() { offset.unwrap() } else { 0 })
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch notifications"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<Log> = Vec::new();

        for row in rows {
            let row = self.base.textify_row(row).data;
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
            message: String::from("Notifications exist"),
            payload: Option::Some(output),
        };
    }

    /// Check if the given `user` has an active notification
    ///
    /// # Arguments:
    /// * `user` - username of user to check
    pub async fn user_has_notification(&self, user: String) -> DefaultReturn<Option<Vec<Log>>> {
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"logtype\" = 'notification' LIMIT 1"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"logtype\" = 'notification' LIMIT 1"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
            .bind::<&String>(&format!("%\"user\":\"{user}\"%"))
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch notification"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();

        if rows.len() == 0 {
            return DefaultReturn {
                success: true,
                message: String::from("No"),
                payload: Option::None,
            };
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Yes"),
            payload: Option::None,
        };
    }

    // SET
    /// Create a new [`Notification`] for a given [`UserState`]
    ///
    /// # Arguments:
    /// * `props` - [`Notification`]
    pub async fn push_user_notification(
        &self,
        props: &mut Notification,
    ) -> DefaultReturn<Option<String>> {
        let p: &mut Notification = props; // borrowed props

        // make sure user exists
        let existing: DefaultReturn<Option<FullUser<String>>> =
            self.get_user_by_username(p.user.to_owned()).await;

        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("User does not exist!"),
                payload: Option::None,
            };
        }

        // return
        self.logs
            .create_log(
                String::from("notification"),
                serde_json::to_string::<Notification>(&p).unwrap(),
            )
            .await
    }

    // boards

    // GET
    /// Get all [`UserMailStreamIdentifier`] [`Board`] by a single participating user
    ///
    /// # Arguments:
    /// * `user` - username of the user
    /// * `offset` - optional value representing the SQL fetch offset
    pub async fn get_user_mail_streams(
        &self,
        user: String,
        offset: Option<i32>,
    ) -> DefaultReturn<Vec<BoardIdentifier>> {
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Boards\" WHERE \"metadata\" LIKE ? OR \"metadata\" LIKE ? ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Boards\" WHERE \"metadata\" LIKE $1 OR \"metadata\" LIKE $2 ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $3"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
            .bind::<String>(if &self.base.db._type == "mysql" {
                format!("%\\\\\"user1\\\\\":\\\\\"{}\\\\\"%", user)
            } else {
                format!("%\\\"user1\\\":\\\"{}\\\"%", user)
            })
            .bind::<String>(if &self.base.db._type == "mysql" {
                format!("%\\\\\"user2\\\\\":\\\\\"{}\\\\\"%", user)
            } else {
                format!("%\\\"user2\\\":\\\"{}\\\"%", user)
            })
            .bind(if offset.is_some() { offset.unwrap() } else { 0 })
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Boards do not exist"),
                payload: Vec::new(),
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<BoardIdentifier> = Vec::new();

        for row in rows {
            let row = self.base.textify_row(row).data;

            let metadata =
                serde_json::from_str::<BoardMetadata>(row.get("metadata").unwrap()).unwrap();

            let mailstream =
                serde_json::from_str::<UserMailStreamIdentifier>(&metadata.about.unwrap()).unwrap();

            output.push(BoardIdentifier {
                name: row.get("name").unwrap().to_string(),
                // we're going to use tags to store the name of the other user
                tags: if user == mailstream.user1 {
                    mailstream.user2
                } else {
                    mailstream.user1
                },
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Boards exists"),
            payload: output,
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
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'board_post' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'board_post' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $1"
        };

        let c = &self.base.db.client;
        let res = sqlquery(query)
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
            let row = self.base.textify_row(row).data;
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
}
