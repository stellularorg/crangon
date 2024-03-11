use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};

use crate::{
    db::bundlesdb::{AppData, DefaultReturn, UserFollow, UserMetadata, UserState},
    utility,
};

#[derive(serde::Deserialize)]
struct RegisterInfo {
    username: String,
}

#[derive(serde::Deserialize)]
struct LoginInfo {
    uid: String,
}

#[derive(serde::Deserialize)]
struct UpdateAboutInfo {
    about: String,
}

#[post("/api/auth/register")]
pub async fn register(body: web::Json<RegisterInfo>, data: web::Data<AppData>) -> impl Responder {
    // if server disabled registration, return
    let disabled = crate::config::get_var("REGISTRATION_DISABLED");

    if disabled.is_some() {
        return HttpResponse::NotAcceptable()
            .body("This server requires has registration disabled");
    }

    // ...
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
        .append_header(("Content-Type", "application/json"))
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
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[get("/api/auth/logout")]
pub async fn logout(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let cookie = req.cookie("__Secure-Token");

    if cookie.is_none() {
        return HttpResponse::NotAcceptable().body("Missing token");
    }

    let res = data
        .db
        .get_user_by_hashed(cookie.unwrap().value().to_string()) // if the user is returned, that means the ID is valid
        .await;

    if !res.success {
        return HttpResponse::NotAcceptable().body("Invalid token");
    }

    // return
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", "__Secure-Token=refresh; SameSite=Strict; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age=0"))
        .append_header(("Content-Type", "text/plain"))
        .body("You have been signed out. You can now close this tab.");
}

#[post("/api/auth/users/{name:.*}/about")]
pub async fn edit_about_request(
    req: HttpRequest,
    body: web::Json<UpdateAboutInfo>,
    data: web::Data<AppData>,
) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // get owner
    let token_cookie = req.cookie("__Secure-Token");
    let token_user = if token_cookie.is_some() {
        Option::Some(
            data.db
                .get_user_by_hashed(token_cookie.as_ref().unwrap().value().to_string()) // if the user is returned, that means the ID is valid
                .await,
        )
    } else {
        Option::None
    };

    if token_user.is_some() {
        // make sure user exists
        if token_user.as_ref().unwrap().success == false {
            return HttpResponse::NotFound().body("Invalid token");
        }
    } else {
        return HttpResponse::NotAcceptable().body("An account is required to do this");
    }

    let token_user = token_user.unwrap().payload.unwrap();

    // make sure profile exists
    let profile: DefaultReturn<Option<UserState<String>>> =
        data.db.get_user_by_username(name.to_owned()).await;

    if !profile.success {
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "application/json"))
            .body(
                serde_json::to_string::<DefaultReturn<Option<String>>>(&DefaultReturn {
                    success: false,
                    message: String::from("Profile does not exist!"),
                    payload: Option::None,
                })
                .unwrap(),
            );
    }

    let profile = profile.payload.unwrap();
    let mut user = serde_json::from_str::<UserMetadata>(&profile.metadata).unwrap();

    // check if we can update this user
    // must be authenticated AND same user OR staff
    let can_update: bool =
        (token_user.username == profile.username) | (token_user.role == String::from("staff"));

    if can_update == false {
        return HttpResponse::NotFound()
            .body("You do not have permission to manage this user's contents.");
    }

    // (check length)
    if (body.about.len() < 2) | (body.about.len() > 200_000) {
        return HttpResponse::Ok()
            .append_header(("Content-Type", "application/json"))
            .body(
                serde_json::to_string::<DefaultReturn<Option<String>>>(&DefaultReturn {
                    success: false,
                    message: String::from("Content is invalid"),
                    payload: Option::None,
                })
                .unwrap(),
            );
    }

    // update about
    user.about = body.about.clone();

    // ...
    let res = data.db.edit_user_metadata_by_name(name, user).await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[post("/api/auth/users/{name:.*}/follow")]
pub async fn follow_request(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // get owner
    let token_cookie = req.cookie("__Secure-Token");
    let token_user = if token_cookie.is_some() {
        Option::Some(
            data.db
                .get_user_by_hashed(token_cookie.as_ref().unwrap().value().to_string()) // if the user is returned, that means the ID is valid
                .await,
        )
    } else {
        Option::None
    };

    if token_user.is_some() {
        // make sure user exists
        if token_user.as_ref().unwrap().success == false {
            return HttpResponse::NotFound().body("Invalid token");
        }
    } else {
        return HttpResponse::NotAcceptable().body("An account is required to do this");
    }

    let token_user = token_user.unwrap().payload.unwrap();

    // ...
    let res = data
        .db
        .toggle_user_follow(&mut UserFollow {
            user: token_user.username,
            is_following: name,
        })
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[post("/api/auth/users/{name:.*}/update")]
pub async fn update_request(
    req: HttpRequest,
    body: web::Json<UserMetadata>,
    data: web::Data<AppData>,
) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // get owner
    let token_cookie = req.cookie("__Secure-Token");
    let token_user = if token_cookie.is_some() {
        Option::Some(
            data.db
                .get_user_by_hashed(token_cookie.as_ref().unwrap().value().to_string()) // if the user is returned, that means the ID is valid
                .await,
        )
    } else {
        Option::None
    };

    if token_user.is_some() {
        // make sure user exists
        if token_user.as_ref().unwrap().success == false {
            return HttpResponse::NotFound().body("Invalid token");
        }
    } else {
        return HttpResponse::NotAcceptable().body("An account is required to do this");
    }

    // make sure profile exists
    let profile: DefaultReturn<Option<UserState<String>>> =
        data.db.get_user_by_username(name.to_owned()).await;

    if !profile.success {
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "application/json"))
            .body(
                serde_json::to_string::<DefaultReturn<Option<String>>>(&DefaultReturn {
                    success: false,
                    message: String::from("Profile does not exist!"),
                    payload: Option::None,
                })
                .unwrap(),
            );
    }

    let token_user = token_user.unwrap().payload.unwrap();
    let profile = profile.payload.unwrap();

    // check if we can update this user
    // must be authenticated AND same user OR staff
    let can_update: bool =
        (token_user.username == profile.username) | (token_user.role == String::from("staff"));

    if can_update == false {
        return HttpResponse::NotFound()
            .body("You do not have permission to manage this user's contents.");
    }

    // ...
    let res = data
        .db
        .edit_user_metadata_by_name(
            name,            // select user
            body.to_owned(), // new metadata
        )
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[get("/api/auth/users/{name:.*}/avatar")]
pub async fn avatar_request(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // make sure profile exists
    let profile: DefaultReturn<Option<UserState<String>>> =
        data.db.get_user_by_username(name.to_owned()).await;

    if !profile.success {
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "application/json"))
            .body(
                serde_json::to_string::<DefaultReturn<Option<String>>>(&DefaultReturn {
                    success: false,
                    message: String::from("Profile does not exist!"),
                    payload: Option::None,
                })
                .unwrap(),
            );
    }

    let profile = profile.payload.unwrap();
    let user = serde_json::from_str::<UserMetadata>(&profile.metadata).unwrap();

    if user.avatar_url.is_none() {
        return HttpResponse::NotFound().body("User does not have an avatar set");
    }

    let avatar = user.avatar_url.unwrap();

    // fetch avatar
    let res = data
        .http_client
        .get(avatar)
        .timeout(std::time::Duration::from_millis(5_000))
        .insert_header(("User-Agent", "stellular-bundlrs/1.0"))
        .send()
        .await;

    if res.is_err() {
        return HttpResponse::NotFound().body("Failed to fetch avatar on server");
    }

    // ...
    let mut res = res.unwrap();
    let body = res.body().limit(20_000_000).await;

    if body.is_err() {
        return HttpResponse::NotFound()
            .body("Failed to fetch avatar on server (image likely too large)");
    }

    let body = body.unwrap();

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", res.content_type()))
        .body(body);
}
