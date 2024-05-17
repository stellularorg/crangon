use actix_web::HttpResponse;
use actix_web::{get, web, HttpRequest, Responder};

use super::base;
use askama::Template;

use crate::db::{self, AppData, FullPaste, Paste, PasteMetadata};

#[derive(Template)]
#[template(path = "paste/password_ask.html")]
struct PasswordAskTemplate {
    custom_url: String,
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    site_name: String,
    body_embed: String,
}

#[derive(Template)]
#[template(path = "paste/paste_view.html")]
struct PasteViewTemplate {
    title: String,
    head_string: String,
    paste: Paste<PasteMetadata>,
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    site_name: String,
    body_embed: String,
}

#[derive(Default, PartialEq, serde::Deserialize)]
pub struct PasteViewProps {
    pub view: Option<String>,
}

#[derive(Template)]
#[template(path = "paste/dashboard.html")]
struct DashboardTemplate {
    pastes: Vec<db::PasteIdentifier>,
    offset: i32,
    // required fields (super::base)
    info: String,
    auth_state: bool,
    guppy: String,
    site_name: String,
    body_embed: String,
}

#[get("/{url:.*}")]
/// Available at "/{custom_url}"
pub async fn paste_view_request(
    req: HttpRequest,
    data: web::Data<AppData>,
    info: web::Query<PasteViewProps>,
) -> impl Responder {
    // get paste
    let url: String = req.match_info().get("url").unwrap().to_string();
    let url_c = url.clone();

    let paste: db::DefaultReturn<Option<FullPaste<PasteMetadata, String>>> =
        data.db.get_paste_by_url(url).await;

    if paste.success == false {
        return super::errors::error404(req, data).await;
    }

    let unwrap = paste.payload.as_ref().unwrap();

    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    // ...
    let metadata = &unwrap.paste.metadata;

    // handle view password
    if metadata.view_password.is_some()
        && info.view.is_none()
        && metadata.view_password.as_ref().unwrap() != "off"
    {
        if metadata
            .view_password
            .as_ref()
            .unwrap()
            .starts_with("LOCKED(USER_BANNED)-")
        {
            return HttpResponse::NotFound().body("Failed to view paste (LOCKED: OWNER BANNED)");
        }

        let base = base::get_base_values(token_user.is_some());
        return HttpResponse::Ok()
            .append_header(("Set-Cookie", ""))
            .append_header(("Content-Type", "text/html"))
            .body(
                PasswordAskTemplate {
                    custom_url: unwrap.clone().paste.custom_url,
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

    // (check password)
    if info.view.is_some()
        && metadata.view_password.is_some()
        && metadata.view_password.as_ref().unwrap() != "off"
        && &info.view.as_ref().unwrap() != &metadata.view_password.as_ref().unwrap()
    {
        return HttpResponse::NotFound()
            .body("You do not have permission to view this paste's contents.");
    }

    // count view
    if token_user.is_some() && token_user.as_ref().unwrap().payload.is_some() {
        let payload = &token_user.as_ref().unwrap().payload;
        let username = &payload.as_ref().unwrap().user.username;

        // count view (this will check for an existing view!)
        data.db.add_view_to_url(&url_c, &username).await;

        // check permission
        let in_permissions_list = unwrap.paste.metadata.permissions_list.get(username);

        if in_permissions_list.is_some() {
            // "Blocked" is NOT as secure as setting view_password!
            let permission = in_permissions_list.unwrap();

            if permission == &db::PastePermissionLevel::Blocked {
                return HttpResponse::NotAcceptable()
                    .append_header(("Content-Type", "text/plain"))
                    .body("You're blocked from this paste.");
            }
        }
    }

    // ...
    let paste_preview_text: String = unwrap
        .paste
        .content
        .chars()
        .take(100)
        .collect::<String>()
        .replace("\"", "'");

    let title_unwrap = metadata.title.as_ref();
    let description_unwrap = metadata.description.as_ref();
    let embed_color_unwrap = metadata.embed_color.as_ref();
    let favicon_unwrap = metadata.favicon.as_ref();

    let base = base::get_base_values(token_user.is_some());

    // ...
    let paste = unwrap.clone().paste;
    let metadata = &paste.metadata;

    // ...
    let body_content = PasteViewTemplate {
        title: if metadata.title.is_none() | title_unwrap.unwrap().is_empty() {
            url_c.clone()
        } else {
            title_unwrap.unwrap().clone()
        },
        paste: paste.clone(),
        head_string: format!(
            "<meta property=\"og:url\" content=\"{}\" />
                    <meta property=\"og:title\" content=\"{}\" />
                    <meta property=\"og:description\" content=\"{}\" />
                    <meta name=\"theme-color\" content=\"{}\" />
                    <link rel=\"icon\" href=\"{}\" />",
            &format!(
                "{}{}",
                req.headers().get("Host").unwrap().to_str().unwrap(),
                req.head().uri.to_string()
            ),
            // optionals
            if metadata.title.is_none() | title_unwrap.unwrap().is_empty() {
                &url_c
            } else {
                &title_unwrap.unwrap()
            },
            if metadata.description.is_none() | description_unwrap.unwrap().is_empty() {
                &paste_preview_text
            } else {
                &description_unwrap.unwrap()
            },
            if metadata.embed_color.is_none() {
                "#ff9999"
            } else {
                &embed_color_unwrap.unwrap()
            },
            if metadata.favicon.is_none() {
                "/static/favicon.svg"
            } else {
                &favicon_unwrap.unwrap()
            }
        ),
        // required fields
        info: base.info,
        auth_state: base.auth_state,
        guppy: base.guppy,
        site_name: base.site_name,
        body_embed: base.body_embed,
    }
    .render()
    .unwrap();

    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(if paste.metadata.favicon.is_some() {
            // make the original favicon useless
            body_content.replacen("rel=\"icon\"", "rel=\"old_icon\"", 1)
        } else {
            body_content
        });
}

#[get("/dashboard/pastes")]
/// Available at "/dashboard/pastes"
pub async fn dashboard_request(
    req: HttpRequest,
    data: web::Data<db::AppData>,
    info: web::Query<crate::api::pastes::OffsetQueryProps>,
) -> impl Responder {
    // verify auth status
    let (set_cookie, _, token_user) = base::check_auth_status(req.clone(), data.clone()).await;

    if token_user.is_none() {
        // you must have an account to use atomic pastes
        // we'll likely track bandwidth used by atomic pastes and limit it in the future
        return super::errors::error401(req, data).await;
    }

    // fetch pastes
    let pastes = data
        .db
        .get_pastes_by_owner_limited(
            token_user.clone().unwrap().payload.unwrap().user.username,
            info.offset,
        )
        .await;

    // ...
    let base = base::get_base_values(token_user.is_some());
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            DashboardTemplate {
                pastes: pastes.payload.unwrap(),
                offset: if info.offset.is_some() {
                    info.offset.unwrap()
                } else {
                    0
                },
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
