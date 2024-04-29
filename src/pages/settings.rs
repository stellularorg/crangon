use actix_web::HttpResponse;
use actix_web::{get, web, HttpRequest, Responder};

use super::base;
use askama::Template;

use crate::db::{self, AppData, FullPaste, Paste, PasteMetadata};
use crate::pages::paste_view;

#[derive(Template)]
#[template(path = "paste/settings.html")]
struct PasteSettingsTemplate {
    paste: Paste<PasteMetadata>,
    metadata: String,
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    site_name: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "user/settings.html")]
struct UserSettingsTemplate {
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    site_name: String,
    body_embed: String,
}

#[get("/dashboard/settings")]
/// Available at "/dashboard/settings"
pub async fn user_settings_request(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            UserSettingsTemplate {
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

#[get("/dashboard/settings/paste/{url:.*}")]
/// Available at "/dashboard/settings/paste/{custom_url}"
pub async fn paste_settings_request(
    req: HttpRequest,
    data: web::Data<AppData>,
    info: web::Query<paste_view::PasteViewProps>,
) -> impl Responder {
    // get paste
    let url: String = req.match_info().get("url").unwrap().to_string();

    let paste: db::DefaultReturn<Option<FullPaste<PasteMetadata, String>>> =
        data.db.get_paste_by_url(url).await;

    if paste.success == false {
        return HttpResponse::NotFound().body(paste.message);
    }

    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    // ...
    let unwrap = paste.payload.clone().unwrap();
    let metadata = &unwrap.paste.metadata;

    // handle view password
    if metadata.view_password.is_some()
        && info.view.is_none()
        && metadata.view_password.as_ref().unwrap() != "off"
    {
        return super::errors::error404(req, data).await;
    }

    // (check password)
    if info.view.is_some()
        && metadata.view_password.is_some()
        && metadata.view_password.as_ref().unwrap() != "off"
        && &info.view.as_ref().unwrap() != &metadata.view_password.as_ref().unwrap()
    {
        return HttpResponse::NotFound()
            .body("You do not have permission to view this paste's contents.");
    }

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            PasteSettingsTemplate {
                paste: paste.payload.clone().unwrap().paste,
                metadata: serde_json::to_string::<PasteMetadata>(metadata)
                    .unwrap()
                    .replace("/", "\\/"),
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
