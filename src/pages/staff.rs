use actix_web::HttpRequest;
use actix_web::{get, web, HttpResponse, Responder};

use super::base;
use askama::Template;

use crate::db::{self, DefaultReturn, FullUser};

use crate::api::pastes::OffsetQueryProps;

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
#[template(path = "staff/posts.html")]
struct PostsTemplate {
    offset: i32,
    posts: Vec<db::Log>,
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    puffer: String,
    site_name: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "staff/users.html")]
struct UsersTemplate {
    user: Option<FullUser<String>>,
    username: String,
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

#[get("/d/staff")]
/// Available at "/d/staff"
pub async fn dashboard_request(req: HttpRequest, data: web::Data<db::AppData>) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        // you must have an account to use the staff dashboard
        return HttpResponse::NotFound().body(
            "You must have an account to use the staff dashboard.
You can login at: /d/auth/login
You can create an account at: /d/auth/register",
        );
    }

    // validate role
    let user = token_user.as_ref().unwrap().payload.as_ref().unwrap();

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

#[get("/d/staff/boards")]
/// Available at "/d/staff/boards"
pub async fn staff_boards_dashboard_request(
    req: HttpRequest,
    data: web::Data<db::AppData>,
    info: web::Query<OffsetQueryProps>,
) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        // you must have an account to use the staff dashboard
        return HttpResponse::NotFound().body(
            "You must have an account to use the staff dashboard.
You can login at: /d/auth/login
You can create an account at: /d/auth/register",
        );
    }

    // validate role
    let user = token_user.as_ref().unwrap().payload.as_ref().unwrap();

    if !user
        .level
        .permissions
        .contains(&String::from("StaffDashboard"))
    {
        return HttpResponse::NotFound().body("You do not have permission to do this");
    }

    // get posts
    let posts: db::DefaultReturn<Option<Vec<db::Log>>> =
        data.db.fetch_most_recent_posts(info.offset).await;

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            PostsTemplate {
                offset: if info.offset.is_some() {
                    info.offset.unwrap()
                } else {
                    0
                },
                posts: posts.payload.unwrap(),
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

#[get("/d/staff/users")]
/// Available at "/d/staff/users"
pub async fn staff_users_dashboard_request(
    req: HttpRequest,
    data: web::Data<db::AppData>,
    info: web::Query<UsersQueryProps>,
) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        // you must have an account to use the staff dashboard
        return HttpResponse::NotFound().body(
            "You must have an account to use the staff dashboard.
You can login at: /d/auth/login
You can create an account at: /d/auth/register",
        );
    }

    // validate role
    let user = token_user.as_ref().unwrap().payload.as_ref().unwrap();

    if !user
        .level
        .permissions
        .contains(&String::from("StaffDashboard"))
    {
        return HttpResponse::NotFound().body("You do not have permission to do this");
    }

    // get user
    let user: db::DefaultReturn<Option<FullUser<String>>> = if info.username.is_some() {
        data.db
            .get_user_by_username(info.username.as_ref().unwrap().to_owned())
            .await
    } else {
        DefaultReturn {
            success: false,
            message: String::new(),
            payload: Option::None,
        }
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
                user: user.payload,
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
