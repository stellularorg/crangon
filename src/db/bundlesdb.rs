use super::sql::{self, Database, DatabaseOpts};
use sqlx::{Executor, Row};

use crate::utility;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppData {
    pub db: BundlesDB,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DefaultReturn<T> {
    pub success: bool,
    pub message: String,
    pub payload: T,
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
    pub metadata: M, // JSON Object
    pub views: usize,
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
    pub owner: String, // custom_url of owner paste
}

#[derive(PartialEq, sqlx::FromRow, Clone, Serialize, Deserialize)]
pub struct UserState {
    // selectors
    pub username: String,
    pub id_hashed: String, // users use their UNHASHED id to login, it is used as their session id too!
    //                        the hashed id is the only id that should ever be public!
    // dates
    pub timestamp: u128,
}

#[derive(PartialEq, sqlx::FromRow, Clone, Serialize, Deserialize)]
pub struct Log {
    // selectors
    pub id: String,
    pub logtype: String,
    // dates
    pub timestamp: u128,
    // ...
    pub content: String,
}

// ...
#[derive(Clone)]
pub struct BundlesDB {
    pub db: Database,
}

impl BundlesDB {
    pub async fn new(options: DatabaseOpts) -> BundlesDB {
        return BundlesDB {
            db: sql::create_db(options).await,
        };
    }

    pub async fn init(&mut self) {
        // ...

        // create tables
        let c = &self.db.client;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Pastes\" (
                custom_url TEXT NOT NULL,
                id TEXT NOT NULL,
                group_name TEXT NOT NULL,
                edit_password TEXT NOT NULL,
                pub_date TEXT NOT NULL,
                edit_date TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT NOT NULL
            )",
        )
        .execute(c)
        .await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Groups\" (
                name TEXT NOT NULL,
                submit_password TEXT NOT NULL,
                metadata TEXT NOT NULL
            )",
        )
        .execute(c)
        .await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Users\" (
                username TEXT NOT NULL,
                id_hashed TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )",
        )
        .execute(c)
        .await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Logs\" (
                id TEXT NOT NULL,
                logtype TEXT NOT NULL,
                timestamp  TEXT NOT NULL,
                content TEXT NOT NULL
            )",
        )
        .execute(c)
        .await;
    }

    // users

    // GET
    pub async fn get_user_by_hashed(&self, hashed: String) -> DefaultReturn<Option<UserState>> {
        let query: &str = if self.db._type == "sqlite" {
            "SELECT * FROM \"Users\" WHERE \"id_hashed\" = ?"
        } else {
            "SELECT * FROM \"Users\" WHERE \"id_hashed\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind(&hashed).fetch_one(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("User does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();

        // return
        return DefaultReturn {
            success: true,
            message: String::from("User exists"),
            payload: Option::Some(UserState {
                username: row.get("username"),
                id_hashed: row.get("id_hashed"),
                timestamp: row.get::<String, _>("timestamp").parse::<u128>().unwrap(),
            }),
        };
    }

    pub async fn get_user_by_username(&self, username: String) -> DefaultReturn<Option<UserState>> {
        let query: &str = if self.db._type == "sqlite" {
            "SELECT * FROM \"Users\" WHERE \"username\" = ?"
        } else {
            "SELECT * FROM \"Users\" WHERE \"username\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind(&username).fetch_one(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("User does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();

        // return
        return DefaultReturn {
            success: true,
            message: String::from("User exists"),
            payload: Option::Some(UserState {
                username: row.get("username"),
                id_hashed: row.get("id_hashed"),
                timestamp: row.get::<String, _>("timestamp").parse::<u128>().unwrap(),
            }),
        };
    }

    // SET
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
        let query: &str = if self.db._type == "sqlite" {
            "INSERT INTO \"Users\" VALUES (?, ?, ?)"
        } else {
            "INSERT INTO \"Users\" VALUES ($1, $2, $3)"
        };

        let user_id_unhashed: String = utility::uuid();
        let user_id_hashed: String = utility::hash(user_id_unhashed.clone());
        let timestamp = utility::unix_epoch_timestamp().to_string();

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind(username)
            .bind(&user_id_hashed)
            .bind(&timestamp)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to create user"),
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
    pub async fn get_log_by_id(&self, id: String) -> DefaultReturn<Option<Log>> {
        let query: &str = if self.db._type == "sqlite" {
            "SELECT * FROM \"Logs\" WHERE \"id\" = ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"id\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind(&id).fetch_one(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Log does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste exists"),
            payload: Option::Some(Log {
                id: row.get("id"),
                logtype: row.get("logtype"),
                timestamp: row.get::<String, _>("timestamp").parse::<u128>().unwrap(),
                content: row.get("content"),
            }),
        };
    }

    // SET
    pub async fn create_log(
        &self,
        logtype: String,
        content: String,
    ) -> DefaultReturn<Option<String>> {
        let query: &str = if self.db._type == "sqlite" {
            "INSERT INTO \"Logs\" VALUES (?, ?, ?, ?)"
        } else {
            "INSERT INTO \"Logs\" VALUES ($1, $2, $3, $4)"
        };

        let log_id: String = utility::random_id();

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind(&log_id)
            .bind(logtype)
            .bind(utility::unix_epoch_timestamp().to_string())
            .bind(content)
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to create log"),
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
        let query: &str = if self.db._type == "sqlite" {
            "UPDATE \"Logs\" SET (\"content\") = (?) WHERE \"id\" = ?"
        } else {
            "UPDATE \"Logs\" SET (\"content\") = ($1) WHERE \"id\" = $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind(&content)
            .bind(&id)
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to update log"),
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
        let query: &str = if self.db._type == "sqlite" {
            "DELETE FROM \"Logs\" WHERE \"id\" = ?"
        } else {
            "DELETE FROM \"Logs\" WHERE \"id\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind(&id).fetch_one(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to delete log"),
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

    // GET
    pub async fn get_paste_by_url(&self, url: String) -> DefaultReturn<Option<Paste<String>>> {
        let query: &str = if self.db._type == "sqlite" {
            "SELECT * FROM \"Pastes\" WHERE \"custom_url\" = ?"
        } else {
            "SELECT * FROM \"Pastes\" WHERE \"custom_url\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind(&url.to_lowercase())
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

        // count views
        let query: &str = if self.db._type == "sqlite" {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'view_paste' AND \"content\" LIKE ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'view_paste' AND \"content\" LIKE $1"
        };

        let views_res = sqlx::query(query)
            .bind(format!("{}::%", &url))
            .fetch_all(c)
            .await;

        if views_res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch views"),
                payload: Option::None,
            };
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste exists"),
            payload: Option::Some(Paste {
                custom_url: row.get("custom_url"),
                id: row.get("id"),
                group_name: row.get("group_name"),
                edit_password: row.get("edit_password"),
                pub_date: row.get::<String, _>("pub_date").parse::<u128>().unwrap(),
                edit_date: row.get::<String, _>("edit_date").parse::<u128>().unwrap(),
                content: row.get("content"),
                metadata: row.get::<String, _>("metadata"),
                views: views_res.unwrap().len(),
            }),
        };
    }

    // SET
    pub async fn create_paste(
        &self,
        props: &mut Paste<String>,
        as_user: Option<String>, // id of paste owner
    ) -> DefaultReturn<Option<Paste<String>>> {
        let p: &mut Paste<String> = props; // borrowed props

        // create default metadata
        let metadata: PasteMetadata = PasteMetadata {
            owner: if as_user.is_some() {
                as_user.unwrap()
            } else {
                String::new()
            },
            private_source: String::from("off"),
            // optionals
            title: Option::Some(String::new()),
            description: Option::Some(String::new()),
            favicon: Option::None,
            embed_color: Option::Some(String::from("#ff9999")),
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
                        message: String::from("Failed to create group!"),
                        payload: Option::None,
                    };
                }
            } else {
                // check group password
                if utility::hash(e.to_string())
                    != utility::hash(existing_group.payload.unwrap().submit_password)
                {
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
        let query: &str = if self.db._type == "sqlite" {
            "INSERT INTO \"Pastes\" VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        } else {
            "INSERT INTO \"Pastes\" VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        };

        let c: &sqlx::Pool<sqlx::Any> = &self.db.client;
        let p: &mut Paste<String> = &mut props.clone();
        p.id = utility::random_id();

        let edit_password = &p.edit_password;
        let edit_password_hash = utility::hash(edit_password.to_string());

        let edit_date = &p.edit_date;
        let pub_date = &p.pub_date;

        let res = sqlx::query(query)
            .bind(&p.custom_url)
            .bind(&p.id)
            .bind(&p.group_name)
            .bind(&edit_password_hash)
            .bind(pub_date.to_string())
            .bind(edit_date.to_string())
            .bind(&p.content)
            .bind(serde_json::to_string(&metadata).unwrap())
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
        let query: &str = if self.db._type == "sqlite" {
            "UPDATE \"Pastes\" SET (\"content\", \"edit_password\", \"custom_url\", \"edit_date\") = (?, ?, ?, ?) WHERE \"custom_url\" = ?"
        } else {
            "UPDATE \"Pastes\" SET (\"content\", \"edit_password\", \"custom_url\", \"edit_date\") = ($1, $2, $3, $4) WHERE \"custom_url\" = $5"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind(&content)
            .bind(&edit_password_hash)
            .bind(&custom_url)
            .bind(utility::unix_epoch_timestamp().to_string()) // update edit_date
            .bind(&url)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to update paste"),
                payload: Option::None,
            };
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste updated!"),
            payload: Option::Some(custom_url.to_string()),
        };
    }

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

        // update paste
        let query: &str = if self.db._type == "sqlite" {
            "UPDATE \"Pastes\" SET (\"metadata\") = (?) WHERE \"custom_url\" = ?"
        } else {
            "UPDATE \"Pastes\" SET (\"metadata\") = ($1) WHERE \"custom_url\" = $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind(serde_json::to_string(&metadata).unwrap())
            .bind(&url)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to update paste"),
                payload: Option::None,
            };
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste updated!"),
            payload: Option::Some(url),
        };
    }

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
        let query: &str = if self.db._type == "sqlite" {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'view_paste' AND \"content\" LIKE ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'view_paste' AND \"content\" LIKE $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind(format!("{}::{}", &url, &view_as))
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

    pub async fn delete_paste_by_url(
        &self,
        url: String,
        edit_password: String,
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

        // verify password
        let paste = &existing.payload.clone().unwrap();

        if utility::hash(edit_password) != paste.edit_password {
            return DefaultReturn {
                success: false,
                message: String::from("Password invalid"),
                payload: Option::None,
            };
        }

        // delete paste
        let query: &str = if self.db._type == "sqlite" {
            "DELETE FROM \"Pastes\" WHERE \"custom_url\" = ?"
        } else {
            "DELETE FROM \"Pastes\" WHERE \"custom_url\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind(&url).execute(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to delete paste"),
                payload: Option::None,
            };
        }

        // delete paste views
        let query: &str = if self.db._type == "sqlite" {
            "DELETE FROM \"Logs\" WHERE \"content\" LIKE ?"
        } else {
            "DELETE FROM \"Logs\" WHERE \"content\" LIKE $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind(format!("{}::%", &url))
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to delete paste"),
                payload: Option::None,
            };
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
    pub async fn get_group_by_name(&self, url: String) -> DefaultReturn<Option<Group<String>>> {
        let query: &str = if self.db._type == "sqlite" {
            "SELECT * FROM \"Groups\" WHERE \"name\" = ?"
        } else {
            "SELECT * FROM \"Groups\" WHERE \"name\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind(&url).fetch_one(c).await;

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
                metadata: row.get::<String, _>("metadata"),
            }),
        };
    }

    // SET
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
        let query: &str = if self.db._type == "sqlite" {
            "INSERT INTO \"Groups\" VALUES (?, ?, ?)"
        } else {
            "INSERT INTO \"Groups\" VALUES ($1, $2, $3)"
        };

        let c: &sqlx::Pool<sqlx::Any> = &self.db.client;
        let p: &mut Group<GroupMetadata> = &mut props.clone();

        let res = sqlx::query(query)
            .bind(&p.name)
            .bind(&p.submit_password)
            .bind(serde_json::to_string(&p.metadata).unwrap())
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
}

pub fn create_dummy(mut custom_url: Option<&str>) -> DefaultReturn<Option<Paste<String>>> {
    if custom_url.is_none() {
        custom_url = Option::Some("dummy_paste");
    }

    return DefaultReturn {
        success: true,
        message: String::from("Paste exists"),
        payload: Option::Some(Paste {
            custom_url: custom_url.unwrap().to_string(),
            id: "".to_string(),
            group_name: "".to_string(),
            // passwords
            edit_password: "".to_string(),
            // dates
            pub_date: utility::unix_epoch_timestamp(),
            edit_date: utility::unix_epoch_timestamp(),
            // ...
            content: "dummy paste".to_string(),
            metadata: "".to_string(),
            views: 0,
        }),
    };
}
