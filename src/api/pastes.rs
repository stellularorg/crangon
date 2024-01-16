use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};

use crate::db::bundlesdb;
use crate::{markdown, utility};

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

#[post("/api/markdown")]
pub async fn render_request(body: web::Json<RenderInfo>) -> impl Responder {
    return HttpResponse::Ok().body(markdown::parse_markdown(&body.text));
}

#[post("/api/new")]
pub async fn create_request(
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

    let res = data
        .db
        .create_paste(&mut bundlesdb::Paste {
            custom_url: custom_url.clone(),
            id: String::new(), // reassigned anyways, this doesn't matter
            edit_password: edit_password.to_string(),
            content,
            pub_date: utility::unix_epoch_timestamp(),
            edit_date: utility::unix_epoch_timestamp(),
            group_name: g_name_for_real.to_string(),
            metadata: bundlesdb::PasteMetadata { owner: custom_url },
        })
        .await;

    // return
    return HttpResponse::Ok().body(serde_json::to_string(&res).unwrap());
}

#[post("/api/edit")]
pub async fn edit_request(
    body: web::Json<EditInfo>,
    data: web::Data<bundlesdb::AppData>,
) -> impl Responder {
    let custom_url: String = body.custom_url.trim().to_string();
    let content: String = body.content.trim().to_string();
    let edit_password: String = body.edit_password.to_owned();
    let new_url: Option<String> = body.new_custom_url.to_owned();
    let new_edit_password: Option<String> = body.new_edit_password.to_owned();

    let res = data
        .db
        .edit_paste_by_url(
            custom_url,
            content,
            edit_password,
            new_url,
            new_edit_password,
        )
        .await;

    // return
    return HttpResponse::Ok().body(serde_json::to_string(&res).unwrap());
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
    return HttpResponse::Ok().body(serde_json::to_string(&res).unwrap());
}

#[get("/api/exists/{url:.*}")]
pub async fn exits_request(
    req: HttpRequest,
    data: web::Data<bundlesdb::AppData>,
) -> impl Responder {
    let custom_url: String = req.match_info().get("url").unwrap().to_string();

    let res = data.db.get_paste_by_url(custom_url).await;

    // return
    return HttpResponse::Ok().body(res.success.to_string());
}
