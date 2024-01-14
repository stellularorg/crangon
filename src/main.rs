use actix_files as fs;
use actix_web::HttpResponse;
use actix_web::{web, App, HttpServer};

use yew::ServerRenderer;

mod config;
mod db;
mod utility;

mod components;
mod pages;

mod markdown;

use crate::db::bundlesdb::BundlesDB;
use crate::db::sql::DatabaseOpts;

use sqlx;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = config::collect_arguments();

    let port_search: Option<String> = config::get_named_argument(&args, "port");
    let mut port: u16 = 8080;

    if port_search.is_some() {
        port = port_search.unwrap().parse::<u16>().unwrap();
    }

    // create database
    let db_type: Option<String> = config::get_named_argument(&args, "db-type");
    let db_host: Option<String> = config::get_named_argument(&args, "db-host");
    let db_user: Option<String> = config::get_named_argument(&args, "db-user");
    let db_pass: Option<String> = config::get_named_argument(&args, "db-pass");
    let db_name: Option<String> = config::get_named_argument(&args, "db-name");

    let db_is_psql: bool = db_type
        .clone()
        .is_some_and(|x| x == String::from("postgres"));

    if db_is_psql && (db_user.is_none() | db_pass.is_none() | db_name.is_none()) {
        panic!("Missing required database config settings!");
    }

    sqlx::any::install_default_drivers(); // install database drivers
    let mut db: BundlesDB = BundlesDB::new(DatabaseOpts {
        _type: db_type,
        host: db_host,
        user: if db_is_psql {
            db_user.unwrap()
        } else {
            String::new()
        },
        pass: if db_is_psql {
            db_pass.unwrap()
        } else {
            String::new()
        },
        name: if db_is_psql {
            db_name.unwrap()
        } else {
            String::new()
        },
    })
    .await;

    db.init().await;

    // start server
    println!("Starting server at: http://localhost:{port}");
    HttpServer::new(|| {
        App::new()
            // static dir
            .service(fs::Files::new("/static", "./static").show_files_listing())
            // GET root
            .service(crate::pages::home::home_request)
            .service(crate::pages::paste_view::paste_view_request) // must be run last as it matches all other paths!
            // ERRORS
            .default_service(web::to(|| async {
                let renderer = ServerRenderer::<crate::pages::errors::_404Page>::new();

                return HttpResponse::NotFound()
                    .body(utility::format_html(renderer.render().await));
            }))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
