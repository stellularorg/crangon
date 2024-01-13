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

use crate::db::bundlesdb::create_database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = config::collect_arguments();

    let port_search: Option<String> = config::get_named_argument(&args, "port");
    let mut port: u16 = 8080;

    if port_search.is_some() {
        port = port_search.unwrap().parse::<u16>().unwrap();
    }

    // create database
    create_database();

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
