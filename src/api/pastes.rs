use actix_web::{post, web, HttpResponse, Responder};

use crate::markdown;

#[derive(serde::Deserialize)]
struct RenderInfo {
    text: String,
}

#[post("/api/markdown")]
pub async fn render_request(body: web::Json<RenderInfo>) -> impl Responder {
    return HttpResponse::Ok().body(markdown::parse_markdown(&body.text));
}
