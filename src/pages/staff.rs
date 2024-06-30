use actix_web::HttpRequest;
use actix_web::{get, web, HttpResponse, Responder};

use super::base;
use askama::Template;

use crate::db::{self, FullUser, PasteIdentifier};
use dorsal::db::special::auth_db::{AuthError, Result as AuthResult, UserMetadata};

#[derive(Template)]
#[template(path = "staff/homepage.html")]
struct HomeTemplate {
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    site_name: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "staff/users.html")]
struct UsersTemplate {
    user: AuthResult<FullUser<UserMetadata>>,
    username: String,
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    site_name: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "staff/pastes.html")]
struct PastesTemplate {
    pastes: Vec<PasteIdentifier>,
    search_content: String,
    offset: i32,
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    site_name: String,
    body_embed: String,
}

#[derive(Default, PartialEq, serde::Deserialize)]
pub struct UsersQueryProps {
    pub username: Option<String>,
}

#[derive(Default, PartialEq, serde::Deserialize)]
pub struct PastesQueryProps {
    pub offset: Option<i32>,
    pub search_content: Option<String>,
}

#[get("/dashboard/staff")]
/// Available at "/dashboard/staff"
pub async fn dashboard_request(req: HttpRequest, data: web::Data<db::AppData>) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        // you must have an account to use the staff dashboard
        return super::errors::error401(req, data).await;
    }

    // validate role
    let user = token_user.as_ref().unwrap();

    if !user
        .level
        .permissions
        .contains(&String::from("StaffDashboard"))
    {
        return HttpResponse::NotFound().body("You do not have permission to do this");
    }

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            HomeTemplate {
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

#[get("/dashboard/staff/users")]
/// Available at "/dashboard/staff/users"
pub async fn staff_users_dashboard_request(
    req: HttpRequest,
    data: web::Data<db::AppData>,
    info: web::Query<UsersQueryProps>,
) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        // you must have an account to use the staff dashboard
        return super::errors::error401(req, data).await;
    }

    // validate role
    let user = token_user.as_ref().unwrap();

    if !user
        .level
        .permissions
        .contains(&String::from("StaffDashboard"))
    {
        return HttpResponse::NotFound().body("You do not have permission to do this");
    }

    // get user
    let user = if info.username.is_some() {
        data.db
            .get_user_by_username(info.username.as_ref().unwrap().to_owned())
            .await
    } else {
        Err(AuthError::Other)
    };

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            UsersTemplate {
                username: if info.username.is_some() {
                    info.username.as_ref().unwrap().to_owned()
                } else {
                    String::new()
                },
                user,
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

#[get("/dashboard/staff/pastes")]
/// Available at "/dashboard/staff/pastes"
pub async fn staff_pastes_dashboard_request(
    req: HttpRequest,
    data: web::Data<db::AppData>,
    info: web::Query<PastesQueryProps>,
) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        // you must have an account to use the staff dashboard
        return super::errors::error401(req, data).await;
    }

    // validate role
    let user = token_user.as_ref().unwrap();

    if !user
        .level
        .permissions
        .contains(&String::from("StaffDashboard"))
    {
        return HttpResponse::NotFound().body("You do not have permission to do this");
    }

    // get pastes
    let pastes = if info.search_content.is_some() {
        data.db
            .get_all_pastes_by_content_limited(info.search_content.clone().unwrap(), info.offset)
            .await
    } else {
        data.db.get_all_pastes_limited(info.offset).await
    };

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            PastesTemplate {
                pastes: pastes.ok().unwrap(),
                search_content: info.search_content.clone().unwrap_or(String::new()),
                offset: if info.offset.is_some() {
                    info.offset.unwrap()
                } else {
                    0
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
