use actix_web::HttpRequest;
use actix_web::{get, web, HttpResponse, Responder};

use yew::prelude::*;
use yew::ServerRenderer;

use crate::components::navigation::{Footer, GlobalMenu};
use crate::db::{self, AtomicPasteFSFile, FullPaste, PasteMetadata};
use crate::utility::{self, format_html};

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct EditQueryProps {
    pub path: Option<String>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct FSProps {
    pub custom_url: String,
    pub files: Vec<db::AtomicPasteFSFile>,
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct EditProps {
    pub custom_url: String,
    pub file: db::AtomicPasteFSFile,
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct NewProps {
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct Props {
    pub pastes: Vec<db::PasteIdentifier>,
    pub auth_state: Option<bool>,
}

#[function_component]
fn Dashboard(props: &Props) -> Html {
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
                        <a href="/d/pastes" class="button">{"Pastes"}</a>
                        <a href="/d/atomic" class="button active">{"Atomic"}</a>
                        <a href="::PUFFER_ROOT::/d" class="button">{"Boards"}</a>
                    </div>
                </div>

                <main class="small flex flex-column g-4">
                    <div class="flex justify-space-between align-center">
                        <b>{"Atomic Pastes"}</b>

                        <a class="button bundles-primary round" href="/d/atomic/new">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-plus-square"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M8 12h8"/><path d="M12 8v8"/></svg>
                            {"New"}
                        </a>
                    </div>

                    <div class="card round secondary flex g-4 flex-column justify-center" id="pastes_list">
                        {for props.pastes.iter().map(|p| html! {
                            <a class="button secondary round full justify-start" href={format!("/d/atomic/{}?", &p.id)}>
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-folder-archive"><circle cx="15" cy="19" r="2"/><path d="M20.9 19.8A2 2 0 0 0 22 18V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2h5.1"/><path d="M15 11v-1"/><path d="M15 17v-2"/></svg>
                                {&p.custom_url}
                            </a>
                        })}
                    </div>

                    <Footer auth_state={props.auth_state} />
                </main>
            </div>
        </div>
    };
}

fn build_dashboard_renderer_with_props(props: Props) -> ServerRenderer<Dashboard> {
    ServerRenderer::<Dashboard>::with_props(|| props)
}

#[get("/d/atomic")]
/// Available at "/d/atomic"
pub async fn dashboard_request(req: HttpRequest, data: web::Data<db::AppData>) -> impl Responder {
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
    }

    if token_user.is_none() {
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
        .get_atomic_pastes_by_owner(token_user.clone().unwrap().payload.unwrap().user.username)
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
                            required={true}
                        />

                        <hr />

                        <button class="bundles-primary full round">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-plus"><path d="M5 12h14"/><path d="M12 5v14"/></svg>
                            {"Create Site"}
                        </button>
                    </form>
                </div>

                <script type="module">
                    {"import \"/static/js/NewAtomic.js\";"}
                </script>

                <Footer auth_state={props.auth_state} />
            </main>
        </div>
    };
}

fn build_new_renderer_with_props(props: NewProps) -> ServerRenderer<CreateNew> {
    ServerRenderer::<CreateNew>::with_props(|| props)
}

#[get("/d/atomic/new")]
/// Available at "/d/atomic/new"
pub async fn new_request(req: HttpRequest, data: web::Data<db::AppData>) -> impl Responder {
    // verify auth status
    let token_cookie = req.cookie("__Secure-Token");
    let mut set_cookie: &str = "";

    let token_user = if token_cookie.is_some() {
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
            // token_user = Option::None;
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
            <div class="panes flex mobile:flex-column" style="height: 100%; overflow: auto;">
                <div id="_doc" class="full" style="height: 100%; overflow: auto; display: block;"></div>

                <div id="_preview_browser" class="full" style="height: 100%; overflow: hidden; display: none;">
                    <div class="full flex g-4 bg-0" style="padding: var(--u-04); height: 47.8px;">
                        <button class="round" id="preview" title="Refresh Preview">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-refresh-cw"><path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/></svg>
                        </button>
                    </div>

                    <iframe id="_preview_pane" class="full" style="height: calc(100% - 47.8px); overflow: auto;" frameborder="0" src="about:blank"></iframe>
                </div>

                <style>
                    {"#_preview_pane { background: white; }
                    #_preview_browser { border-left: solid 1px var(--background-surface2a); }
                    @media screen and (max-width: 900px) { #_preview_browser { border-left: 0; border-top: solid 1px var(--background-surface2a); } }"}
                </style>
            </div>

            <div class="card secondary flex mobile:justify-center justify-space-between align-center" style="
                overflow: auto hidden;
                border-top: 1px solid var(--background-surface2a);
                padding: var(--u-04);
                height: 47.8px;
            ">
                // editor actions
                <b style="min-width: max-content;" class="device:desktop">{&props.file.path}</b>

                <div class="flex g-4">
                    <button class="round secondary green" id="save" title="Save File">
                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-save"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
                    </button>

                    <a href="?" class="button round secondary" id="file_explorer" title="Manage Files">
                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-folder-tree"><path d="M20 10a1 1 0 0 0 1-1V6a1 1 0 0 0-1-1h-2.5a1 1 0 0 1-.8-.4l-.9-1.2A1 1 0 0 0 15 3h-2a1 1 0 0 0-1 1v5a1 1 0 0 0 1 1Z"/><path d="M20 21a1 1 0 0 0 1-1v-3a1 1 0 0 0-1-1h-2.9a1 1 0 0 1-.88-.55l-.42-.85a1 1 0 0 0-.92-.6H13a1 1 0 0 0-1 1v5a1 1 0 0 0 1 1Z"/><path d="M3 5a2 2 0 0 0 2 2h3"/><path d="M3 3v13a2 2 0 0 0 2 2h3"/></svg>
                    </a>

                    <div class="hr-left" />

                    <button class="round secondary red" id="split_view" title="Split View">
                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-columns-2"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M12 3v18"/></svg>
                    </button>
                </div>
            </div>

            <script type="module">
                {format!("import {{ create_editor }} from \"/static/js/AtomicEditor.js\";
                create_editor(document.getElementById('_doc'), '{}', '{}');
                globalThis.AtomicEditor.Update(`{}`)", &props.custom_url, &props.file.path, &props.file.content.replace("\\", "\\\\").replace("`", "\\`").replace("$", "\\$"))}
            </script>

            <style>
                {".cm-editor, .cm-line, .cm-line span { font-family: monospace !important; }"}
            </style>
        </div>
    };
}

fn build_edit_renderer_with_props(props: EditProps) -> ServerRenderer<EditPaste> {
    ServerRenderer::<EditPaste>::with_props(|| props)
}

#[function_component]
fn PasteFiles(props: &FSProps) -> Html {
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
                        <h1 class="no-margin">{&props.custom_url}</h1>
                    </div>

                    <div class="link-header-bottom">
                        <a href="/d" class="button">{"Home"}</a>
                        <a href="/d/pastes" class="button">{"Pastes"}</a>
                        <a href="/d/atomic" class="button active">{"Atomic"}</a>
                        <a href="::PUFFER_ROOT::/d" class="button">{"Boards"}</a>
                    </div>
                </div>

                <main class="flex flex-column g-4 small">
                    <div id="error" class="mdnote note-error full" style="display: none;" />
                    <div id="success" class="mdnote note-note full" style="display: none;" />

                    <div id="custom_url" style="display: none;">{&props.custom_url}</div>

                    <form class="flex justify-center align-center g-4">
                        <input type="text" placeholder="/index.(html|css|js)" name="path" class="round full" minlength={4} />
                        <button class="round bundles-primary" style="min-width: max-content;">{"Open"}</button>
                    </form>

                    <table class="full stripped">
                        <thead>
                            <tr>
                                <th>{"Path"}</th>
                                <th>{"Actions"}</th>
                            </tr>
                        </thead>

                        <tbody>
                            {for props.files.iter().map(|p| html! {
                                <tr>
                                    <td><a href={format!("?path={}", &p.path)}>{&p.path}</a></td>

                                    <td class="flex g-4 flex-wrap">
                                        <a class="button secondary round" href={format!("?path={}", &p.path)} title="Edit File">
                                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-file-pen-line"><path d="m18 5-3-3H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2"/><path d="M8 18h1"/><path d="M18.4 9.6a2 2 0 1 1 3 3L17 17l-4 1 1-4Z"/></svg>
                                        </a>

                                        <button class="secondary round action:more-modal" data-suffix={format!("{}{}", &props.custom_url, &p.path)} title="More Options">
                                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-wrench"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/></svg>
                                        </button>
                                    </td>
                                </tr>
                            })}
                        </tbody>
                    </table>

                    <hr />

                    <h6 class="no-margin">{"Paste Options"}</h6>

                    <table class="full stripped">
                        <thead>
                            <tr>
                                <th>{"Name"}</th>
                                <th>{"Use"}</th>
                            </tr>
                        </thead>

                        <tbody>
                            <tr>
                                <td>{"View"}</td>
                                <td>
                                    <a class="button round secondary" target="_blank" href={format!("/{}", props.custom_url)}>
                                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-circle-play"><circle cx="12" cy="12" r="10"/><polygon points="10 8 16 12 10 16 10 8"/></svg>
                                        {"Run"}
                                    </a>
                                </td>
                            </tr>

                            <tr>
                                <td>{"Delete"}</td>
                                <td>
                                    <button class="round secondary" id="delete">
                                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-circle-play"><circle cx="12" cy="12" r="10"/><polygon points="10 8 16 12 10 16 10 8"/></svg>
                                        {"Run"}
                                    </button>
                                </td>
                            </tr>
                        </tbody>
                    </table>

                    // ...
                    <dialog id="more-modal">
                        <div style="width: 25rem; max-width: 100%;">
                            <h2 class="no-margin full text-center">{"More Options"}</h2>

                            <hr />
                            <div id="more-modal-actions" class="flex flex-column g-4"></div>
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

                    <script type="module">
                        {"import \"/static/js/AtomicOverview.js\";"}
                    </script>

                    <Footer auth_state={props.auth_state} />
                </main>
            </div>
        </div>
    };
}

fn build_fs_renderer_with_props(props: FSProps) -> ServerRenderer<PasteFiles> {
    ServerRenderer::<PasteFiles>::with_props(|| props)
}

#[get("/d/atomic/{id:.*}")]
/// Available at "/d/atomic/{id}"
pub async fn edit_request(
    req: HttpRequest,
    data: web::Data<db::AppData>,
    info: web::Query<EditQueryProps>,
) -> impl Responder {
    // verify auth status
    let token_cookie = req.cookie("__Secure-Token");
    let mut set_cookie: &str = "";

    let token_user = if token_cookie.is_some() {
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
            // token_user = Option::None;
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
    let paste: db::DefaultReturn<Option<FullPaste<PasteMetadata, String>>> =
        data.db.get_paste_by_id(id).await;

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
        let renderer = build_fs_renderer_with_props(FSProps {
            custom_url: unwrap.custom_url.clone(),
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
