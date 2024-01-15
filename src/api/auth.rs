use actix_web::{post, web, HttpResponse, Responder};

use crate::{db::bundlesdb::AppData, utility};

#[derive(serde::Deserialize)]
struct RegisterInfo {
    username: String,
}

#[derive(serde::Deserialize)]
struct LoginInfo {
    uid: String,
}

#[post("/api/auth/register")]
pub async fn register(body: web::Json<RegisterInfo>, data: web::Data<AppData>) -> impl Responder {
    let username = &body.username.trim();
    let res = data.db.create_user(username.to_string()).await;

    let c = res.clone();
    let set_cookie = if res.success && res.payload.is_some() {
        format!("__Secure-Token={}; SameSite=Strict; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age={}", c.payload.unwrap(), 60 * 60 * 24 * 365)
    } else {
        String::new()
    };

    // return
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", if res.success { &set_cookie } else { "" }))
        .body(serde_json::to_string(&res).unwrap());
}

#[post("/api/auth/login")]
pub async fn login(body: web::Json<LoginInfo>, data: web::Data<AppData>) -> impl Responder {
    let id = body.uid.trim();
    let id_hashed = utility::hash(id.to_string());

    let res = data
        .db
        .get_user_by_hashed(id_hashed) // if the user is returned, that means the ID is valid
        .await;

    let c = res.clone();
    let set_cookie = if res.success && res.payload.is_some() {
        format!("__Secure-Token={}; SameSite=Strict; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age={}", c.payload.unwrap().id_hashed, 60 * 60 * 24 * 365)
    } else {
        String::new()
    };

    // return
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", if res.success { &set_cookie } else { "" }))
        .body(serde_json::to_string(&res).unwrap());
}
