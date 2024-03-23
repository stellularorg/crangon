use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer};
use dotenv;

use sqlx;
use yew::ServerRenderer;

mod config;
mod db;
mod utility;

mod api;
mod components;
mod pages;

mod markdown;
mod ssm;

use crate::db::bundlesdb::{AppData, BundlesDB};
use crate::db::sql::DatabaseOpts;

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

    sqlx::any::install_default_drivers(); // install database drivers
    let db: BundlesDB = BundlesDB::new(DatabaseOpts {
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
        cache_enabled: config::get_var("CACHE_ENABLED"),
    })
    .await;

    db.init().await;

    // start server
    println!("Starting server at: http://localhost:{port}");

    // serve routes
    HttpServer::new(move || {
        let client = awc::Client::default();
        let data = web::Data::new(AppData {
            db: db.clone(),
            http_client: client,
        });

        App::new()
            .app_data(web::Data::clone(&data))
            // middleware
            .wrap(actix_web::middleware::Logger::default())
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
            // docs
            .service(fs::Files::new("/api/docs", "./target/doc").show_files_listing())
            // POST api
            // POST auth
            .service(crate::api::auth::register)
            .service(crate::api::auth::login)
            .service(crate::api::auth::login_secondary_token)
            .service(crate::api::auth::edit_about_request)
            .service(crate::api::auth::refresh_secondary_token_request)
            .service(crate::api::auth::update_request)
            .service(crate::api::auth::follow_request)
            .service(crate::api::auth::ban_request)
            // POST api::pastes
            .service(crate::api::pastes::render_request)
            .service(crate::api::pastes::create_request)
            .service(crate::api::pastes::edit_request)
            .service(crate::api::pastes::edit_atomic_request)
            .service(crate::api::pastes::delete_request)
            .service(crate::api::pastes::metadata_request)
            // POST api::pastes SSM
            .service(crate::api::pastes::render_ssm_request)
            .service(crate::api::pastes::render_paste_ssm_request)
            // GET api
            .service(crate::api::pastes::get_from_url_request)
            .service(crate::api::pastes::get_from_id_request)
            .service(crate::api::pastes::exists_request)
            .service(crate::api::auth::logout)
            // GET dashboard
            .service(crate::pages::home::dashboard_request)
            .service(crate::pages::home::notifications_request)
            .service(crate::pages::auth::register_request)
            .service(crate::pages::auth::login_request)
            .service(crate::pages::auth::login_secondary_token_request)
            .service(crate::pages::settings::user_settings_request)
            .service(crate::pages::settings::paste_settings_request)
            .service(crate::pages::paste_view::dashboard_request)
            // GET dashboard (atomic pastes)
            .service(crate::pages::atomic_editor::dashboard_request)
            .service(crate::pages::atomic_editor::new_request)
            .service(crate::pages::atomic_editor::edit_request)
            // GET boards
            .service(crate::pages::boards::dashboard_request)
            .service(crate::pages::boards::search_by_tags_request)
            .service(crate::pages::boards::new_request)
            .service(crate::pages::boards::view_board_post_request)
            .service(crate::pages::boards::board_settings_request)
            .service(crate::pages::boards::create_board_post_request)
            .service(crate::pages::boards::view_board_request)
            // GET boards api
            .service(crate::api::boards::get_posts_request)
            .service(crate::api::boards::get_post_request)
            // POST boards api
            .service(crate::api::boards::create_request)
            .service(crate::api::boards::create_post_request)
            .service(crate::api::boards::update_post_request)
            .service(crate::api::boards::update_post_tags_request)
            .service(crate::api::boards::metadata_request)
            .service(crate::api::boards::pin_post_request)
            // DELETE boards api
            .service(crate::api::boards::delete_post_request)
            .service(crate::api::boards::delete_board_request)
            // GET staff
            .service(crate::pages::staff::dashboard_request)
            .service(crate::pages::staff::staff_boards_dashboard_request)
            .service(crate::pages::staff::staff_users_dashboard_request)
            // GET users
            .service(crate::pages::auth::followers_request)
            .service(crate::pages::auth::following_request)
            .service(crate::pages::auth::user_settings_request)
            .service(crate::pages::auth::profile_view_request)
            .service(crate::api::auth::avatar_request)
            .service(crate::api::auth::followers_request)
            .service(crate::api::auth::following_request)
            .service(crate::api::auth::level_request)
            .service(crate::api::auth::get_from_owner_request)
            // GET root
            .service(crate::pages::home::home_request)
            .service(crate::pages::home::robotstxt)
            .service(crate::pages::paste_view::atomic_paste_view_request)
            .service(crate::pages::paste_view::paste_view_request) // must be run last as it matches all other paths!
            // ERRORS
            .default_service(web::to(|| async {
                let renderer = ServerRenderer::<crate::pages::errors::_404Page>::new();

                return HttpResponse::NotFound().body(utility::format_html(
                    renderer.render().await,
                    "<title>404: Not Found</title>",
                ));
            }))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
