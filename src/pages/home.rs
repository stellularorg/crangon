use actix_web::HttpRequest;
use actix_web::{get, web, HttpResponse, Responder};

use super::base;
use crate::db::{self, AppData};
use askama::Template;

#[derive(Template)]
#[template(path = "user/homepage.html")]
struct HomeTemplate {
    edit_mode: bool,
    starting_content: String,
    editing: String,
    password_not_needed: bool,
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    deducktive: String,
    site_name: String,
    body_embed: String,
}

#[derive(Default, PartialEq, serde::Deserialize)]
pub struct HomeQueryProps {
    pub editing: Option<String>,
}

#[derive(Template)]
#[template(path = "user/dashboard.html")]
struct DashboardTemplate {
    user: db::UserState<db::UserMetadata>,
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
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

    let paste = match str {
        Some(ref str) => Option::Some(data.db.get_paste_by_url(str.to_owned()).await),
        None => Option::None,
    };

    let metadata = match paste.as_ref() {
        Some(p) => match p {
            Ok(ref p) => Some(p.paste.metadata.to_owned()),
            Err(_) => None,
        },
        None => None,
    };

    // if metadata has "private_source" set to "on" and we're not the owner, return
    if metadata.is_some() {
        let owner = &metadata.as_ref().unwrap().owner;
        if metadata.as_ref().unwrap().private_source == String::from("on") {
            match token_user {
                Some(ref ua) => {
                    if owner.to_string() != ua.user.username {
                        return HttpResponse::NotFound()
                            .body("You do not have permission to view this paste's contents.");
                    }
                }
                None => {
                    return HttpResponse::NotFound()
                        .body("You do not have permission to view this paste's contents.")
                }
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
                starting_content: match paste {
                    Some(p) => match p {
                        Ok(p) => p
                            .paste
                            .content
                            .replace("\\", "\\\\")
                            .replace("`", "\\`")
                            .replace("$", "\\$")
                            .replace("/", "\\/"),
                        Err(_) => String::new(),
                    },
                    None => String::new(),
                },
                password_not_needed: if metadata.is_some() {
                    let metadata = metadata.unwrap();
                    match token_user {
                        Some(ua) => {
                            let username = ua.user.username;
                            let in_permissions_list = metadata.permissions_list.get(&username);

                            // MUST be paste owner
                            if metadata.owner == username {
                                true
                            }
                            // OR have a passwordless permission
                            else if in_permissions_list.is_some() {
                                let permission = in_permissions_list.unwrap();
                                // OR must have EditTextPasswordless or Passwordless
                                (permission == &db::PastePermissionLevel::EditTextPasswordless)
                                    | (permission == &db::PastePermissionLevel::Passwordless)
                            } else {
                                false
                            }
                        }
                        None => false,
                    }
                } else {
                    false
                },
                // required fields
                info: base.info,
                auth_state: base.auth_state,
                guppy: base.guppy,
                deducktive: base.deducktive,
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
    return HttpResponse::Ok()
        .append_header(("Content-Type", "text/plain"))
        .body(
            "User-agent: *
Allow: /
Disallow: /api
Disallow: /admin
Disallow: /paste
Disallow: /dashboard
Disallow: /*?",
        );
}

#[get("/ads.txt")]
/// Available at "/ads.txt"
pub async fn adstxt() -> impl Responder {
    return HttpResponse::Ok()
        .append_header(("Content-Type", "text/plain"))
        .body(std::env::var("ADS_TXT").unwrap_or(String::new()));
}

#[get("/dashboard")]
/// Available at "/dashboard"
pub async fn dashboard_request(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        // you must have an account to use the user dashboard
        return super::errors::error401(req, data).await;
    }

    // ...
    let base = base::get_base_values(token_user.is_some());
    let user = token_user.unwrap();

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
                site_name: base.site_name,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}
