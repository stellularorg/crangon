use actix_web::HttpRequest;
use actix_web::{get, web, HttpResponse, Responder};

use yew::prelude::*;
use yew::ServerRenderer;

use crate::components::navigation::Footer;
use crate::db::{self, bundlesdb};
use crate::utility::format_html;

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct Props {
    pub editing: Option<String>,
    pub starting_content: Option<String>,
    pub password_not_needed: Option<bool>,
    pub auth_state: Option<bool>,
}

#[function_component]
fn Home(props: &Props) -> Html {
    return html! {
        <div class="flex flex-column g-4" style="height: 100dvh;">
            <main style="height: calc(100% - 1rem);">
                <div class="tabbar justify-space-between full">
                    // left
                    <div class="flex">
                        <button id="editor-open-tab-text">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-notebook-pen"><path d="M13.4 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-7.4"/><path d="M2 6h4"/><path d="M2 10h4"/><path d="M2 14h4"/><path d="M2 18h4"/><path d="M18.4 2.6a2.17 2.17 0 0 1 3 3L16 11l-4 1 1-4Z"/></svg>
                            {"Text"}
                        </button>
                        <button id="editor-open-tab-preview" class="secondary">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-paintbrush"><path d="M18.37 2.63 14 7l-1.59-1.59a2 2 0 0 0-2.82 0L8 7l9 9 1.59-1.59a2 2 0 0 0 0-2.82L17 10l4.37-4.37a2.12 2.12 0 1 0-3-3Z"/><path d="M9 8c-2 3-4 3.5-7 4l8 10c2-1 6-5 6-7"/><path d="M14.5 17.5 4.5 15"/></svg>
                            {"Preview"}
                        </button>
                    </div>
                </div>

                <div id="-editor" class="tab-container card secondary round" style="border-top-left-radius: 0px !important; padding: var(--u-10) !important;">
                    <div id="editor-tab-text" class="editor-tab -editor active" style="height: 100%;" />
                    <div id="editor-tab-preview" class="editor-tab -editor" />
                </div>

                <form class="flex flex-wrap mobile:justify-center justify-space-between g-4 align-center" action="/api/new" id="save-changes" data-edit={if props.editing.is_some() { props.editing.as_ref().unwrap().to_owned() } else { "false".to_string() }}>
                    if props.editing.is_none() {
                        <div class="mobile:justify-center flex g-4 justify-start">
                            <button class="round">
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-plus"><path d="M5 12h14"/><path d="M12 5v14"/></svg>
                                {"Publish"}
                            </button>

                            <a
                                class="button round border"
                                href="javascript:document.getElementById('more-modal').showModal();"
                            >
                                {"More"}
                            </a>
                        </div>

                        <div class="mobile:justify-center flex-wrap flex g-4 justify-start">
                            <input
                                class="secondary round"
                                type="text"
                                placeholder="Custom URL"
                                minlength="2"
                                maxlength="500"
                                name="custom_url"
                                id="custom_url"
                                autocomplete="off"
                            />

                            <input
                                class="secondary round"
                                type="text"
                                placeholder="Edit Password"
                                minlength="5"
                                name="edit_password"
                            />
                        </div>

                        <dialog id="more-modal">
                            <div style="width: 25rem; max-width: 100%;">
                                <h2 class="no-margin full text-center">{"More Options"}</h2>

                                <hr />

                                <details class="full round">
                                    <summary>{"Group Settings"}</summary>

                                    <div class="card secondary">
                                        <input
                                            class="full secondary round"
                                            type="text"
                                            placeholder="Group Name"
                                            minlength="2"
                                            maxlength="500"
                                            name="group_name"
                                            id="group_name"
                                            autocomplete="off"
                                        />
                                    </div>
                                </details>

                                <hr />

                                <div class="full flex justify-right">
                                    <a
                                        class="button round red"
                                        href="javascript:document.getElementById('more-modal').close();"
                                    >
                                        {"Close"}
                                    </a>
                                </div>
                            </div>
                        </dialog>
                    } else {
                        <div class="mobile:justify-center flex g-4 justify-start full mobile:flex-column">
                            <input
                                class="secondary round full"
                                type="text"
                                placeholder="Edit Password"
                                minlength="5"
                                name="edit_password"
                                disabled={props.password_not_needed.is_some() && props.password_not_needed.unwrap() == true}
                                value={if props.password_not_needed.is_some() && props.password_not_needed.unwrap() == true {
                                    "not needed, you're the owner!"
                                } else {
                                    ""
                                }}
                            />

                            <input
                                class="secondary round full"
                                type="text"
                                placeholder="New Edit Password (optional)"
                                minlength="5"
                                name="new_edit_password"
                            />

                            <input
                                class="secondary round full"
                                type="text"
                                placeholder="New Custom URL (optional)"
                                minlength="2"
                                maxlength="500"
                                name="new_custom_url"
                                id="new_custom_url"
                                autocomplete="off"
                            />
                        </div>

                        <div class="flex g-4 justify-space-between full">
                            <div class="flex g-4 justify-start">
                                <button class="green round">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-save"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
                                    {"Save"}
                                </button>

                                <a href={format!("/d/settings/paste/{}", props.editing.as_ref().unwrap())} class="button round">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-file-cog"><path d="M4 22h14a2 2 0 0 0 2-2V7l-5-5H6a2 2 0 0 0-2 2v2"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/><circle cx="6" cy="14" r="3"/><path d="M6 10v1"/><path d="M6 17v1"/><path d="M10 14H9"/><path d="M3 14H2"/><path d="m9 11-.88.88"/><path d="M3.88 16.12 3 17"/><path d="m9 17-.88-.88"/><path d="M3.88 11.88 3 11"/></svg>
                                    {"Config"}
                                </a>

                                <a href="/" class="border button round">{"Cancel"}</a>
                            </div>

                            <a href="javascript:" id="delete-btn" class="button round red">{"Delete"}</a>
                        </div>
                    }

                </form>

                <script type="module">
                    {format!(
                        "import CreateEditor from \"/static/js/MarkdownEditor.js\"; CreateEditor(\"editor-tab-text\", `{}`);",
                        if props.starting_content.is_some() {
                            props.starting_content.as_ref().unwrap()
                        } else {
                            ""
                        }
                    )}
                </script>

                <div style={if props.editing.is_none() { "display: block;" } else { "display: none;" }}>
                    <Footer auth_state={props.auth_state} />
                </div>
            </main>
        </div>
    };
}

fn build_renderer_with_props(props: Props) -> ServerRenderer<Home> {
    return ServerRenderer::<Home>::with_props(|| props);
}

#[get("/")]
pub async fn home_request(
    req: HttpRequest,
    data: web::Data<db::bundlesdb::AppData>,
    info: web::Query<Props>,
) -> impl Responder {
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
    let str: &Option<String> = &info.editing;

    let paste = if str.is_some() {
        Option::Some(data.db.get_paste_by_url(str.to_owned().unwrap()).await)
    } else {
        Option::None
    };

    let metadata = if paste.is_some() && paste.as_ref().unwrap().payload.is_some() {
        Option::Some(
            serde_json::from_str::<bundlesdb::PasteMetadata>(
                &paste.as_ref().unwrap().payload.as_ref().unwrap().metadata,
            )
            .unwrap(),
        )
    } else {
        Option::None
    };

    // if metadata has "private_source" set to "on" and we're not the owner, return
    if metadata.is_some() {
        let owner = &metadata.as_ref().unwrap().owner;
        if metadata.as_ref().unwrap().private_source == String::from("on") {
            if token_user.is_none() {
                return HttpResponse::NotFound()
                    .body("You do not have permission to view this paste's contents.");
            }

            let payload = &token_user.as_ref().unwrap().payload;
            if owner.to_string() != payload.as_ref().unwrap().username {
                return HttpResponse::NotFound()
                    .body("You do not have permission to view this paste's contents.");
            }
        }
    };

    // ...
    let renderer = build_renderer_with_props(Props {
        editing: str.to_owned(),
        starting_content: if paste.is_some() {
            if paste.as_ref().unwrap().success {
                Option::Some(paste.unwrap().payload.unwrap().content.replace(r"`", "\\`"))
            } else {
                Option::None
            }
        } else {
            Option::None
        },
        password_not_needed: if metadata.is_some() && token_user.is_some() {
            Option::Some(metadata.unwrap().owner == token_user.unwrap().payload.unwrap().username)
        } else {
            Option::None
        },
        auth_state: if req.cookie("__Secure-Token").is_some() {
            Option::Some(true)
        } else {
            Option::Some(false)
        },
    });

    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .append_header(("Content-Type", "text/html"))
        .body(format_html(
            renderer.render().await,
            "<title>Bundlrs</title>
<meta property=\"og:title\" content=\"Create a new paste...\" />
<meta property=\"og:description\" content=\"Bundlrs, the open-source Rust rewrite of Bundles.\" />",
        ));
}

#[get("/robots.txt")]
pub async fn robotstxt() -> impl Responder {
    return HttpResponse::Ok().body(
        "User-agent: *
Allow: /
Disallow: /api
Disallow: /admin
Disallow: /paste
Disallow: /*?",
    );
}
