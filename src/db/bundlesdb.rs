use super::sql::{self, Database, DatabaseOpts};
use sqlx::Executor;

#[derive(Default, PartialEq)]
pub struct Paste {
    // selectors
    pub custom_url: String,
    pub id: String,
    // passwords
    pub edit_password: String,
    // dates
    pub pub_date: i64,
    pub edit_date: i64,
    // ...
    pub content: String,
    pub metadata: PasteMetadata,
}

#[derive(Default, PartialEq)]
pub struct PasteMetadata {
    pub owner: String,
}

// ...
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
}

pub fn create_dummy(mut custom_url: Option<&str>) -> Paste {
    if custom_url.is_none() {
        custom_url = Option::Some("dummy_paste");
    }

    return Paste {
        custom_url: custom_url.unwrap().to_string(),
        id: "".to_string(),
        // passwords
        edit_password: "".to_string(),
        // dates
        pub_date: 0,
        edit_date: 0,
        // ...
        content: "".to_string(),
        metadata: PasteMetadata {
            owner: "".to_string(),
        },
    };
}
