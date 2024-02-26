#[derive(Debug, Clone)]
pub struct DatabaseOpts {
    pub _type: Option<String>,
    pub host: Option<String>,
    pub user: String,
    pub pass: String,
    pub name: String,
    pub cache_enabled: Option<String>,
}

// ...
#[derive(Clone)]
pub struct Database<T> {
    pub client: T,
    pub _type: String,
}

// ...
#[cfg(feature = "mysql")]
/// Create a new "mysql" database
pub async fn create_db(options: DatabaseOpts) -> Database<sqlx::MySqlPool> {
    // mysql
    let opts = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(25)
        .acquire_timeout(std::time::Duration::from_millis(2000))
        .idle_timeout(Some(std::time::Duration::from_secs(60 * 5)));
    // .max_lifetime(Some(std::time::Duration::from_secs(120)));

    let client = opts
        .connect(&format!(
            "mysql://{}:{}@{}/{}",
            options.user,
            options.pass,
            if options.host.is_some() {
                options.host.unwrap()
            } else {
                "localhost".to_string()
            },
            options.name
        ))
        .await;

    if client.is_err() {
        panic!("failed to connect to database: {}", client.err().unwrap());
    }

    return Database {
        client: client.unwrap(),
        _type: String::from("mysql"),
    };
}

#[cfg(feature = "postgres")]
/// Create a new "postgres" database
pub async fn create_db(options: DatabaseOpts) -> Database<sqlx::PgPool> {
    // postgres
    let opts = sqlx::postgres::PgPoolOptions::new()
        .max_connections(25)
        .acquire_timeout(std::time::Duration::from_millis(2000))
        .idle_timeout(Some(std::time::Duration::from_secs(60 * 5)));
    // .max_lifetime(Some(std::time::Duration::from_secs(120)));

    let client = opts
        .connect(&format!(
            "postgres://{}:{}@{}/{}",
            options.user,
            options.pass,
            if options.host.is_some() {
                options.host.unwrap()
            } else {
                "localhost".to_string()
            },
            options.name
        ))
        .await;

    if client.is_err() {
        panic!("failed to connect to database: {}", client.err().unwrap());
    }

    return Database {
        client: client.unwrap(),
        _type: String::from("postgres"),
    };
}

#[cfg(feature = "sqlite")]
/// Create a new "sqlite" database
pub async fn create_db(_options: DatabaseOpts) -> Database<sqlx::SqlitePool> {
    // sqlite
    let client = sqlx::SqlitePool::connect("sqlite://bundlrs.db").await;

    if client.is_err() {
        panic!("Failed to connect to database!");
    }

    return Database {
        client: client.unwrap(),
        _type: String::from("sqlite"),
    };
}
