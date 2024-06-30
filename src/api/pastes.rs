use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};

use crate::db;
use crate::markdown;
use crate::model::DatabaseError;
use dorsal::{utility, DefaultReturn};

#[derive(Default, PartialEq, serde::Deserialize)]
pub struct OffsetQueryProps {
    pub offset: Option<i32>,
}

#[derive(serde::Deserialize)]
struct RenderInfo {
    text: String,
}

#[derive(serde::Deserialize)]
struct CreateInfo {
    custom_url: String,
    content: String,
    edit_password: String,
    group_name: Option<String>,
}

#[derive(serde::Deserialize)]
struct EditInfo {
    custom_url: String,
    content: String,
    edit_password: String,
    new_edit_password: Option<String>,
    new_custom_url: Option<String>,
}

#[derive(serde::Deserialize)]
struct DeleteInfo {
    custom_url: String,
    edit_password: String,
}

#[derive(serde::Deserialize)]
struct MetadataInfo {
    custom_url: String,
    edit_password: String,
    metadata: db::PasteMetadata,
}

#[post("/api/v1/markdown")]
pub async fn render_request(body: web::Json<RenderInfo>) -> impl Responder {
    return HttpResponse::Ok()
        .append_header(("Content-Type", "text/html"))
        .body(markdown::parse_markdown(body.text.clone()));
}

#[post("/api/v1/new")]
/// Create a new paste (`create_paste`)
pub async fn create_request(
    req: HttpRequest,
    body: web::Json<CreateInfo>,
    data: web::Data<db::AppData>,
) -> impl Responder {
    let custom_url: String = body.custom_url.trim().to_string();
    let edit_password: &String = &body.edit_password;
    let content: String = body.content.trim().to_string();

    let group_name: &Option<String> = &body.group_name;
    let g_name_for_real: &str = if group_name.is_some() {
        group_name.as_ref().unwrap()
    } else {
        ""
    };

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
        // if server requires an account, return
        let requires_account = crate::config::get_var("AUTH_REQUIRED");

        if requires_account.is_some() {
            return HttpResponse::NotAcceptable()
                .body("This server requires an account to create pastes");
        }
    }

    // create paste
    let res = data
        .db
        .create_paste(
            db::Paste {
                custom_url: custom_url.clone(),
                id: String::new(), // reassigned anyways, this doesn't matter
                edit_password: edit_password.to_string(),
                content: content.clone(),
                content_html: crate::markdown::parse_markdown(content), // go ahead and render the content
                pub_date: utility::unix_epoch_timestamp(),
                edit_date: utility::unix_epoch_timestamp(),
                group_name: g_name_for_real.to_string(),
                metadata: String::new(), // will be filled automatically
                views: 0,
            },
            if token_user.is_some() {
                Some(token_user.unwrap().user.username)
            } else {
                None
            },
        )
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(
            serde_json::to_string(&match res {
                Ok(r) => DefaultReturn {
                    success: true,
                    message: r.edit_password.clone(),
                    payload: r,
                },
                Err(e) => e.into(),
            })
            .unwrap(),
        );
}

#[post("/api/v1/edit")]
/// Edit a paste
pub async fn edit_request(
    req: HttpRequest,
    body: web::Json<EditInfo>,
    data: web::Data<db::AppData>,
) -> impl Responder {
    let custom_url: String = body.custom_url.trim().to_string();
    let content: String = body.content.trim().to_string();
    let edit_password: String = body.edit_password.to_owned();
    let new_url: Option<String> = body.new_custom_url.to_owned();
    let new_edit_password: Option<String> = body.new_edit_password.to_owned();

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

    // ...
    let res = data
        .db
        .edit_paste_by_url(
            custom_url.clone(),
            content,
            edit_password,
            new_url,
            new_edit_password,
            if token_user.is_some() {
                Some(token_user.unwrap().user.username)
            } else {
                None
            },
        )
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(
            serde_json::to_string(&match res {
                Ok(_) => DefaultReturn {
                    success: true,
                    message: String::from("Paste edited"),
                    payload: custom_url,
                },
                Err(e) => e.into(),
            })
            .unwrap(),
        );
}

#[post("/api/v1/delete")]
/// Delete a paste
pub async fn delete_request(
    req: HttpRequest,
    body: web::Json<DeleteInfo>,
    data: web::Data<db::AppData>,
) -> impl Responder {
    let custom_url: String = body.custom_url.trim().to_string();
    let edit_password: String = body.edit_password.to_owned();

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

    // delete
    let res = data
        .db
        .delete_paste_by_url(
            custom_url,
            edit_password,
            if token_user.is_some() {
                Option::Some(token_user.unwrap().user.username)
            } else {
                Option::None
            },
        )
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(
            serde_json::to_string(&match res {
                Ok(_) => DefaultReturn {
                    success: true,
                    message: String::from("Paste deleted"),
                    payload: true,
                },
                Err(e) => e.into(),
            })
            .unwrap(),
        );
}

#[post("/api/v1/metadata")]
/// Edit paste metadata (`edit_paste_metadata_by_url`)
pub async fn metadata_request(
    req: HttpRequest,
    body: web::Json<MetadataInfo>,
    data: web::Data<db::AppData>,
) -> impl Responder {
    let custom_url: String = body.custom_url.trim().to_string();
    let edit_password: String = body.edit_password.to_owned();

    let m = body.metadata.to_owned();
    let metadata: db::PasteMetadata = m;

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

    // ...
    let res = data
        .db
        .edit_paste_metadata_by_url(
            custom_url,
            metadata,
            edit_password,
            if token_user.is_some() {
                Option::Some(token_user.unwrap().user.username)
            } else {
                Option::None
            },
        )
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(
            serde_json::to_string(&match res {
                Ok(_) => DefaultReturn {
                    success: true,
                    message: String::from("Paste edited"),
                    payload: true,
                },
                Err(e) => e.into(),
            })
            .unwrap(),
        );
}

#[get("/api/v1/exists/{url:.*}")]
/// Check if a paste exists
pub async fn exists_request(req: HttpRequest, data: web::Data<db::AppData>) -> impl Responder {
    let custom_url: String = req.match_info().get("url").unwrap().to_string();
    let res = data.db.get_paste_by_url(custom_url).await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "text/plain"))
        .body(res.is_ok().to_string());
}

#[get("/api/v1/url/{url:.*}")]
/// Get paste by `custom_url`
pub async fn get_from_url_request(
    req: HttpRequest,
    data: web::Data<db::AppData>,
) -> impl Responder {
    let custom_url: String = req.match_info().get("url").unwrap().to_string();
    let res = data.db.get_paste_by_url(custom_url).await;

    if let Ok(ref r) = res {
        if r.paste.metadata.private_source == "on" {
            return HttpResponse::NotFound().body(DatabaseError::NotAllowed.to_string());
        }

        if r.paste.metadata.view_password.is_some() {
            return HttpResponse::NotFound().body(DatabaseError::NotAllowed.to_string());
        }
    }

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(
            serde_json::to_string(&match res {
                Ok(r) => DefaultReturn {
                    success: true,
                    message: String::from("Paste exists"),
                    payload: Some(r),
                },
                Err(e) => e.into(),
            })
            .unwrap(),
        );
}

#[get("/api/v1/id/{id:.*}")]
/// Get paste by ID
pub async fn get_from_id_request(req: HttpRequest, data: web::Data<db::AppData>) -> impl Responder {
    let id: String = req.match_info().get("id").unwrap().to_string();
    let res = data.db.get_paste_by_id(id).await;

    if let Ok(ref r) = res {
        if r.paste.metadata.private_source == "on" {
            return HttpResponse::NotFound().body(DatabaseError::NotAllowed.to_string());
        }

        if r.paste.metadata.view_password.is_some() {
            return HttpResponse::NotFound().body(DatabaseError::NotAllowed.to_string());
        }
    }

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(
            serde_json::to_string(&match res {
                Ok(r) => DefaultReturn {
                    success: true,
                    message: String::from("Paste exists"),
                    payload: Some(r),
                },
                Err(e) => e.into(),
            })
            .unwrap(),
        );
}

#[post("/api/v1/paste/{id:.*}/favorite")]
/// Toggle a paste favorite
pub async fn favorite_request(req: HttpRequest, data: web::Data<db::AppData>) -> impl Responder {
    let paste_id = req.match_info().get("id").unwrap();

    // verify auth status
    let (set_cookie, _, token_user) =
        crate::pages::base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        return HttpResponse::NotAcceptable().body(DatabaseError::NotAllowed.to_string());
    }

    // ...
    let res = data
        .db
        .toggle_user_paste_favorite(token_user.unwrap().user.username, paste_id.to_string())
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .append_header(("Set-Cookie", set_cookie))
        .body(
            serde_json::to_string(&match res {
                Ok(_) => DefaultReturn {
                    success: true,
                    message: String::from("Paste favorite toggled"),
                    payload: true,
                },
                Err(e) => e.into(),
            })
            .unwrap(),
        );
}
