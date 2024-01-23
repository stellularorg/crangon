use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};

use crate::db::bundlesdb;
use crate::{markdown, ssm, utility};

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
    metadata: bundlesdb::PasteMetadata,
}

#[post("/api/markdown")]
pub async fn render_request(body: web::Json<RenderInfo>) -> impl Responder {
    return HttpResponse::Ok()
        .append_header(("Content-Type", "text/html"))
        .body(markdown::parse_markdown(&body.text));
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
    data: web::Data<bundlesdb::AppData>,
) -> impl Responder {
    let custom_url: String = req.match_info().get("url").unwrap().to_string();
    let res = data.db.get_paste_by_url(custom_url).await;

    if !res.success {
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "application/json"))
            .body(serde_json::to_string::<bundlesdb::DefaultReturn<Option<bundlesdb::Paste<String>>>>(&res).unwrap());
    }

    // make sure the paste allows their SSM content to be public
    if !res
        .payload
        .as_ref()
        .unwrap()
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
        .body(ssm::parse_ssm_blocks(res.payload.unwrap().content));
}

#[post("/api/new")]
pub async fn create_request(
    req: HttpRequest,
    body: web::Json<CreateInfo>,
    data: web::Data<bundlesdb::AppData>,
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
    }

    // create paste
    let res = data
        .db
        .create_paste(
            &mut bundlesdb::Paste {
                custom_url: custom_url.clone(),
                id: String::new(), // reassigned anyways, this doesn't matter
                edit_password: edit_password.to_string(),
                content: content.clone(),
                content_html: crate::markdown::parse_markdown(&content), // go ahead and render the content
                pub_date: utility::unix_epoch_timestamp(),
                edit_date: utility::unix_epoch_timestamp(),
                group_name: g_name_for_real.to_string(),
                metadata: String::new(), // will be filled automatically
                views: 0,
            },
            if token_user.is_some() {
                Option::Some(token_user.unwrap().payload.unwrap().username)
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
pub async fn edit_request(
    req: HttpRequest,
    body: web::Json<EditInfo>,
    data: web::Data<bundlesdb::AppData>,
) -> impl Responder {
    let custom_url: String = body.custom_url.trim().to_string();
    let content: String = body.content.trim().to_string();
    let edit_password: String = body.edit_password.to_owned();
    let new_url: Option<String> = body.new_custom_url.to_owned();
    let new_edit_password: Option<String> = body.new_edit_password.to_owned();

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
                Option::Some(token_user.unwrap().payload.unwrap().username)
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
pub async fn delete_request(
    body: web::Json<DeleteInfo>,
    data: web::Data<bundlesdb::AppData>,
) -> impl Responder {
    let custom_url: String = body.custom_url.trim().to_string();
    let edit_password: String = body.edit_password.to_owned();

    let res = data.db.delete_paste_by_url(custom_url, edit_password).await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(serde_json::to_string(&res).unwrap());
}

#[post("/api/metadata")]
pub async fn metadata_request(
    req: HttpRequest,
    body: web::Json<MetadataInfo>,
    data: web::Data<bundlesdb::AppData>,
) -> impl Responder {
    let custom_url: String = body.custom_url.trim().to_string();
    let edit_password: String = body.edit_password.to_owned();

    let m = body.metadata.to_owned();
    let metadata: bundlesdb::PasteMetadata = m;

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
    }

    // ...
    let res = data
        .db
        .edit_paste_metadata_by_url(
            custom_url,
            metadata,
            edit_password,
            if token_user.is_some() {
                Option::Some(token_user.unwrap().payload.unwrap().username)
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
pub async fn exists_request(
    req: HttpRequest,
    data: web::Data<bundlesdb::AppData>,
) -> impl Responder {
    let custom_url: String = req.match_info().get("url").unwrap().to_string();
    let res = data.db.get_paste_by_url(custom_url).await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "text/plain"))
        .body(res.success.to_string());
}

#[get("/api/url/{url:.*}")]
pub async fn get_from_url_request(
    req: HttpRequest,
    data: web::Data<bundlesdb::AppData>,
) -> impl Responder {
    let custom_url: String = req.match_info().get("url").unwrap().to_string();
    let res: bundlesdb::DefaultReturn<Option<bundlesdb::Paste<String>>> =
        data.db.get_paste_by_url(custom_url).await;

    // if res.metadata contains '"private_source":"on"', return NotFound
    if res.payload.is_some()
        && res
            .clone()
            .payload
            .unwrap()
            .metadata
            .contains("\"private_source\":\"on\",")
    {
        return HttpResponse::NotFound()
            .body("You do not have permission to view this paste's contents.");
    }

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(
            serde_json::to_string::<bundlesdb::DefaultReturn<Option<bundlesdb::Paste<String>>>>(
                &res,
            )
            .unwrap(),
        );
}

#[get("/api/id/{id:.*}")]
pub async fn get_from_id_request(
    req: HttpRequest,
    data: web::Data<bundlesdb::AppData>,
) -> impl Responder {
    let id: String = req.match_info().get("id").unwrap().to_string();
    let res: bundlesdb::DefaultReturn<Option<bundlesdb::Paste<String>>> =
        data.db.get_paste_by_id(id).await;

    // if res.metadata contains '"private_source":"on"', return NotFound
    if res.payload.is_some()
        && res
            .clone()
            .payload
            .unwrap()
            .metadata
            .contains("\"private_source\":\"on\",")
    {
        return HttpResponse::NotFound()
            .body("You do not have permission to view this paste's contents.");
    }

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(
            serde_json::to_string::<bundlesdb::DefaultReturn<Option<bundlesdb::Paste<String>>>>(
                &res,
            )
            .unwrap(),
        );
}

#[get("/api/owner/{username:.*}")]
pub async fn get_from_owner_request(
    req: HttpRequest,
    data: web::Data<bundlesdb::AppData>,
) -> impl Responder {
    let username: String = req.match_info().get("username").unwrap().to_string();
    let res: bundlesdb::DefaultReturn<Option<Vec<bundlesdb::PasteIdentifier>>> =
        data.db.get_pastes_by_owner(username).await;

    // return
    return HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(
            serde_json::to_string::<
                bundlesdb::DefaultReturn<Option<Vec<bundlesdb::PasteIdentifier>>>,
            >(&res)
            .unwrap(),
        );
}
