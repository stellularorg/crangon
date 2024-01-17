pub struct DatabaseOpts {
    pub _type: Option<String>,
    pub host: Option<String>,
    pub user: String,
    pub pass: String,
    pub name: String,
}

// ...
#[derive(Clone)]
pub struct Database {
    pub client: sqlx::AnyPool,
    pub _type: String,
}

// ...
pub async fn create_db(options: DatabaseOpts) -> Database {
    let mut _type = options._type;

    if _type.is_none() {
        _type = Option::from("sqlite".to_string());
    }

    // create client
    if _type.unwrap() == "sqlite" {
        // sqlite
        let client = sqlx::AnyPool::connect("sqlite://test.db").await;

        if client.is_err() {
            panic!("Failed to connect to database!");
        }

        return Database {
            client: client.unwrap(),
            _type: String::from("sqlite"),
        };
    } else {
        // postgres
        let client = sqlx::AnyPool::connect(&format!(
            "postgres://{}:{}@{}/{}",
            options.user,
            options.pass,
            if options.host.is_some() {
                options.host.unwrap()
            } else {
                "127.0.0.1".to_string()
            },
            options.name
        ))
        .await;

        if client.is_err() {
            panic!("Failed to connect to database!");
        }

        return Database {
            client: client.unwrap(),
            _type: String::from("sqlite"),
        };
    }
}
