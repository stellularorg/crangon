use actix_files as fs;
use actix_web::{web, App, HttpServer};
use dotenv;

mod config;
mod db;
mod log_db;

mod api;
mod model;
mod pages;

mod markdown;

use crate::db::{AppData, Database};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    // ...
    let args: Vec<String> = config::collect_arguments();

    let port_search: Option<String> = config::get_named_argument(&args, "port");
    let mut port: u16 = 8080;

    if port_search.is_some() {
        port = port_search.unwrap().parse::<u16>().unwrap();
    }

    let static_dir_flag: Option<String> = config::get_named_argument(&args, "static-dir");
    let public_serve_path = std::env::var("PUBLIC_PATH").unwrap_or(String::from("/public"));

    // create database
    let db_type: Option<String> = config::get_named_argument(&args, "db-type");
    let db_host: Option<String> = config::get_var("DB_HOST");
    let db_user: Option<String> = config::get_var("DB_USER");
    let db_pass: Option<String> = config::get_var("DB_PASS");
    let db_name: Option<String> = config::get_var("DB_NAME");

    let db_is_other: bool = db_type
        .clone()
        .is_some_and(|x| (x == String::from("postgres")) | (x == String::from("mysql")));

    if db_is_other && (db_user.is_none() | db_pass.is_none() | db_name.is_none()) {
        panic!("Missing required database config settings!");
    }

    let db: Database = Database::new(dorsal::DatabaseOpts {
        _type: db_type,
        host: db_host,
        user: if db_is_other {
            db_user.unwrap()
        } else {
            String::new()
        },
        pass: if db_is_other {
            db_pass.unwrap()
        } else {
            String::new()
        },
        name: if db_is_other {
            db_name.unwrap()
        } else {
            String::new()
        },
    })
    .await;

    db.init().await;

    // start server
    println!("Starting server at: http://localhost:{port}");

    // serve routes
    HttpServer::new(move || {
        let data = web::Data::new(AppData { db: db.clone() });
        let cors = actix_cors::Cors::default().send_wildcard();

        App::new()
            .app_data(web::Data::clone(&data))
            // middleware
            .wrap(actix_web::middleware::Logger::default())
            .wrap(cors)
            // static dir
            .service(
                fs::Files::new(
                    "/static",
                    if static_dir_flag.is_some() {
                        static_dir_flag.as_ref().unwrap()
                    } else {
                        "./static"
                    },
                )
                .show_files_listing(),
            )
            // public dir
            .service(
                fs::Files::new(&public_serve_path, "./public")
                    .show_files_listing()
                    .index_file("index.html")
                    .redirect_to_slash_directory(),
            )
            // docs
            .service(fs::Files::new("/api/docs", "./target/doc").show_files_listing())
            // POST api
            .service(crate::api::auth::ban_request)
            // POST api::pastes
            .service(crate::api::pastes::render_request)
            .service(crate::api::pastes::create_request)
            .service(crate::api::pastes::edit_request)
            .service(crate::api::pastes::delete_request)
            .service(crate::api::pastes::metadata_request)
            .service(crate::api::pastes::favorite_request)
            // GET api
            .service(crate::api::pastes::get_from_url_request)
            .service(crate::api::pastes::get_from_id_request)
            .service(crate::api::pastes::exists_request)
            .service(crate::api::auth::callback_request)
            .service(crate::api::auth::logout)
            .service(crate::api::auth::whoami) // get current username
            // GET dashboard
            .service(crate::pages::home::dashboard_request)
            .service(crate::pages::settings::user_settings_request)
            .service(crate::pages::settings::paste_settings_request)
            .service(crate::pages::paste_view::dashboard_request)
            // GET staff
            .service(crate::pages::staff::dashboard_request)
            .service(crate::pages::staff::staff_users_dashboard_request)
            .service(crate::pages::staff::staff_pastes_dashboard_request)
            // GET users
            .service(crate::api::auth::get_from_owner_request)
            // GET root
            .service(crate::pages::home::home_request)
            .service(crate::pages::home::robotstxt)
            .service(crate::pages::home::adstxt)
            .service(crate::pages::paste_view::paste_view_request) // must be run last as it matches all other paths!
            // ERRORS
            .default_service(web::to(|req, data| async {
                return crate::pages::errors::error404(req, data).await;
            }))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
