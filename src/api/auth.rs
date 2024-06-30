use crate::db::{self, AppData};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use dorsal::DefaultReturn;

#[derive(Default, PartialEq, serde::Deserialize)]
pub struct CallbackQueryProps {
    pub uid: Option<String>, // this uid will need to be sent to the client as a token
                             // the uid will also be sent to the client as a token on GUPPY_ROOT, meaning we'll have signed in here and there!
}

#[get("/api/v1/auth/callback")]
pub async fn callback_request(info: web::Query<CallbackQueryProps>) -> impl Responder {
    let set_cookie = if info.uid.is_some() {
        format!(
            "__Secure-Token={}; SameSite=Lax; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age={}",
            info.uid.as_ref().unwrap(),
            60 * 60 * 24 * 365
        )
    } else {
        String::new()
    };

    // return
    return HttpResponse::Ok()
        .append_header((
            "Set-Cookie",
            if info.uid.is_some() { &set_cookie } else { "" },
        ))
        .append_header(("Content-Type", "text/html"))
        .body(
            "<head>
                <meta http-equiv=\"Refresh\" content=\"0; URL=/dashboard\" />
            </head>",
        );
}

#[get("/api/v1/auth/logout")]
pub async fn logout(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let cookie = req.cookie("__Secure-Token");

    if cookie.is_none() {
        return HttpResponse::NotAcceptable().body("Missing token");
    }

    if let Err(e) = data
        .db
        .get_user_by_unhashed(
            cookie
                .unwrap()
                .value()
                .to_string()
                .replace("__Secure-Token=", ""),
        ) // if the user is returned, that means the ID is valid
        .await
    {
        return HttpResponse::NotAcceptable().body(e.to_string());
    };

    // return
    return HttpResponse::Ok()
        .append_header((
            "Set-Cookie",
            "__Secure-Token=refresh; SameSite=Strict; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age=0",
        ))
        .append_header(("Content-Type", "text/plain"))
        .body("You have been signed out. You can now close this tab.");
}

#[get("/api/v1/auth/whoami")]
pub async fn whoami(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let cookie = req.cookie("__Secure-Token");

    if cookie.is_none() {
        // just return nothing on error
        return HttpResponse::Ok().body("");
    }

    match data
        .db
        .get_user_by_unhashed(
            cookie
                .unwrap()
                .value()
                .to_string()
                .replace("__Secure-Token=", ""),
        ) // if the user is returned, that means the ID is valid
        .await
    {
        Ok(ua) => HttpResponse::Ok()
            .append_header(("Content-Type", "text/plain"))
            .body(ua.user.username),
        Err(_) => HttpResponse::Ok()
            .append_header(("Content-Type", "text/plain"))
            .body(String::new()),
    }
}

#[get("/api/v1/auth/users/{name:.*?}/pastes")]
/// Get all pastes by owner
pub async fn get_from_owner_request(
    req: HttpRequest,
    data: web::Data<db::AppData>,
    info: web::Query<crate::api::pastes::OffsetQueryProps>,
) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // return
    match data.db.get_pastes_by_owner_limited(name, info.offset).await {
        Ok(p) => HttpResponse::Ok()
            .append_header(("Content-Type", "application/json"))
            .body(
                serde_json::to_string(&DefaultReturn {
                    success: true,
                    message: String::new(),
                    payload: p,
                })
                .unwrap(),
            ),
        Err(e) => HttpResponse::BadRequest()
            .append_header(("Content-Type", "application/json"))
            .body(serde_json::to_string::<DefaultReturn<()>>(&e.into()).unwrap()),
    }
}

#[post("/api/v1/auth/users/{name:.*?}/ban")]
/// Ban user
pub async fn ban_request(req: HttpRequest, data: web::Data<db::AppData>) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // get token user
    let token_cookie = req.cookie("__Secure-Token");
    let token_user = match token_cookie {
        Some(c) => match data
            .db
            .auth
            .get_user_by_unhashed(c.to_string().replace("__Secure-Token=", ""))
            .await
        {
            Ok(ua) => Some(ua),
            Err(_) => None,
        },
        None => None,
    };

    if token_user.is_none() {
        return HttpResponse::NotAcceptable().body("An account is required to do this.");
    }

    // make sure token_user is of role "staff"
    if !token_user
        .unwrap()
        .level
        .permissions
        .contains(&String::from("ManageUsers"))
    {
        return HttpResponse::NotAcceptable().body("Only staff can do this");
    }

    // return
    match data.db.ban_user_by_name(name).await {
        Ok(r) => HttpResponse::Ok()
            .append_header(("Content-Type", "application/json"))
            .body(
                serde_json::to_string::<DefaultReturn<()>>(&DefaultReturn {
                    success: true,
                    message: String::from("User banned"),
                    payload: r,
                })
                .unwrap(),
            ),
        Err(e) => HttpResponse::NotAcceptable()
            .append_header(("Content-Type", "application/json"))
            .body(serde_json::to_string::<DefaultReturn<()>>(&e.into()).unwrap()),
    }
}
