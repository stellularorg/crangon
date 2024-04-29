use crate::db::AppData;

use super::base;
use actix_web::{web, HttpRequest, HttpResponse};
use askama::Template;
use awc::http::StatusCode;

#[derive(Template)]
#[template(path = "general/404.html")]
struct Error404Template {
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    site_name: String,
    body_embed: String,
}

pub async fn error404(req: HttpRequest, data: web::Data<AppData>) -> HttpResponse {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::NotFound()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            Error404Template {
                // required fields
                info: base.info,
                auth_state: base.auth_state,
                guppy: base.guppy,
                site_name: base.site_name,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}

#[derive(Template)]
#[template(path = "general/401.html")]
struct Error401Template {
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    site_name: String,
    body_embed: String,
}

pub async fn error401(req: HttpRequest, data: web::Data<AppData>) -> HttpResponse {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::build(StatusCode::from_u16(401).unwrap())
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            Error401Template {
                // required fields
                info: base.info,
                auth_state: base.auth_state,
                guppy: base.guppy,
                site_name: base.site_name,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}
