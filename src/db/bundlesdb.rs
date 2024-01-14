use super::sql::{self, Database, DatabaseOpts};
use sqlx::{Executor, Row};

use crate::utility;
use json;

#[allow(dead_code)]
pub struct DefaultReturn<T> {
    pub success: bool,
    pub message: String,
    pub payload: T,
}

#[derive(Default, PartialEq, sqlx::FromRow, Clone)]
pub struct Paste {
    // selectors
    pub custom_url: String,
    pub id: String,
    // passwords
    pub edit_password: String,
    // dates
    pub pub_date: u128,
    pub edit_date: u128,
    // ...
    pub content: String,
    pub metadata: String, // JSON Object
}

// ...
#[derive(Clone)]
pub struct BundlesDB {
    pub db: Database,
}

#[derive(Clone)]
pub struct AppData {
    pub db: BundlesDB,
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
        let query: &str = "CREATE TABLE IF NOT EXISTS \"Pastes\" (
            custom_url TEXT NOT NULL,
            id TEXT NOT NULL,
            edit_password TEXT NOT NULL,
            pub_date: int,
            edit_date: int,
            content: TEXT NOT NULL,
            metadata: TEXT NOT NULL,
        )";

        let c = &mut self.db.client;
        c.execute(sqlx::query(query));
    }

    // pastes

    // GET
    pub async fn get_paste_by_url(&self, url: String) -> DefaultReturn<Option<Paste>> {
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
                edit_password: row.get("edit_password"),
                pub_date: row.get::<String, _>("pub_date").parse::<u128>().unwrap(),
                edit_date: row.get::<String, _>("edit_date").parse::<u128>().unwrap(),
                content: row.get("content"),
                metadata: row.get("metadata"),
            }),
        };
    }

    // SET
    pub async fn create_paste(&self, props: Paste) -> DefaultReturn<&str> {
        let query: &str = if self.db._type == "sqlite" {
            "INSERT INTO \"Pastes\" VALUES (?, ?, ?, ?, ?, ?, ?)"
        } else {
            "INSERT INTO \"Pastes\" VALUES ($1, $2, $3, $4, $5, $6, $7)"
        };

        let c = &self.db.client;
        let p = props.clone();

        c.execute(
            sqlx::query(query)
                .bind(&p.custom_url)
                .bind(&p.id)
                .bind(&p.edit_password)
                .bind(&p.pub_date.to_string())
                .bind(&p.edit_date.to_string())
                .bind(&p.content)
                .bind(json::stringify(p.metadata)),
        );

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste created"),
            payload: query,
        };
    }
}

pub fn create_dummy(mut custom_url: Option<&str>) -> DefaultReturn<Option<Paste>> {
    if custom_url.is_none() {
        custom_url = Option::Some("dummy_paste");
    }

    return DefaultReturn {
        success: true,
        message: String::from("Paste exists"),
        payload: Option::Some(Paste {
            custom_url: custom_url.unwrap().to_string(),
            id: "".to_string(),
            // passwords
            edit_password: "".to_string(),
            // dates
            pub_date: utility::unix_epoch_timestamp(),
            edit_date: utility::unix_epoch_timestamp(),
            // ...
            content: format!("dummy url test: {}\n# hi\n## hi1", custom_url.unwrap()),
            metadata: "".to_string(),
        }),
    };
}
