use actix_web::HttpResponse;
use actix_web::{get, web, HttpRequest, Responder};

use yew::prelude::*;
use yew::ServerRenderer;

use crate::db::bundlesdb::{self, AppData, Log, UserMetadata, UserState};
use crate::utility;
use crate::utility::format_html;

use crate::components::navigation::Footer;

#[derive(Default, Properties, PartialEq)]
struct Props {
    pub user: UserState<String>,
    pub paste_count: usize,
    pub board_count: usize,
    pub auth_state: Option<bool>,
    pub active_user: Option<UserState<String>>,
    pub edit_mode: bool,
    pub follower_count: usize,
    pub is_following: bool,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
pub struct QueryProps {
    pub edit: Option<bool>, // Props.edit_mode
}

#[function_component]
fn Register() -> Html {
    return html! {
        <div class="flex flex-column g-4" style="height: 100dvh;">
            <main class="small flex flex-column align-center g-8">
                <div id="success" class="card border round" style="display: none;" />

                <div class="card secondary round border" style="width: 25rem;" id="forms">
                    <div id="error" class="mdnote note-error full" style="display: none;" />
                    <form class="full flex flex-column g-4" action="/api/auth/register" id="register-user">
                        <label for="username"><b>{"Username"}</b></label>

                        <input
                            type="text"
                            name="username"
                            id="username"
                            placeholder="my-unique-username"
                            class="full round"
                            required={true}
                            minlength={4}
                            maxlength={32}
                        />

                        <hr />

                        <button class="bundles-primary full round">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-user-plus"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><line x1="19" x2="19" y1="8" y2="14"/><line x1="22" x2="16" y1="11" y2="11"/></svg>
                            {"Create Account"}
                        </button>
                    </form>
                </div>

                <script type="module">
                    {"import AuthPages from \"/static/js/AuthPages.js\";"}
                </script>

                <Footer auth_state={Option::None} />
            </main>
        </div>
    };
}

#[function_component]
fn Login() -> Html {
    return html! {
        <div class="flex flex-column g-4" style="height: 100dvh;">
            <main class="small flex flex-column align-center g-8">
                <div id="success" class="card border round" style="display: none;" />

                <div class="card secondary round border" style="width: 25rem;" id="forms">
                    <div id="error" class="mdnote note-error full" style="display: none;" />
                    <form class="full flex flex-column g-4" action="/api/auth/login" id="login-user">
                        <label for="uid"><b>{"Account ID"}</b></label>

                        <input
                            type="text"
                            name="uid"
                            id="uid"
                            placeholder="00000000-0000-0000-0000-000000000000"
                            class="full round"
                            required={true}
                            minlength={36}
                            maxlength={36}
                        />

                        <hr />

                        <button class="bundles-primary full round">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-key-round"><path d="M2 18v3c0 .6.4 1 1 1h4v-3h3v-3h2l1.4-1.4a6.5 6.5 0 1 0-4-4Z"/><circle cx="16.5" cy="7.5" r=".5"/></svg>
                            {"Login"}
                        </button>
                    </form>
                </div>

                <script type="module">
                    {"import AuthPages from \"/static/js/AuthPages.js\";"}
                </script>

                <Footer auth_state={Option::None} />
            </main>
        </div>
    };
}

#[get("/d/auth/register")]
/// Available at "/d/auth/register"
/// Still renders even if `REGISTRATION_DISABLED` is present
pub async fn register_request(req: HttpRequest) -> impl Responder {
    if req.cookie("__Secure-Token").is_some() {
        return HttpResponse::NotFound().body("You're already signed in.");
    }

    // ...
    let renderer = ServerRenderer::<Register>::new();
    return HttpResponse::Ok()
        .append_header(("Content-Type", "text/html"))
        .body(format_html(
            renderer.render().await,
            "<title>Register - ::SITE_NAME::</title>",
        ));
}

#[get("/d/auth/login")]
/// Available at "/d/auth/login"
pub async fn login_request(req: HttpRequest) -> impl Responder {
    if req.cookie("__Secure-Token").is_some() {
        return HttpResponse::NotFound().body("You're already signed in.");
    }

    // ...
    let renderer = ServerRenderer::<Login>::new();
    return HttpResponse::Ok()
        .append_header(("Content-Type", "text/html"))
        .body(format_html(
            renderer.render().await,
            "<title>Login - ::SITE_NAME::</title>",
        ));
}

#[function_component]
fn ProfileView(props: &Props) -> Html {
    let meta = serde_json::from_str::<UserMetadata>(&props.user.metadata).unwrap();

    let can_edit = props.active_user.is_some()
        && props.active_user.as_ref().unwrap().username == props.user.username;

    return html! {
        <div class="flex flex-column" style="height: 100dvh;">
            <div class="toolbar flex justify-space-between">
                // left
                <div class="flex">
                    <a class="button" href="/" style="border-left: 0">
                        <b>{"::SITE_NAME::"}</b>
                    </a>

                    <a class="button" href={format!("/~{}", props.user.username)} style="border-left: 0">
                        {&props.user.username}
                    </a>
                </div>

                // right
                <div class="flex"></div>
            </div>

            <div class="toolbar-layout-wrapper">
                <main class="small flex flex-column g-4">
                    <div id="error" class="mdnote note-error full" style="display: none;" />
                    <div id="success" class="mdnote note-note full" style="display: none;" />

                    <div class="flex justify-space-between align-center">
                        <h1 class="no-margin">{&props.user.username}</h1>

                        // must not be receiver and must still be authenticated
                        {if (can_edit == false) && (props.auth_state.is_some()) && (props.auth_state.unwrap() == true) {
                            html! {
                                <div class="flex flex-wrap g-4">
                                    <button class="round bundles-primary" id="follow-user" data-endpoint={format!("/api/auth/users/{}/follow", &props.user.username)}>
                                        {if props.is_following == false {
                                            "Follow"
                                        } else {
                                            "Unfollow"
                                        }}
                                    </button>
                                    // direct messages will likely just be all within the same board with "is_private" enabled
                                    // <button class="round bundles-primary" disabled={true}>{"Message"}</button>
                                </div>
                            }
                        } else {
                            html! {}
                        }}
                    </div>

                    <div class="card secondary round">
                        <ul>
                            <li>{"Role: "}<span class="chip badge">{&props.user.role}</span></li>
                            <li>{"Joined: "}<span class="date-time-to-localize">{&props.user.timestamp}</span></li>
                            <li>{"Paste count: "}{&props.paste_count}</li>
                            <li>{"Board count: "}{&props.board_count}</li>
                            <li>{"Followers: "}{&props.follower_count}</li>
                        </ul>

                        <hr />

                        <div class="flex flex-column g-4">
                            <div class="card round" id="description">
                                {if props.edit_mode == false {
                                    // view mode
                                    // TODO: maybe store meta.about in meta.about_html and update it when profile about updates...
                                    let content = Html::from_html_unchecked(AttrValue::from(
                                        crate::markdown::render::parse_markdown(&meta.about)
                                    ));

                                    html! { {content} }
                                } else {
                                    // edit mode
                                    html! { <form id="edit-about" class="flex flex-column g-4" data-endpoint={format!("/api/auth/users/{}/about", &props.user.username)}>
                                        <div class="full flex justify-space-between align-center g-4">
                                            <b>{"Edit About"}</b>

                                            <button class="bundles-primary round">
                                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-save"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
                                                {"Save"}
                                            </button>
                                        </div>

                                        <textarea
                                            type="text"
                                            name="about"
                                            id="about"
                                            placeholder="About"
                                            class="full round"
                                            value={meta.about}
                                            minlength={2}
                                            maxlength={200_000}
                                            required={true}
                                        ></textarea>
                                    </form> }
                                }}

                                {if (can_edit == true) && (props.edit_mode == false) {
                                    html! { <a class="button round bundles-primary" href="?edit=true">{"Edit About"}</a> }
                                } else {
                                    html! {}
                                }}
                            </div>
                        </div>
                    </div>

                    <script type="module">
                        {"import \"/static/js/ProfileView.js\";"}
                    </script>

                    <Footer auth_state={props.auth_state} />
                </main>
            </div>
        </div>
    };
}

fn build_renderer_with_props(props: Props) -> ServerRenderer<ProfileView> {
    ServerRenderer::<ProfileView>::with_props(|| props)
}

#[get("/~{username:.*}")]
/// Available at "/~{username}"
pub async fn profile_view_request(
    req: HttpRequest,
    data: web::Data<AppData>,
    info: web::Query<QueryProps>,
) -> impl Responder {
    // get paste
    let username: String = req.match_info().get("username").unwrap().to_string();
    let username_c = username.clone();

    let user: bundlesdb::DefaultReturn<Option<UserState<String>>> =
        data.db.get_user_by_username(username).await;

    if user.success == false {
        let renderer = ServerRenderer::<crate::pages::errors::_404Page>::new();
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "text/html"))
            .body(utility::format_html(
                renderer.render().await,
                "<title>404: Not Found</title>",
            ));
    }

    let unwrap = user.payload.as_ref().unwrap();

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
    let pastes_res: bundlesdb::DefaultReturn<Option<Vec<bundlesdb::PasteIdentifier>>> =
        data.db.get_pastes_by_owner(username_c.clone()).await;

    let boards_res: bundlesdb::DefaultReturn<Option<Vec<bundlesdb::BoardIdentifier>>> =
        data.db.get_boards_by_owner(username_c.clone()).await;

    let followers_res: bundlesdb::DefaultReturn<usize> =
        data.db.get_user_follow_count(username_c.clone()).await;

    let is_following_res: Option<bundlesdb::DefaultReturn<Option<Log>>> = if token_user.is_some() {
        Option::Some(
            data.db
                .get_follow_by_user(
                    token_user
                        .as_ref()
                        .unwrap()
                        .payload
                        .as_ref()
                        .unwrap()
                        .username
                        .clone(),
                    username_c.clone(),
                )
                .await,
        )
    } else {
        Option::None
    };

    let renderer = build_renderer_with_props(Props {
        user: unwrap.clone(),
        paste_count: if pastes_res.success {
            pastes_res.payload.unwrap().len()
        } else {
            0
        },
        board_count: if boards_res.success {
            boards_res.payload.unwrap().len()
        } else {
            0
        },
        auth_state: if req.cookie("__Secure-Token").is_some() {
            Option::Some(req.cookie("__Secure-Token").is_some())
        } else {
            Option::Some(false)
        },
        active_user: if token_user.is_some() {
            Option::Some(token_user.unwrap().payload.unwrap())
        } else {
            Option::None
        },
        edit_mode: if info.edit.is_some() {
            info.edit.unwrap()
        } else {
            false
        },
        follower_count: followers_res.payload,
        is_following: if is_following_res.is_some() {
            is_following_res.unwrap().payload.is_some()
        } else {
            false
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
                <meta property=\"og:description\" content=\"{}\" />",
                &username_c,
                &format!(
                    "{}{}",
                    req.headers().get("Host").unwrap().to_str().unwrap(),
                    req.head().uri.to_string()
                ),
                // extras
                &username_c,
                format!("{} on ::SITE_NAME::", &username_c)
            ),
        ));
}
