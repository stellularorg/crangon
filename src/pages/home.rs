use actix_web::HttpRequest;
use actix_web::{get, web, HttpResponse, Responder};

use super::base;
use crate::db::{self, AppData};
use askama::Template;

#[derive(Template)]
#[template(path = "general/homepage.html")]
struct HomeTemplate {
    edit_mode: bool,
    starting_content: String,
    editing: String,
    password_not_needed: bool,
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    site_name: String,
    body_embed: String,
}

#[derive(Default, PartialEq, serde::Deserialize)]
pub struct HomeQueryProps {
    pub editing: Option<String>,
}

#[derive(Template)]
#[template(path = "general/dashboard.html")]
struct DashboardTemplate {
    user: db::UserState<String>,
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    puffer: String,
    site_name: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "general/inbox.html")]
struct InboxTemplate {
    boards: Vec<db::BoardIdentifier>,
    offset: i32,
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    puffer: String,
    site_name: String,
    body_embed: String,
}

#[get("/")]
/// Available at "/"
pub async fn home_request(
    req: HttpRequest,
    data: web::Data<AppData>,
    info: web::Query<HomeQueryProps>,
) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    // ...
    let str: &Option<String> = &info.editing;

    let paste = if str.is_some() {
        Option::Some(data.db.get_paste_by_url(str.to_owned().unwrap()).await)
    } else {
        Option::None
    };

    let metadata = if paste.is_some() && paste.as_ref().unwrap().payload.is_some() {
        Option::Some(
            &paste
                .as_ref()
                .unwrap()
                .payload
                .as_ref()
                .unwrap()
                .paste
                .metadata,
        )
    } else {
        Option::None
    };

    // if metadata has "private_source" set to "on" and we're not the owner, return
    if metadata.is_some() {
        let owner = &metadata.as_ref().unwrap().owner;
        if metadata.as_ref().unwrap().private_source == String::from("on") {
            if token_user.is_none() {
                return HttpResponse::NotFound()
                    .body("You do not have permission to view this paste's contents.");
            }

            let payload = &token_user.as_ref().unwrap().payload;
            if owner.to_string() != payload.as_ref().unwrap().user.username {
                return HttpResponse::NotFound()
                    .body("You do not have permission to view this paste's contents.");
            }
        }
    };

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            HomeTemplate {
                edit_mode: if info.editing.is_some() { true } else { false },
                editing: if str.is_some() {
                    str.clone().unwrap()
                } else {
                    String::new()
                },
                starting_content: if paste.is_some() {
                    if paste.as_ref().unwrap().success {
                        paste
                            .as_ref()
                            .unwrap()
                            .payload
                            .as_ref()
                            .unwrap()
                            .paste
                            .content
                            .replace(r"`", "\\`")
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                },
                password_not_needed: if metadata.is_some() && token_user.is_some() {
                    metadata.unwrap().owner == token_user.unwrap().payload.unwrap().user.username
                } else {
                    false
                },
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

#[get("/robots.txt")]
/// Available at "/robots.txt"
pub async fn robotstxt() -> impl Responder {
    return HttpResponse::Ok().body(
        "User-agent: *
Allow: /
Disallow: /api
Disallow: /admin
Disallow: /paste
Disallow: /d/atomic
Disallow: /*?",
    );
}

#[get("/d")]
/// Available at "/d"
pub async fn dashboard_request(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        // you must have an account to use the user dashboard
        return HttpResponse::NotFound().body(
            "You must have an account to use the user dashboard.
You can login at: /d/auth/login
You can create an account at: /d/auth/register",
        );
    }

    // ...
    let base = base::get_base_values(token_user.is_some());
    let user = token_user.unwrap().payload.unwrap();

    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            DashboardTemplate {
                user: user.user,
                // required fields
                info: base.info,
                auth_state: base.auth_state,
                guppy: base.guppy,
                puffer: base.puffer,
                site_name: base.site_name,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}

#[get("/d/inbox")]
/// Available at "/d/inbox"
pub async fn inbox_request(
    req: HttpRequest,
    data: web::Data<AppData>,
    info: web::Query<crate::api::pastes::OffsetQueryProps>,
) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        // you must have an account to use the user dashboard
        return HttpResponse::NotFound().body(
            "You must have an account to use the user dashboard.
You can login at: /d/auth/login
You can create an account at: /d/auth/register",
        );
    }

    // get inboxes
    let user = token_user.as_ref().unwrap().payload.as_ref().unwrap();
    let boards_res = data
        .db
        .get_user_mail_streams(user.user.username.clone(), info.offset)
        .await;

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            InboxTemplate {
                boards: boards_res.payload,
                offset: if info.offset.is_some() {
                    info.offset.unwrap()
                } else {
                    0
                },
                // required fields
                info: base.info,
                auth_state: base.auth_state,
                guppy: base.guppy,
                puffer: base.puffer,
                site_name: base.site_name,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}
