use actix_web::HttpRequest;
use actix_web::{get, web, HttpResponse, Responder};

use super::base;
use askama::Template;

use crate::db::{self, AtomicPasteFSFile, FullPaste, PasteMetadata};

#[derive(Default, PartialEq, serde::Deserialize)]
struct EditQueryProps {
    pub path: Option<String>,
}

#[derive(Template)]
#[template(path = "paste/atomic/overview.html")]
struct FSOverviewTemplate {
    custom_url: String,
    files: Vec<db::AtomicPasteFSFile>,
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    puffer: String,
    site_name: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "paste/atomic/editor.html")]
struct EditorTemplate {
    custom_url: String,
    file: db::AtomicPasteFSFile,
    file_content: String,
    // required fields
    site_name: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "paste/atomic/new.html")]
struct NewTemplate {
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    puffer: String,
    site_name: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "paste/atomic/dashboard.html")]
struct DashboardTemplate {
    pastes: Vec<db::PasteIdentifier>,
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    puffer: String,
    site_name: String,
    body_embed: String,
}

#[get("/d/atomic")]
/// Available at "/d/atomic"
pub async fn dashboard_request(req: HttpRequest, data: web::Data<db::AppData>) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        // you must have an account to use atomic pastes
        return HttpResponse::NotFound().body(
            "You must have an account to use atomic pastes.
You can login at: /d/auth/login
You can create an account at: /d/auth/register",
        );
    }

    // fetch pastes
    let pastes = data
        .db
        .get_atomic_pastes_by_owner(token_user.clone().unwrap().payload.unwrap().user.username)
        .await;

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            DashboardTemplate {
                pastes: pastes.payload.unwrap(),
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

#[get("/d/atomic/new")]
/// Available at "/d/atomic/new"
pub async fn new_request(req: HttpRequest, data: web::Data<db::AppData>) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        // you must have an account to use atomic pastes
        return HttpResponse::NotFound().body(
            "You must have an account to use atomic pastes.
You can login at: /d/auth/login
You can create an account at: /d/auth/register",
        );
    }

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            NewTemplate {
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

#[get("/d/atomic/{id:.*}")]
/// Available at "/d/atomic/{id}"
pub async fn edit_request(
    req: HttpRequest,
    data: web::Data<db::AppData>,
    info: web::Query<EditQueryProps>,
) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        // you must have an account to use atomic pastes
        // we'll likely track bandwidth used by atomic pastes and limit it in the future
        return HttpResponse::NotFound().body(
            "You must have an account to use atomic pastes.
You can login at: /d/auth/login
You can create an account at: /d/auth/register",
        );
    }

    // get paste
    let id: String = req.match_info().get("id").unwrap().to_string();
    let paste: db::DefaultReturn<Option<FullPaste<PasteMetadata, String>>> =
        data.db.get_paste_by_id(id).await;

    if paste.success == false {
        return super::errors::error404(req, data).await;
    }

    // make sure paste is an atomic paste
    let unwrap = paste.payload.unwrap().paste;
    let is_atomic = unwrap.content.contains("\"_is_atomic\":true");

    if is_atomic == false {
        return HttpResponse::NotFound().body("Paste is not atomic");
    }

    // get file from path
    let real_content = serde_json::from_str::<db::AtomicPaste>(&unwrap.content);

    if real_content.is_err() {
        return HttpResponse::NotAcceptable().body("Paste failed to deserialize");
    }

    let decoded = real_content.unwrap();

    // show file list if path is none
    if info.path.is_none() {
        let base = base::get_base_values(token_user.is_some());
        return HttpResponse::Ok()
            .append_header(("Set-Cookie", set_cookie))
            .append_header(("Content-Type", "text/html"))
            .body(
                FSOverviewTemplate {
                    custom_url: unwrap.custom_url.clone(),
                    files: decoded.files,
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

    let path_unwrap = info.path.clone().unwrap();

    // ...
    let mut file = decoded.files.iter().find(|f| f.path == path_unwrap);
    let blank_file = AtomicPasteFSFile {
        path: path_unwrap.clone(),
        content: String::from("<!-- New HTML Page -->"),
    };

    if file.is_none() {
        file = Option::Some(&blank_file);
    }

    // ...
    let file = file.unwrap().to_owned();
    let file_content = file
        .content
        .replace("\\", "\\\\")
        .replace("`", "\\`")
        .replace("$", "\\$")
        .replace("/", "\\/");

    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            EditorTemplate {
                custom_url: unwrap.custom_url,
                file,
                file_content,
                // required fields
                site_name: base.site_name,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}
