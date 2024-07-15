use axum::{routing::get, Router};
use pastemd::{database::Database, routing::api};
use std::env;

mod base;
mod markdown;
mod pages;

pub fn env_options() -> pastemd::DatabaseOpts {
    use std::env::var;
    pastemd::DatabaseOpts {
        _type: match var("DB_TYPE") {
            Ok(v) => Option::Some(v),
            Err(_) => Option::None,
        },
        host: match var("DB_HOST") {
            Ok(v) => Option::Some(v),
            Err(_) => Option::None,
        },
        user: var("DB_USER").unwrap_or(String::new()),
        pass: var("DB_PASS").unwrap_or(String::new()),
        name: var("DB_NAME").unwrap_or(String::new()),
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok(); // load .env

    let port: u16 = match env::var("PORT") {
        Ok(v) => v.parse::<u16>().unwrap(),
        Err(_) => 8080,
    };

    // init database
    let database = Database::new(
        env_options(),
        pastemd::database::ServerOptions {
            view_password: true,
            guppy: env::var("GUPPY_ROOT").is_ok(),
            paste_ownership: true,
            view_mode: if env::var("GUPPY_ROOT").is_ok() {
                pastemd::database::ViewMode::AuthenticatedOnce
            } else {
                pastemd::database::ViewMode::OpenMultiple
            },
            table_pastes: pastemd::database::PastesTableConfig {
                table_name: "cr_pastes".to_string(),
                prefix: "cr_paste".to_string(),
                url: "custom_url".to_string(),
                password: "edit_password".to_string(),
                content: "content".to_string(),
                date_published: "pub_date".to_string(),
                date_edited: "edit_date".to_string(),
                ..Default::default()
            },
            table_views: pastemd::database::ViewsTableConfig {
                table_name: "cr_views".to_string(),
                prefix: "cr_views".to_string(),
            },
        },
    )
    .await;

    database.init().await;

    // ...
    let app = Router::new()
        .route("/", get(pages::homepage))
        .merge(pages::routes(database.clone()))
        .nest("/api", api::routes(database.clone()))
        .fallback(api::not_found);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();

    println!("Starting server at http://localhost:{port}!");
    axum::serve(listener, app).await.unwrap();
}
