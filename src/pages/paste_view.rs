use actix_web::HttpResponse;
use actix_web::{get, web, HttpRequest, Responder};

use super::base;
use askama::Template;
use serde_json::json;

use crate::db::{self, AppData, FullPaste, PasteMetadata, UserMetadata};

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
    page_content: String,
    head_string: String,
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
    puffer: String,
    site_name: String,
    body_embed: String,
}

pub fn paste_view_hb_template() -> String {
    String::from("<div
    id=\"editor-tab-preview\"
    class=\"card round secondary tab-container secondary round\"
    style=\"height: max-content; max-height: initial; margin-bottom: 0px;\"
>
    {{{ content }}}
</div>

<div class=\"flex justify-space-between g-4 full\" id=\"paste-info-box\">
    <div class=\"flex g-4 flex-wrap\">
        {{{ edit_button }}}
        {{{ config_button }}}
    </div>

    <div class=\"flex flex-column g-2\" style=\"color: var(--text-color-faded); min-width: max-content; align-items: flex-end;\">
        <span class=\"flex g-4\" id=\"paste-info-pub\">
            <svg xmlns=\"http://www.w3.org/2000/svg\" width=\"18\" height=\"18\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" class=\"lucide lucide-cake-slice\"><circle cx=\"9\" cy=\"7\" r=\"2\"/><path d=\"M7.2 7.9 3 11v9c0 .6.4 1 1 1h16c.6 0 1-.4 1-1v-9c0-2-3-6-7-8l-3.6 2.6\"/><path d=\"M16 13H3\"/><path d=\"M16 17H3\"/></svg>
            Pub: <span class=\"date-time-to-localize\">{{ pub_date }}</span>
        </span>

        <span class=\"flex g-4\" id=\"paste-info-edit\">
            <svg xmlns=\"http://www.w3.org/2000/svg\" width=\"18\" height=\"18\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" class=\"lucide lucide-pencil\"><path d=\"M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z\"/><path d=\"m15 5 4 4\"/></svg>
            Edit: <span class=\"date-time-to-localize\">{{ edit_date }}</span>
        </span>

        <span id=\"paste-info-owner\">
            Owner: {{{ owner_button }}}
        </span>

        <span id=\"paste-info-views\">Views: {{ views }}</span>
    </div>
</div>")
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

    // handle atomic pastes (just return index.html)
    if unwrap.paste.content.contains("\"_is_atomic\":true") {
        let real_content = serde_json::from_str::<db::AtomicPaste>(&unwrap.paste.content);

        if real_content.is_err() {
            return HttpResponse::NotAcceptable().body("Paste failed to deserialize");
        }

        let decoded = real_content.unwrap();
        let index_html = decoded.files.iter().find(|f| f.path == "/index.html");

        if index_html.is_none() {
            return HttpResponse::NotAcceptable()
                .append_header(("Content-Type", "text/plain"))
                .body("Paste is missing a file at the path '/index.html'");
        }

        return HttpResponse::Ok()
            .append_header(("Content-Type", "text/html"))
            .body(index_html.unwrap().content.clone());
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
    let user = unwrap.clone().user;

    let metadata = &paste.metadata;
    let user_metadata = if user.is_some() {
        Option::Some(
            serde_json::from_str::<UserMetadata>(&user.as_ref().unwrap().user.metadata).unwrap(),
        )
    } else {
        Option::None
    };

    // template things
    let edit_button = format!("<a class=\"button round\" href=\"/?editing={}\">
        <svg xmlns=\"http://www.w3.org/2000/svg\" width=\"18\" height=\"18\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" class=\"lucide lucide-pencil\"><path d=\"M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z\"/><path d=\"m15 5 4 4\"/></svg>
        Edit
    </a>", &paste.custom_url);

    let config_button = format!("<a href=\"/dashboard/settings/paste/{}\" class=\"button round\">
        <svg xmlns=\"http://www.w3.org/2000/svg\" width=\"18\" height=\"18\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" class=\"lucide lucide-settings\"><path d=\"M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z\"/><circle cx=\"12\" cy=\"12\" r=\"3\"/></svg>
        <span class=\"device:desktop\">Config</span>
    </a>", &paste.custom_url);

    let owner_button = format!("<a href=\"{}/{}\">{}</a>", &base.guppy, &metadata.owner, {
        if user_metadata.is_some() && user_metadata.as_ref().unwrap().nickname.is_some() {
            user_metadata.as_ref().unwrap().nickname.as_ref().unwrap()
        } else {
            &metadata.owner
        }
    });

    // render template
    let default_template = &paste_view_hb_template();
    let reg = handlebars::Handlebars::new();
    let page = reg.render_template(
        if metadata.page_template.is_some() && !metadata.page_template.as_ref().unwrap().is_empty()
        {
            metadata.page_template.as_ref().unwrap() // use provided template
        } else {
            default_template // use default template
        },
        &json!({
            // paste info
            "content": paste.content_html,
            "pub_date": paste.pub_date,
            "edit_date": paste.edit_date,
            "views": paste.views,
            // buttons
            "edit_button": edit_button,
            "config_button": config_button,
            "owner_button": owner_button,
            // full data
            "paste": paste,
            "metadata": metadata
        }),
    );

    if page.is_err() {
        return HttpResponse::NotAcceptable()
            .append_header(("Set-Cookie", set_cookie))
            .append_header(("Content-Type", "text/html"))
            .body(page.err().unwrap().to_string());
    }

    // ...
    // TODO: properly sanitize if needed
    let page = page.unwrap().replace("fetch(", "fetch(\\");

    // ...
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(
            PasteViewTemplate {
                page_content: page,
                title: if metadata.title.is_none() | title_unwrap.unwrap().is_empty() {
                    url_c.clone()
                } else {
                    title_unwrap.unwrap().clone()
                },
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
            .unwrap(),
        );
}

// #[get("/h/{url:.*}/{path:.*}")]
#[get("/+{url:.*}/{path:.*}")]
/// Available at "/+{custom_url}/{file_path}"
pub async fn atomic_paste_view_request(
    req: HttpRequest,
    data: web::Data<AppData>,
) -> impl Responder {
    // get paste
    let url: String = req.match_info().get("url").unwrap().to_string();
    let path: String = req.match_info().get("path").unwrap().to_string();

    let paste: db::DefaultReturn<Option<FullPaste<PasteMetadata, String>>> =
        data.db.get_paste_by_url(url).await;

    if paste.success == false {
        return super::errors::error404(req, data).await;
    }

    let unwrap = paste.payload.as_ref().unwrap();

    // handle atomic pastes (just return index.html)
    if unwrap.paste.content.contains("\"_is_atomic\":true") {
        let real_content = serde_json::from_str::<db::AtomicPaste>(&unwrap.paste.content);

        if real_content.is_err() {
            return HttpResponse::NotAcceptable().body("Paste failed to deserialize");
        }

        let decoded = real_content.unwrap();
        let html_file = decoded
            .files
            .iter()
            .find(|f| f.path == format!("/{}", path));

        if html_file.is_none() {
            return HttpResponse::NotAcceptable()
                .body("Paste is missing a file at the requested path");
        }

        let content_type = match path.split(".").collect::<Vec<&str>>().pop().unwrap() {
            "html" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            _ => "text/plain",
        };

        return HttpResponse::Ok()
            .append_header(("Content-Type", content_type))
            .body(html_file.unwrap().content.clone());
    } else {
        return HttpResponse::NotAcceptable().body("Paste is not atomic (cannot select HTML file)");
    }
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
                puffer: base.puffer,
                site_name: base.site_name,
                body_embed: base.body_embed,
            }
            .render()
            .unwrap(),
        );
}
