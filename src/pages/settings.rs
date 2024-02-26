use actix_web::HttpResponse;
use actix_web::{get, web, HttpRequest, Responder};

use yew::prelude::*;
use yew::ServerRenderer;

use crate::db::bundlesdb::{self, AppData, Paste};
use crate::utility::format_html;

use crate::components::navigation::Footer;
use crate::pages::paste_view;

#[derive(Default, Properties, PartialEq)]
struct Props {
    pub paste: Paste<String>,
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq)]
struct UserSettingsProps {
    pub auth_state: Option<bool>,
}

#[function_component]
fn PasteSettings(props: &Props) -> Html {
    let metadata = serde_json::from_str::<bundlesdb::PasteMetadata>(&props.paste.metadata).unwrap();

    return html! {
        <main class="flex flex-column g-4 small">
            <h2 class="full text-center">{"Paste Settings"}</h2>

            <div class="card round secondary flex flex-column g-4">
                <div class="flex full justify-space-between">
                    <div class="flex g-4">
                        <form action="/api/metadata" id="update-form">
                            <button class="green round secondary">
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-save"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
                                {"Save"}
                            </button>
                        </form>

                        <button class="secondary round" id="add_field">{"Add Field"}</button>
                    </div>

                    <a href={format!("/{}", props.paste.custom_url)} class="button round secondary">{"Cancel"}</a>
                </div>

                <div id="options-field" class="flex flex-wrap mobile:flex-column g-4 full justify-space-between" />
            </div>

            <script type="module">
                {format!("import {{ paste_settings }} from \"/static/js/SettingsEditor.js\";
                paste_settings({}, \"{}\", document.getElementById(\"options-field\"));", serde_json::to_string(&metadata).unwrap(), &props.paste.custom_url)}
            </script>

            <Footer auth_state={props.auth_state} />
        </main>
    };
}

#[function_component]
fn UserSettings(props: &UserSettingsProps) -> Html {
    return html! {
        <main class="flex flex-column g-4 small">
            <h2 class="full text-center">{"User Settings"}</h2>

            <div class="card round secondary flex flex-column g-4">
                <div id="options-field" class="flex flex-wrap flex-column g-4 full justify-center" />
            </div>

            <script type="module">
                {"import { user_settings } from \"/static/js/SettingsEditor.js\";
                user_settings(document.getElementById(\"options-field\"));"}
            </script>

            <Footer auth_state={props.auth_state} />
        </main>
    };
}

fn build_paste_settings_with_props(props: Props) -> ServerRenderer<PasteSettings> {
    return ServerRenderer::<PasteSettings>::with_props(|| props);
}

fn build_user_settings_with_props(props: UserSettingsProps) -> ServerRenderer<UserSettings> {
    return ServerRenderer::<UserSettings>::with_props(|| props);
}

#[get("/d/settings")]
/// Available at "/d/settings"
pub async fn user_settings_request(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    // verify auth status
    let token_cookie = req.cookie("__Secure-Token");
    let mut set_cookie: &str = "";

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
        // make sure user exists, refresh token if not
        if token_user.as_ref().unwrap().success == false {
            set_cookie = "__Secure-Token=refresh; SameSite=Strict; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age=0";
        }
    }

    // ...
    let renderer = build_user_settings_with_props(UserSettingsProps {
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
            "<title>User Settings</title>
            <meta property=\"og:title\" content=\"User Settings - ::SITE_NAME::\" />",
        ));
}

#[get("/d/settings/paste/{url:.*}")]
/// Available at "/d/settings/paste/{custom_url}"
pub async fn paste_settings_request(
    req: HttpRequest,
    data: web::Data<AppData>,
    info: web::Query<paste_view::PasteViewProps>,
) -> impl Responder {
    // get paste
    let url: String = req.match_info().get("url").unwrap().to_string();
    let url_c = url.clone();

    let paste: bundlesdb::DefaultReturn<Option<Paste<String>>> =
        data.db.get_paste_by_url(url).await;

    if paste.success == false {
        return HttpResponse::NotFound().body(paste.message);
    }

    // verify auth status
    let token_cookie = req.cookie("__Secure-Token");
    let mut set_cookie: &str = "";

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
        // make sure user exists, refresh token if not
        if token_user.as_ref().unwrap().success == false {
            set_cookie = "__Secure-Token=refresh; SameSite=Strict; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age=0";
        }
    }

    // ...
    let unwrap = paste.payload.clone().unwrap();
    let metadata = serde_json::from_str::<bundlesdb::PasteMetadata>(&unwrap.metadata).unwrap();

    // handle view password
    if metadata.view_password.is_some() && info.view.is_none() {
        let renderer = paste_view::build_password_ask_renderer_with_props(paste_view::Props {
            paste: unwrap,
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
        && info.view.as_ref().unwrap() != &metadata.view_password.unwrap()
    {
        return HttpResponse::NotFound()
            .body("You do not have permission to view this paste's contents.");
    }

    // ...
    let renderer = build_paste_settings_with_props(Props {
        paste: paste.payload.clone().unwrap(),
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
                <meta property=\"og:title\" content=\"{} (paste settings) - ::SITE_NAME::\" />",
                &url_c, &url_c
            ),
        ));
}
