use actix_web::HttpRequest;
use actix_web::{get, web, HttpResponse, Responder};

use yew::prelude::*;
use yew::ServerRenderer;

use crate::components::navigation::Footer;
use crate::db::bundlesdb::{AtomicPasteFSFile, Paste};
use crate::db::{self, bundlesdb};
use crate::utility::{self, format_html};

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct EditQueryProps {
    pub path: Option<String>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct FSProps {
    pub files: Vec<bundlesdb::AtomicPasteFSFile>,
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct EditProps {
    pub custom_url: String,
    pub file: bundlesdb::AtomicPasteFSFile,
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct NewProps {
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct Props {
    pub pastes: Vec<bundlesdb::PasteIdentifier>,
    pub auth_state: Option<bool>,
}

#[function_component]
fn Dashboard(props: &Props) -> Html {
    return html! {
        <div class="flex flex-column g-4" style="height: 100dvh;">
            <main class="small">
                <div class="card round secondary flex g-4 flex-wrap mobile:flex-column justify-center align-center" id="pastes_list">
                    {for props.pastes.iter().map(|p| html! {
                        <a class="button secondary round mobile:max" href={format!("/d/atomic/{}?path=/index.html", &p.id)}>{&p.custom_url}</a>
                    })}

                    <a class="button border round mobile:max" href="/d/atomic/new">
                        <svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-plus-square"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M8 12h8"/><path d="M12 8v8"/></svg>
                        {"New"}
                    </a>
                </div>

                <style>
                    {"#pastes_list .button {
                        display: flex;
                        width: 10rem !important;
                        height: 10rem !important;
                        flex-direction: column;
                        gap: var(--u-08);
                        align-items: center;
                        justify-content: center;
                    }"}
                </style>

                <Footer auth_state={props.auth_state} />
            </main>
        </div>
    };
}

fn build_dashboard_renderer_with_props(props: Props) -> ServerRenderer<Dashboard> {
    return ServerRenderer::<Dashboard>::with_props(|| props);
}

#[get("/d/atomic")]
pub async fn dashboard_request(
    req: HttpRequest,
    data: web::Data<db::bundlesdb::AppData>,
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
        .get_atomic_pastes_by_owner(token_user.clone().unwrap().payload.unwrap().username)
        .await;

    // ...
    let renderer = build_dashboard_renderer_with_props(Props {
        pastes: pastes.payload.unwrap(),
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
            "<title>Atomic Dashboard - ::SITE_NAME::</title>",
        ));
}

#[function_component]
fn CreateNew(props: &NewProps) -> Html {
    return html! {
        <div class="flex flex-column g-4" style="height: 100dvh;">
            <main class="small flex flex-column g-4 align-center">
                <div class="card secondary round border" style="width: 25rem;" id="forms">
                    <div id="error" class="mdnote note-error full" style="display: none;" />
                    <form class="full flex flex-column g-4" action="/api/auth/register" id="create-site">
                        <label for="custom_url"><b>{"Custom URL"}</b></label>

                        <input
                            type="text"
                            name="custom_url"
                            id="custom_url"
                            placeholder="Custom URL"
                            class="full round"
                            minlength={4}
                            maxlength={32}
                        />

                        <hr />

                        <button class="bundles-primary full round">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-plus"><path d="M5 12h14"/><path d="M12 5v14"/></svg>
                            {"Create Site"}
                        </button>
                    </form>
                </div>

                <script type="module">
                    {"import AuthPages from \"/static/js/NewAtomic.js\";"}
                </script>

                <Footer auth_state={props.auth_state} />
            </main>
        </div>
    };
}

fn build_new_renderer_with_props(props: NewProps) -> ServerRenderer<CreateNew> {
    return ServerRenderer::<CreateNew>::with_props(|| props);
}

#[get("/d/atomic/new")]
pub async fn new_request(
    req: HttpRequest,
    data: web::Data<db::bundlesdb::AppData>,
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
    } else {
        // you must have an account to use atomic pastes
        // we'll likely track bandwidth used by atomic pastes and limit it in the future
        return HttpResponse::NotFound().body(
            "You must have an account to use atomic pastes.
You can login at: /d/auth/login
You can create an account at: /d/auth/register",
        );
    }

    // ...
    let renderer = build_new_renderer_with_props(NewProps {
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
            "<title>New Atomic Paste - ::SITE_NAME::</title>",
        ));
}

#[function_component]
fn EditPaste(props: &EditProps) -> Html {
    return html! {
        <div class="flex flex-column" style="height: 100dvh;">
            <div id="_doc" style="height: 100%; overflow: auto;" />
            <div class="card secondary flex mobile:justify-center justify-space-between align-center" style="
                overflow: auto hidden;
                border-top: 1px solid var(--background-surface2a);
            ">
                // editor actions
                <b style="min-width: max-content;" class="device:desktop">{&props.file.path}</b>

                <div class="flex g-4">
                    <button class="round secondary" id="save">{"Save"}</button>
                    <button class="round red secondary" id="delete">{"Delete"}</button>
                    <a href="?" class="button round secondary" id="save" target="_blank">{"Files"}</a>
                    <div class="hr-left" />
                    <button class="round border" id="preview">{"Preview"}</button>
                </div>
            </div>

            <script type="module">
                {format!("import {{ create_editor }} from \"/static/js/AtomicEditor.js\";
                create_editor(document.getElementById('_doc'), '{}', '{}');
                globalThis.AtomicEditor.Update(`{}`)", &props.custom_url, &props.file.path, &props.file.content.replace("`", "\\`"))}
            </script>

            <style>
                {".cm-editor, .cm-line, .cm-line span { font-family: monospace !important; }"}
            </style>
        </div>
    };
}

fn build_edit_renderer_with_props(props: EditProps) -> ServerRenderer<EditPaste> {
    return ServerRenderer::<EditPaste>::with_props(|| props);
}

#[function_component]
fn PasteFiles(props: &FSProps) -> Html {
    return html! {
        <div class="flex flex-column" style="height: 100dvh;">
            <main class="small">
                <div class="card secondary round flex flex-column g-4">
                    {for props.files.iter().map(|p| html! {
                        <a href={format!("?path={}", &p.path)}>{&p.path}</a>
                    })}

                    <hr />

                    <form class="flex justify-center align-center g-4 flex-wrap mobile:flex-column">
                        <input type="text" placeholder="/index.html" name="path" class="round mobile:max" minlength={4} />
                        <button class="round bundles-primary mobile:max">{"Open"}</button>
                    </form>
                </div>

                <Footer auth_state={props.auth_state} />
            </main>
        </div>
    };
}

fn build_fs_renderer_with_props(props: FSProps) -> ServerRenderer<PasteFiles> {
    return ServerRenderer::<PasteFiles>::with_props(|| props);
}

#[get("/d/atomic/{id:.*}")]
pub async fn edit_request(
    req: HttpRequest,
    data: web::Data<db::bundlesdb::AppData>,
    info: web::Query<EditQueryProps>,
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
    } else {
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
    let paste: bundlesdb::DefaultReturn<Option<Paste<String>>> = data.db.get_paste_by_id(id).await;

    if paste.success == false {
        let renderer = ServerRenderer::<crate::pages::errors::_404Page>::new();
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "text/html"))
            .body(utility::format_html(
                renderer.render().await,
                "<title>404: Not Found</title>",
            ));
    }

    // make sure paste is an atomic paste
    let unwrap = paste.payload.unwrap();
    let is_atomic = unwrap.content.contains("\"_is_atomic\":true");

    if is_atomic == false {
        return HttpResponse::NotFound().body("Paste is not atomic");
    }

    // get file from path
    let real_content = serde_json::from_str::<bundlesdb::AtomicPaste>(&unwrap.content);

    if real_content.is_err() {
        return HttpResponse::NotAcceptable().body("Paste failed to deserialize");
    }

    let decoded = real_content.unwrap();

    // show file list if path is none
    if info.path.is_none() {
        let renderer = build_fs_renderer_with_props(FSProps {
            files: decoded.files,
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
                &format!(
                    "<title>Files in {} - ::SITE_NAME::</title>",
                    &unwrap.custom_url
                ),
            ));
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
    let renderer = build_edit_renderer_with_props(EditProps {
        custom_url: unwrap.custom_url,
        file: file.unwrap().to_owned(),
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
            &format!("<title>{} - ::SITE_NAME::</title>", path_unwrap),
        ));
}
