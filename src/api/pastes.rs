use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};

use crate::db::{self, DefaultReturn, FullPaste, PasteMetadata};
use crate::{markdown, ssm, utility};

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

#[post("/api/markdown")]
pub async fn render_request(body: web::Json<RenderInfo>) -> impl Responder {
    return HttpResponse::Ok()
        .append_header(("Content-Type", "text/html"))
        .body(markdown::render::parse_markdown(&body.text));
}

#[post("/api/ssm")]
pub async fn render_ssm_request(body: web::Json<RenderInfo>) -> impl Responder {
    return HttpResponse::Ok()
        .append_header(("Content-Type", "text/css"))
        .body(ssm::parse_ssm_program(body.text.clone()));
}

#[get("/api/ssm/{url:.*}")]
pub async fn render_paste_ssm_request(
    req: HttpRequest,
    data: web::Data<db::AppData>,
) -> impl Responder {
    let custom_url: String = req.match_info().get("url").unwrap().to_string();
    let res = data.db.get_paste_by_url(custom_url).await;

    if !res.success {
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "application/json"))
            .body(
                serde_json::to_string::<DefaultReturn<Option<FullPaste<PasteMetadata, String>>>>(
                    &res,
                )
                .unwrap(),
            );
    }

    // make sure the paste allows their SSM content to be public
    if !res
        .payload
        .as_ref()
        .unwrap()
        .paste
        .content
        .contains("USE ssm::public")
    {
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "text/plain"))
            .body("This paste does not export any public SSM blocks.");
    }

    // return
    // TODO: check for "USE ssm::public" in the paste content before returning!
    return HttpResponse::Ok()
        .append_header(("Content-Type", "text/css"))
        .body(ssm::parse_ssm_blocks(res.payload.unwrap().paste.content));
}

#[post("/api/new")]
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
            &mut db::Paste {
                custom_url: custom_url.clone(),
                id: String::new(), // reassigned anyways, this doesn't matter
                edit_password: edit_password.to_string(),
                content: content.clone(),
                content_html: crate::markdown::render::parse_markdown(&content), // go ahead and render the content
                pub_date: utility::unix_epoch_timestamp(),
                edit_date: utility::unix_epoch_timestamp(),
                group_name: g_name_for_real.to_string(),
                metadata: String::new(), // will be filled automatically
                views: 0,
            },
            if token_user.is_some() {
                Option::Some(token_user.unwrap().payload.unwrap().user.username)
            } else {
                Option::None
            },
        )
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[post("/api/edit")]
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
    }

    // ...
    let res = data
        .db
        .edit_paste_by_url(
            custom_url,
            content,
            edit_password,
            new_url,
            new_edit_password,
            if token_user.is_some() {
                Option::Some(token_user.unwrap().payload.unwrap().user.username)
            } else {
                Option::None
            },
        )
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[post("/api/delete")]
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
    }

    // delete
    let res = data
        .db
        .delete_paste_by_url(
            custom_url,
            edit_password,
            if token_user.is_some() {
                Option::Some(token_user.unwrap().payload.unwrap().user.username)
            } else {
                Option::None
            },
        )
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[post("/api/metadata")]
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
    }

    // ...
    let res = data
        .db
        .edit_paste_metadata_by_url(
            custom_url,
            metadata,
            edit_password,
            if token_user.is_some() {
                Option::Some(token_user.unwrap().payload.unwrap().user.username)
            } else {
                Option::None
            },
        )
        .await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[get("/api/exists/{url:.*}")]
/// Check if a paste exists
pub async fn exists_request(req: HttpRequest, data: web::Data<db::AppData>) -> impl Responder {
    let custom_url: String = req.match_info().get("url").unwrap().to_string();
    let res = data.db.get_paste_by_url(custom_url).await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "text/plain"))
        .body(res.success.to_string());
}

#[get("/api/url/{url:.*}")]
/// Get paste by `custom_url`
pub async fn get_from_url_request(
    req: HttpRequest,
    data: web::Data<db::AppData>,
) -> impl Responder {
    let custom_url: String = req.match_info().get("url").unwrap().to_string();
    let res: DefaultReturn<Option<FullPaste<PasteMetadata, String>>> =
        data.db.get_paste_by_url(custom_url).await;

    // if res.metadata contains '"private_source":"on"', return NotFound
    if res.payload.is_some() && res.clone().payload.unwrap().paste.metadata.private_source == "on" {
        return HttpResponse::NotFound()
            .body("You do not have permission to view this paste's contents.");
    }

    // if res.metadata contains a view password, return fail
    if res.payload.is_some()
        && res
            .clone()
            .payload
            .unwrap()
            .paste
            .metadata
            .view_password
            .is_some()
    {
        return HttpResponse::NotFound()
            .body("You do not have permission to view this paste's contents.");
    }

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(
            serde_json::to_string::<DefaultReturn<Option<FullPaste<PasteMetadata, String>>>>(&res)
                .unwrap(),
        );
}

#[get("/api/id/{id:.*}")]
/// Get paste by ID
pub async fn get_from_id_request(req: HttpRequest, data: web::Data<db::AppData>) -> impl Responder {
    let id: String = req.match_info().get("id").unwrap().to_string();
    let res: DefaultReturn<Option<FullPaste<PasteMetadata, String>>> =
        data.db.get_paste_by_id(id).await;

    // if res.metadata contains '"private_source":"on"', return NotFound
    if res.payload.is_some() && res.clone().payload.unwrap().paste.metadata.private_source == "on" {
        return HttpResponse::NotFound()
            .body("You do not have permission to view this paste's contents.");
    }

    // if res.metadata contains a view password, return fail
    if res.payload.is_some()
        && res
            .clone()
            .payload
            .unwrap()
            .paste
            .metadata
            .view_password
            .is_some()
    {
        return HttpResponse::NotFound()
            .body("You do not have permission to view this paste's contents.");
    }

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(
            serde_json::to_string::<DefaultReturn<Option<FullPaste<PasteMetadata, String>>>>(&res)
                .unwrap(),
        );
}

// atomic "CRUD" operations
#[get("/api/atomic/crud/{url:.*}/{path:.*}")]
/// Read an atomic paste's "file system"
pub async fn read_atomic_request(req: HttpRequest, data: web::Data<db::AppData>) -> impl Responder {
    let custom_url: String = req.match_info().get("url").unwrap().to_string();
    let path: String = format!("/{}", req.match_info().get("path").unwrap());

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
    }

    // get paste
    let paste: DefaultReturn<Option<FullPaste<PasteMetadata, String>>> =
        data.db.get_paste_by_url(custom_url.clone()).await;

    if paste.success == false {
        return HttpResponse::Ok()
            .append_header(("Content-Type", "application/json"))
            .body(serde_json::to_string(&paste).unwrap());
    }

    // make sure paste is an atomic paste
    let unwrap = paste.payload.unwrap();
    let is_atomic = unwrap.paste.content.contains("\"_is_atomic\":true");

    if is_atomic == false {
        return HttpResponse::NotFound().body("Paste is not atomic");
    }

    // get file from path
    let real_content = serde_json::from_str::<db::AtomicPaste>(&unwrap.paste.content);

    if real_content.is_err() {
        return HttpResponse::NotAcceptable().body("Paste failed to deserialize");
    }

    let decoded = real_content.unwrap();

    // check for existing file in atomic paste fs
    let existing = decoded.files.iter().find(|f| f.path == path);

    if existing.is_none() {
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "text/plain"))
            .body("Path does not exist");
    }

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "text/plain"))
        .body(existing.unwrap().content.clone());
}
