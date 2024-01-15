use super::sql::{self, Database, DatabaseOpts};
use sqlx::{Executor, Row};

use crate::utility;
use json;

#[derive(Clone)]
pub struct AppData {
    pub db: BundlesDB,
}

#[allow(dead_code)]
pub struct DefaultReturn<T> {
    pub success: bool,
    pub message: String,
    pub payload: T,
}

// Paste and Group require the type of their metadata to be specified so it can be converted if needed
#[derive(Default, PartialEq, sqlx::FromRow, Clone)]
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
}

#[derive(Default, PartialEq, sqlx::FromRow, Clone)]
pub struct Group<M> {
    // selectors
    pub name: String,
    // passwords
    pub submit_password: String,
    // ...
    pub metadata: M, // JSON Object
}

#[derive(Default, PartialEq, sqlx::FromRow, Clone)]
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
        let c = &mut self.db.client;

        c.execute(sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Pastes\" (
                custom_url TEXT NOT NULL,
                id TEXT NOT NULL,
                edit_password TEXT NOT NULL,
                pub_date int,
                edit_date int,
                content TEXT NOT NULL,
                metadata TEXT NOT NULL,
            )",
        ));

        c.execute(sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Groups\" (
                name TEXT NOT NULL,
                submit_password TEXT NOT NUL,,
                metadata TEXT NOT NULL
            )",
        ));

        c.execute(sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Logs\" (
                id TEXT NOT NULL,
                logtype TEXT NOT NULL,
                timestamp float,
                content TEXT NOT NULL
            )",
        ));
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

    // pastes

    // GET
    pub async fn get_paste_by_url(&self, url: String) -> DefaultReturn<Option<Paste<String>>> {
        let query: &str = if self.db._type == "sqlite" {
            "SELECT * FROM \"Pastes\" WHERE \"custom_url\" = ?"
        } else {
            "SELECT * FROM \"Pastes\" WHERE \"custom_url\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind(&url).fetch_one(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Paste does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();

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
            }),
        };
    }

    // SET
    pub async fn create_paste(
        &self,
        props: Paste<json::object::Object>,
    ) -> DefaultReturn<Option<String>> {
        let p: &Paste<json::object::Object> = &props; // borrowed props

        // make sure paste does not exist
        let existing: DefaultReturn<Option<Paste<String>>> =
            self.get_paste_by_url(p.custom_url.to_owned()).await;

        if existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Paste does not exist!"),
                payload: Option::None,
            };
        }

        // create paste
        let query: &str = if self.db._type == "sqlite" {
            "INSERT INTO \"Pastes\" VALUES (?, ?, ?, ?, ?, ?, ?)"
        } else {
            "INSERT INTO \"Pastes\" VALUES ($1, $2, $3, $4, $5, $6, $7)"
        };

        let c: &sqlx::Pool<sqlx::Any> = &self.db.client;
        let p: &mut Paste<json::object::Object> = &mut props.clone();
        p.id = utility::random_id();

        c.execute(
            sqlx::query(query)
                .bind(&p.custom_url)
                .bind(&p.id)
                .bind(&p.edit_password)
                .bind(&p.pub_date.to_string())
                .bind(&p.edit_date.to_string())
                .bind(&p.content)
                .bind(p.metadata.dump()),
        );

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste created"),
            payload: Option::Some(p.id.to_string()),
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
    pub async fn create_group(
        &self,
        props: Group<json::object::Object>,
    ) -> DefaultReturn<Option<String>> {
        let p: &Group<json::object::Object> = &props; // borrowed props

        // make sure group does not exist
        let existing: DefaultReturn<Option<Group<String>>> =
            self.get_group_by_name(p.name.to_owned()).await;

        if existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Group does not exist!"),
                payload: Option::None,
            };
        }

        // create paste
        let query: &str = if self.db._type == "sqlite" {
            "INSERT INTO \"Pastes\" VALUES (?, ?, ?, ?, ?, ?, ?)"
        } else {
            "INSERT INTO \"Pastes\" VALUES ($1, $2, $3, $4, $5, $6, $7)"
        };

        let c: &sqlx::Pool<sqlx::Any> = &self.db.client;
        let p: &mut Group<json::object::Object> = &mut props.clone();

        c.execute(
            sqlx::query(query)
                .bind(&p.name)
                .bind(&p.submit_password)
                .bind(p.metadata.dump()),
        );

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
            content: format!(
                "SentryTwo staff can be identified by the e\"[ class chip+badge ]\"staffe\"[ close class ]\" badge in the social section of the options modal of their paste (on the right when viewing their paste!). Anybody you see claiming to be staff without this badge should be immediately reported.

                Staff can be contacted for help with URL issues.
                
                ***
                
                - **Contact** on the [discord](https://discord.gg/sntry), or email [feedback@sentrytwo.com](mailto:feedback@sentrytwo.com)
                - **Socials**:
                    - [Discord](https://discord.gg/sntry)
                    - [Twitter (X)](https://x.com/sentrytwo)"
            ),
            metadata: "".to_string(),
        }),
    };
}
