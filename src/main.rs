use actix_files as fs;
use actix_web::{App, HttpServer};

mod config;
mod utility;

mod components;
mod pages;

mod markdown;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = config::collect_arguments();

    let port_search: Option<String> = config::get_named_argument(&args, "port");
    let mut port: u16 = 8080;

    if port_search.is_some() {
        port = port_search.unwrap().parse::<u16>().unwrap();
    }

    println!(
        "{}",
        markdown::parse_markdown("Hello, <friend>! <style>test</style> &!ast;".to_string())
    );

    // start server
    println!("Starting server at: http://localhost:{port}");
    HttpServer::new(|| {
        App::new()
            // static dir
            .service(fs::Files::new("/static", "./static").show_files_listing())
            // GET root
            .service(crate::pages::home::home_request)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
