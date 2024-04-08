use actix_web::HttpResponse;
use actix_web::{get, web, HttpRequest, Responder};

use serde_json::json;
use yew::prelude::*;
use yew::ServerRenderer;

use crate::db::{self, AppData, FullPaste, FullUser, Paste, PasteMetadata, UserMetadata};

use crate::utility;
use crate::utility::format_html;

use crate::components::navigation::{Footer, GlobalMenu};

#[derive(Default, Properties, PartialEq)]
pub struct Props {
    pub paste: Paste<PasteMetadata>,
    pub user: Option<FullUser<String>>,
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
pub struct PasteViewProps {
    pub view: Option<String>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct DashboardProps {
    pub pastes: Vec<db::PasteIdentifier>,
    pub auth_state: Option<bool>,
    pub offset: i32,
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
    <div class=\"flex g-4 flex-wrap mobile:flex-column\">
        {{{ edit_button }}}
        {{{ config_button }}}
    </div>

    <div class=\"flex flex-column g-2 text-right\" style=\"color: var(--text-color-faded); min-width: max-content;\">
        <span class=\"flex justify-center g-4\" id=\"paste-info-pub\">
            <svg xmlns=\"http://www.w3.org/2000/svg\" width=\"18\" height=\"18\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" class=\"lucide lucide-cake-slice\"><circle cx=\"9\" cy=\"7\" r=\"2\"/><path d=\"M7.2 7.9 3 11v9c0 .6.4 1 1 1h16c.6 0 1-.4 1-1v-9c0-2-3-6-7-8l-3.6 2.6\"/><path d=\"M16 13H3\"/><path d=\"M16 17H3\"/></svg>
            Pub: <span class=\"date-time-to-localize\">{{ pub_date }}</span>
        </span>

        <span class=\"flex justify-center g-4\" id=\"paste-info-edit\">
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

#[function_component]
fn PasteView(props: &Props) -> Html {
    let metadata = &props.paste.metadata;
    let user_metadata = if props.user.is_some() {
        Option::Some(
            serde_json::from_str::<UserMetadata>(&props.user.as_ref().unwrap().user.metadata)
                .unwrap(),
        )
    } else {
        Option::None
    };

    // template things
    let edit_button = format!("<a class=\"button round\" href=\"/?editing={}\">
        <svg xmlns=\"http://www.w3.org/2000/svg\" width=\"18\" height=\"18\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" class=\"lucide lucide-pencil\"><path d=\"M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z\"/><path d=\"m15 5 4 4\"/></svg>
        Edit
    </a>", &props.paste.custom_url);

    let config_button = format!("<a href=\"/d/settings/paste/{}\" class=\"button border round\">
        <svg xmlns=\"http://www.w3.org/2000/svg\" width=\"18\" height=\"18\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" class=\"lucide lucide-file-cog\"><path d=\"M4 22h14a2 2 0 0 0 2-2V7l-5-5H6a2 2 0 0 0-2 2v2\"/><path d=\"M14 2v4a2 2 0 0 0 2 2h4\"/><circle cx=\"6\" cy=\"14\" r=\"3\"/><path d=\"M6 10v1\"/><path d=\"M6 17v1\"/><path d=\"M10 14H9\"/><path d=\"M3 14H2\"/><path d=\"m9 11-.88.88\"/><path d=\"M3.88 16.12 3 17\"/><path d=\"m9 17-.88-.88\"/><path d=\"M3.88 11.88 3 11\"/></svg>
        Config
    </a>", &props.paste.custom_url);

    let owner_button = format!("<a href=\"::GUPPY_ROOT::/{}\">{}</a>", &metadata.owner, {
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
            "content": props.paste.content_html,
            "pub_date": props.paste.pub_date,
            "edit_date": props.paste.edit_date,
            "views": props.paste.views,
            // buttons
            "edit_button": edit_button,
            "config_button": config_button,
            "owner_button": owner_button,
            // full data
            "paste": props.paste,
            "metadata": metadata
        }),
    );

    if page.is_err() {
        return html! { <div>{page.err().unwrap().to_string()}</div> };
    }

    // ...
    // TODO: properly sanitize if needed
    let page =
        Html::from_html_unchecked(AttrValue::from(page.unwrap().replace("fetch(", "fetch(\\")));

    // default return
    return html! {
        <main class="flex flex-column g-4">
            <div id="secret" />

            {page}

            <Footer auth_state={props.auth_state} />

            <script type="module">
                {"import ClientFixMarkdown from \"/static/js/ClientFixMarkdown.js\"; ClientFixMarkdown();"}
            </script>
        </main>
    };
}

fn build_renderer_with_props(props: Props) -> ServerRenderer<PasteView> {
    ServerRenderer::<PasteView>::with_props(|| props)
}

#[function_component]
pub fn PastePasswordAsk(props: &Props) -> Html {
    // default return
    return html! {
        <div class="flex flex-column g-4" style="height: 100dvh;">
            <main class="small flex flex-column g-4 align-center">
                <div class="card secondary round border" style="width: 25rem;" id="forms">
                    <h2 class="no-margin text-center full">{props.paste.custom_url.clone()}</h2>

                    <hr />

                    <form class="full flex flex-column g-4" id="login-to-paste">
                        <label for="view"><b>{"View Password"}</b></label>

                        <input
                            type="text"
                            name="view"
                            id="view"
                            placeholder="Paste View Password"
                            class="full round"
                            maxlength={256}
                        />

                        <hr />

                        <button class="bundles-primary full round">
                            {"Continue"}
                        </button>
                    </form>
                </div>
            </main>
        </div>
    };
}

pub fn build_password_ask_renderer_with_props(props: Props) -> ServerRenderer<PastePasswordAsk> {
    ServerRenderer::<PastePasswordAsk>::with_props(|| props)
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
        let renderer = ServerRenderer::<crate::pages::errors::_404Page>::new();
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "text/html"))
            .body(utility::format_html(
                renderer.render().await,
                "<title>404: Not Found</title>",
            ));
    }

    let unwrap = paste.payload.as_ref().unwrap();

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

        let renderer = build_password_ask_renderer_with_props(Props {
            paste: unwrap.clone().paste,
            user: unwrap.clone().user,
            auth_state: if req.cookie("__Secure-Token").is_some() {
                Option::Some(req.cookie("__Secure-Token").is_some())
            } else {
                Option::Some(false)
            },
        });

        let render = renderer.render();
        return HttpResponse::Ok()
            .append_header(("Set-Cookie", ""))
            .append_header(("Content-Type", "text/html"))
            .body(format_html(render.await, ""));
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
                .body("Paste is missing a file at the path '/index.html'");
        }

        return HttpResponse::Ok()
            .append_header(("Content-Type", "text/html"))
            .body(index_html.unwrap().content.clone());
    }

    // verify auth status
    let token_cookie = req.cookie("__Secure-Token");
    let mut set_cookie: &str = "";

    let mut token_user = if token_cookie.is_some() {
        Option::Some(
            data.db
                .get_user_by_unhashed(token_cookie.as_ref().unwrap().value().to_string()) // if the user is returned, that means the ID is valid
                .await,
        )
    } else {
        Option::None
    };

    if token_user.is_some() && token_user.as_ref().unwrap().success == false {
        set_cookie = "__Secure-Token=refresh; SameSite=Strict; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age=0";
        token_user = Option::None;
    }

    if token_user.is_some() {
        // count view (this will check for an existing view!)
        let payload = &token_user.as_ref().unwrap().payload;
        if payload.as_ref().is_some() {
            data.db
                .add_view_to_url(&url_c, &payload.as_ref().unwrap().user.username)
                .await;
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

    // ...
    let renderer = build_renderer_with_props(Props {
        paste: unwrap.clone().paste,
        user: unwrap.clone().user,
        auth_state: if req.cookie("__Secure-Token").is_some() {
            Option::Some(req.cookie("__Secure-Token").is_some())
        } else {
            Option::Some(false)
        },
    });

    let render = renderer.render();
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(format_html(
            render.await,
            &format!(
                "<title>{}</title>
                <meta property=\"og:url\" content=\"{}\" />
                <meta property=\"og:title\" content=\"{}\" />
                <meta property=\"og:description\" content=\"{}\" />
                <meta name=\"theme-color\" content=\"{}\" />
                <link rel=\"icon\" href=\"{}\" />",
                if metadata.title.is_none() | title_unwrap.unwrap().is_empty() {
                    &url_c
                } else {
                    &title_unwrap.unwrap()
                },
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
        ));
}

#[get("/h/{url:.*}/{path:.*}")]
/// Available at "/h/{custom_url}/{file_path}"
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
        let renderer = ServerRenderer::<crate::pages::errors::_404Page>::new();
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "text/html"))
            .body(utility::format_html(
                renderer.render().await,
                "<title>404: Not Found</title>",
            ));
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

#[function_component]
fn Dashboard(props: &DashboardProps) -> Html {
    return html! {
        <div class="flex flex-column" style="height: 100dvh;">
            <GlobalMenu auth_state={props.auth_state} />

            <div class="toolbar flex justify-space-between">
                // left
                <div class="flex">
                    <button title="Menu" b_onclick="window.toggle_child_menu(event.target, '#upper\\\\:globalmenu')" style="border-left: 0">
                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-menu"><line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="18" y2="18"/></svg>
                    </button>

                    <a class="button" href="/d" style="border-left: 0">
                        {"Dashboard"}
                    </a>
                </div>
            </div>

            <div class="toolbar-layout-wrapper">
                <div id="link-header" style="display: flex;" class="flex-column bg-1">
                    <div class="link-header-top"></div>

                    <div class="link-header-middle">
                        <h1 class="no-margin">{"Dashboard"}</h1>
                    </div>

                    <div class="link-header-bottom">
                        <a href="/d" class="button">{"Home"}</a>
                        <a href="/d/pastes" class="button active">{"Pastes"}</a>
                        <a href="/d/atomic" class="button">{"Atomic"}</a>
                        <a href="::PUFFER_ROOT::/d" class="button">{"Boards"}</a>
                    </div>
                </div>

                <main class="small flex flex-column g-4">
                    <div class="flex justify-space-between align-center">
                        <b>{"Pastes"}</b>

                        <a class="button bundles-primary round" href="/">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-plus-square"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M8 12h8"/><path d="M12 8v8"/></svg>
                            {"New"}
                        </a>
                    </div>

                    <div class="card round secondary flex g-4 flex-column justify-center" id="pastes_list">
                        {for props.pastes.iter().map(|p| html! {
                            <a class="button secondary round full justify-start" href={format!("/?editing={}", &p.custom_url)}>
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-file-pen"><path d="M12 22h6a2 2 0 0 0 2-2V7l-5-5H6a2 2 0 0 0-2 2v10"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/><path d="M10.4 12.6a2 2 0 1 1 3 3L8 21l-4 1 1-4Z"/></svg>
                                {&p.custom_url}
                            </a>
                        })}
                    </div>

                    <div class="full flex justify-space-between" id="pages">
                        <a class="button round" href={format!("?offset={}", props.offset - 50)} disabled={props.offset <= 0}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-left"><path d="m12 19-7-7 7-7"/><path d="M19 12H5"/></svg>
                            {"Back"}
                        </a>

                        <a class="button round" href={format!("?offset={}", props.offset + 50)} disabled={props.pastes.len() == 0}>
                            {"Next"}
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-right"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>
                        </a>
                    </div>

                    <Footer auth_state={props.auth_state} />
                </main>
            </div>
        </div>
    };
}

fn build_dashboard_renderer_with_props(props: DashboardProps) -> ServerRenderer<Dashboard> {
    ServerRenderer::<Dashboard>::with_props(|| props)
}

#[get("/d/pastes")]
/// Available at "/d/pastes"
pub async fn dashboard_request(
    req: HttpRequest,
    data: web::Data<db::AppData>,
    info: web::Query<crate::api::pastes::OffsetQueryProps>,
) -> impl Responder {
    // verify auth status
    let token_cookie = req.cookie("__Secure-Token");
    let mut set_cookie: &str = "";

    let mut token_user = if token_cookie.is_some() {
        Option::Some(
            data.db
                .get_user_by_unhashed(token_cookie.as_ref().unwrap().value().to_string()) // if the user is returned, that means the ID is valid
                .await,
        )
    } else {
        Option::None
    };

    if token_user.is_some() {
        // make sure user exists, refresh token if not
        if token_user.as_ref().unwrap().success == false {
            set_cookie = "__Secure-Token=refresh; SameSite=Strict; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age=0";
            token_user = Option::None;
        }
    } else {
        // you must have an account to use atomic pastes
        // we'll likely track bandwidth used by atomic pastes and limit it in the future
        return HttpResponse::NotFound().body(
            "You must have an account to use atomic pastes.
You can login at: /d/auth/login
You can create an account at: /d/auth/register",
        );
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
    let renderer = build_dashboard_renderer_with_props(DashboardProps {
        pastes: pastes.payload.unwrap(),
        auth_state: if req.cookie("__Secure-Token").is_some() {
            Option::Some(true)
        } else {
            Option::Some(false)
        },
        offset: if info.offset.is_some() {
            info.offset.unwrap()
        } else {
            0
        },
    });

    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(format_html(
            renderer.render().await,
            "<title>Atomic Dashboard - ::SITE_NAME::</title>",
        ));
}
