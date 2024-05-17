use crate::db::{self, AppData};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};

#[derive(Default, PartialEq, serde::Deserialize)]
pub struct CallbackQueryProps {
    pub uid: Option<String>, // this uid will need to be sent to the client as a token
                             // the uid will also be sent to the client as a token on GUPPY_ROOT, meaning we'll have signed in here and there!
}

#[get("/api/v1/auth/callback")]
pub async fn callback_request(info: web::Query<CallbackQueryProps>) -> impl Responder {
    let set_cookie = if info.uid.is_some() {
        format!("__Secure-Token={}; SameSite=Lax; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age={}", info.uid.as_ref().unwrap(), 60 * 60 * 24 * 365)
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

    let res = data
        .db
        .get_user_by_unhashed(cookie.unwrap().value().to_string()) // if the user is returned, that means the ID is valid
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

#[get("/api/v1/auth/users/{name:.*?}/pastes")]
/// Get all pastes by owner
pub async fn get_from_owner_request(
    req: HttpRequest,
    data: web::Data<db::AppData>,
    info: web::Query<crate::api::pastes::OffsetQueryProps>,
) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // get pastes
    let res: db::DefaultReturn<Option<Vec<db::PasteIdentifier>>> =
        data.db.get_pastes_by_owner_limited(name, info.offset).await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(
            serde_json::to_string::<db::DefaultReturn<Option<Vec<db::PasteIdentifier>>>>(&res)
                .unwrap(),
        );
}

#[post("/api/v1/auth/users/{name:.*?}/ban")]
/// Ban user
pub async fn ban_request(req: HttpRequest, data: web::Data<db::AppData>) -> impl Responder {
    let name: String = req.match_info().get("name").unwrap().to_string();

    // get token user
    let token_cookie = req.cookie("__Secure-Token");
    let token_user = if token_cookie.is_some() {
        Option::Some(
            data.db
                .get_user_by_unhashed(token_cookie.as_ref().unwrap().value().to_string()) // if the user is returned, that means the ID is valid
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

    // make sure token_user is of role "staff"
    if !token_user
        .unwrap()
        .payload
        .unwrap()
        .level
        .permissions
        .contains(&String::from("ManageUsers"))
    {
        return HttpResponse::NotAcceptable().body("Only staff can do this");
    }

    // ban user
    let res: db::DefaultReturn<Option<String>> = data.db.ban_user_by_name(name).await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string::<db::DefaultReturn<Option<String>>>(&res).unwrap());
}
