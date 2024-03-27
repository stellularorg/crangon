use actix_web::HttpResponse;
use actix_web::{get, web, HttpRequest, Responder};

use serde_json::json;
use yew::prelude::*;
use yew::ServerRenderer;

use crate::db::bundlesdb::{self, AppData, FullUser, UserFollow, UserMetadata, UserState};
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
    pub following_count: usize,
    pub post_count: usize,
    pub is_following: bool,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
pub struct QueryProps {
    pub edit: Option<bool>, // Props.edit_mode
}

#[derive(Default, Properties, PartialEq)]
struct FollowersProps {
    pub user: UserState<String>,
    pub followers: Vec<bundlesdb::Log>,
    pub offset: i32,
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
pub struct FollowersQueryProps {
    pub offset: Option<i32>,
}

#[derive(Default, Properties, PartialEq)]
struct FollowingProps {
    pub user: UserState<String>,
    pub following: Vec<bundlesdb::Log>,
    pub offset: i32,
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
pub struct FollowingQueryProps {
    pub offset: Option<i32>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct SettingsProps {
    pub profile: UserState<String>,
    pub auth_state: Option<bool>,
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

                <a class="button round border" href="/d/auth/login-st" style="width: 25rem;" id="switch-button">{"Use Secondary Token"}</a>

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

#[function_component]
fn LoginSecondaryToken() -> Html {
    return html! {
        <div class="flex flex-column g-4" style="height: 100dvh;">
            <main class="small flex flex-column align-center g-8">
                <div id="success" class="card border round" style="display: none;" />

                <div class="card secondary round border" style="width: 25rem;" id="forms">
                    <div id="error" class="mdnote note-error full" style="display: none;" />
                    <form class="full flex flex-column g-4" action="/api/auth/login-st" id="login-user-st">
                        <label for="uid"><b>{"Secondary Token"}</b></label>

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

                <a class="button round border" href="/d/auth/login" style="width: 25rem;" id="switch-button">{"Use Account ID"}</a>

                <script type="module">
                    {"import AuthPages from \"/static/js/AuthPages.js\";"}
                </script>

                <Footer auth_state={Option::None} />
            </main>
        </div>
    };
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

#[get("/d/auth/login-st")]
/// Available at "/d/auth/login-st"
pub async fn login_secondary_token_request(req: HttpRequest) -> impl Responder {
    if req.cookie("__Secure-Token").is_some() {
        return HttpResponse::NotFound().body("You're already signed in.");
    }

    // ...
    let renderer = ServerRenderer::<LoginSecondaryToken>::new();
    return HttpResponse::Ok()
        .append_header(("Content-Type", "text/html"))
        .body(format_html(
            renderer.render().await,
            "<title>Login - ::SITE_NAME::</title>",
        ));
}

pub fn profile_view_hb_template() -> String {
    String::from("<main class=\"small flex flex-column g-4\">
    <div id=\"error\" class=\"mdnote note-error full\" style=\"display: none\"></div>
    <div id=\"success\" class=\"mdnote note-note full\" style=\"display: none\"></div>

    <div
        class=\"flex justify-space-between align-center mobile:flex-column g-4 flex-wrap\"
    >
        <div class=\"flex align-center g-4 flex-wrap\" style=\"max-width: 100%\">
            {{{ avatar }}} {{{ username_display }}}
        </div>

        {{{ user_actions }}}
    </div>

    <div class=\"card secondary round\">
        <div id=\"stats-or-info\" class=\"flex flex-column g-4\">
            <details class=\"round border\" open>
                <summary>Info</summary>

                <table class=\"full\" style=\"margin: 0\">
                    <thead>
                        <tr>
                            <th>Key</th>
                            <th>Value</th>
                        </tr>
                    </thead>

                    <tbody>
                        <tr>
                            <td>Level</td>
                            <td>{{{ level_badge }}}</td>
                        </tr>
                        <tr>
                            <td>Joined</td>
                            <td>
                                <span class=\"date-time-to-localize\">
                                    {{ user.timestamp }}
                                </span>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </details>

            <details class=\"round border\">
                <summary>Statistics</summary>

                <table class=\"full\" style=\"margin: 0\">
                    <thead>
                        <tr>
                            <th>Key</th>
                            <th>Value</th>
                        </tr>
                    </thead>

                    <tbody>
                        <tr>
                            <td>Pastes</td>
                            <td>{{ paste_count }}</td>
                        </tr>

                        <tr>
                            <td>Boards</td>
                            <td>{{ board_count }}</td>
                        </tr>

                        <tr>
                            <td>Posts</td>
                            <td>{{ post_count }}</td>
                        </tr>

                        <tr>
                            <td>Followers</td>
                            <td>{{{ followers_button }}}</td>
                        </tr>

                        <tr>
                            <td>Following</td>
                            <td>{{{ following_button }}}</td>
                        </tr>
                    </tbody>
                </table>
            </details>
        </div>

        <hr />

        <div class=\"flex flex-column g-4\">
            <div class=\"card round\" id=\"description\">
                {{{ about }}}
                {{{ edit_about_button }}}
            </div>
        </div>
    </div>
</main>")
}

#[function_component]
fn ProfileView(props: &Props) -> Html {
    let meta = serde_json::from_str::<UserMetadata>(&props.user.metadata).unwrap();

    // ...
    let can_edit = props.active_user.is_some()
        && props.active_user.as_ref().unwrap().username == props.user.username;

    // template
    let avatar = format!(
        "<img
        class=\"avatar\"
        style=\"--size: {}px;\"
        src=\"/api/auth/users/{}/avatar\"
    />",
        60, props.user.username
    );

    let username_c = props.user.username.clone();
    let username_display = format!(
        "<div class=\"flex flex-column\" style=\"max-width: 100%; min-width: max-content\">
        <h2 class=\"no-margin\" id=\"user-fake-name\" style=\"max-width: 100vw\">{}</h2>
    
        <span id=\"user-real-name\">{}</span>
    </div>",
        if meta.nickname.is_some() {
            meta.nickname.as_ref().unwrap()
        } else {
            &username_c
        },
        props.user.username
    );

    let about = if props.edit_mode == true {
        // edit mode form
        format!("<form id=\"edit-about\" class=\"flex flex-column g-4\" data-endpoint=\"/api/auth/users/{}/about\">
            <div class=\"full flex justify-space-between align-center g-4\">
                <b>Edit About</b>
        
                <button class=\"bundles-primary round\">
                    <svg xmlns=\"http://www.w3.org/2000/svg\" width=\"18\" height=\"18\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" class=\"lucide lucide-save\"><path d=\"M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z\"/><polyline points=\"17 21 17 13 7 13 7 21\"/><polyline points=\"7 3 7 8 15 8\"/></svg>
                    Save
                </button>
            </div>
        
            <textarea
                type=\"text\"
                name=\"about\"
                id=\"about\"
                placeholder=\"About\"
                class=\"full round\"
                minlength=\"2\"
                maxlength=\"200000\"
                required
            >{}</textarea>
        </form>", props.user.username, meta.about)
    } else {
        // just show about
        crate::markdown::render::parse_markdown(&meta.about.clone())
    };

    let edit_about_button = if can_edit == true && props.edit_mode == false {
        "<a class=\"button round bundles-primary\" href=\"?edit=true\">Edit About</a>"
    } else {
        ""
    };

    let followers_button = format!(
        "<a href=\"/~{}/followers\">{}</a>",
        props.user.username, props.follower_count
    );

    let following_button = format!(
        "<a href=\"/~{}/following\">{}</a>",
        props.user.username, props.following_count
    );

    let user_actions = format!(
        "{}",
        if (can_edit == false)
            && (props.auth_state.is_some())
            && (props.auth_state.unwrap() == true)
        {
            format!(
                "<div class=\"flex flex-wrap g-4\">
                    <button class=\"round bundles-primary\" id=\"mail-user\" data-endpoint=\"/api/auth/users/{}/mail\">Mail</button>
                    <button class=\"round bundles-primary\" id=\"follow-user\" data-endpoint=\"/api/auth/users/{}/follow\">{}</button>
                </div>", 
                props.user.username, 
                props.user.username, 
                if props.is_following == false {
                    "Follow"
                } else {
                    "Unfollow"
                }
            )
        } else {
            String::new()
        }
    );

    let level_badge = format!("<span class=\"chip badge role-{}\">{}</span>", props.user.role, props.user.role);

    // render template
    let default_template = &profile_view_hb_template();
    let reg = handlebars::Handlebars::new();
    let page = reg.render_template(
        if meta.page_template.is_some() {
            meta.page_template.as_ref().unwrap() // use provided template
        } else {
            default_template // use default template
        },
        &json!({
            // user info
            "username": props.user.username,
            "about": about,
            "paste_count": props.paste_count,
            "board_count": props.board_count,
            "post_count": props.post_count,
            // components
            "avatar": avatar,
            "username_display": username_display,
            "edit_about_button": edit_about_button,
            "followers_button": followers_button,
            "following_button": following_button,
            "user_actions": user_actions,
            "level_badge": level_badge,
            // ...
            "user": props.user,
            "metadata": meta.clone()
        }),
    );

    if page.is_err() {
        return html! { <div>{page.err().unwrap().to_string()}</div> };
    }

    // ...
    // TODO: properly sanitize if needed
    let page =
        Html::from_html_unchecked(AttrValue::from(page.unwrap().replace("fetch(", "fetch(\\")));

    // ...
    return html! {
        <div class="flex flex-column" style="height: 100dvh;">
            <div class="toolbar flex justify-space-between">
                // left
                <div class="flex">
                    <a class="button device:desktop" href="/" style="border-left: 0">
                        <b>{"::SITE_NAME::"}</b>
                    </a>

                    <a class="button" href={format!("/~{}", props.user.username)} style="border-left: 0">
                        {&props.user.username}
                        <span class={format!("chip badge role-{}", props.user.role)} style="padding: 0 0.5rem;">{&props.user.role}</span>
                    </a>
                </div>

                // right
                <div class="flex">
                    <a class="button" href={format!("/~{}/settings", props.user.username)} title="Synced User Settings">
                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-file-cog"><path d="M4 22h14a2 2 0 0 0 2-2V7l-5-5H6a2 2 0 0 0-2 2v2"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/><circle cx="6" cy="14" r="3"/><path d="M6 10v1"/><path d="M6 17v1"/><path d="M10 14H9"/><path d="M3 14H2"/><path d="m9 11-.88.88"/><path d="M3.88 16.12 3 17"/><path d="m9 17-.88-.88"/><path d="M3.88 11.88 3 11"/></svg>
                    </a>
                </div>
            </div>

            <div class="toolbar-layout-wrapper">
                {page}

                <main class="small">
                    <Footer auth_state={props.auth_state} />
                </main>

                <script type="module">
                    {"import \"/static/js/ProfileView.js\";"}
                </script>
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

    let user: bundlesdb::DefaultReturn<Option<FullUser<String>>> =
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

    // ...
    let pastes_res: bundlesdb::DefaultReturn<Option<Vec<bundlesdb::PasteIdentifier>>> =
        data.db.get_pastes_by_owner(username_c.clone()).await;

    let boards_res: bundlesdb::DefaultReturn<Option<Vec<bundlesdb::BoardIdentifier>>> =
        data.db.get_boards_by_owner(username_c.clone()).await;

    let followers_res: bundlesdb::DefaultReturn<usize> =
        data.db.get_user_follow_count(username_c.clone()).await;

    let following_res: bundlesdb::DefaultReturn<usize> =
        data.db.get_user_following_count(username_c.clone()).await;

    let posts_res: bundlesdb::DefaultReturn<usize> =
        data.db.get_user_posts_count(username_c.clone()).await;

    let is_following_res: Option<bundlesdb::DefaultReturn<Option<bundlesdb::Log>>> =
        if token_user.is_some() && token_user.as_ref().unwrap().success {
            Option::Some(
                data.db
                    .get_follow_by_user(
                        token_user
                            .as_ref()
                            .unwrap()
                            .payload
                            .as_ref()
                            .unwrap()
                            .user
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
        user: unwrap.clone().user,
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
        active_user: if token_user.is_some() && token_user.as_ref().unwrap().success {
            Option::Some(token_user.unwrap().payload.unwrap().user)
        } else {
            Option::None
        },
        edit_mode: if info.edit.is_some() {
            info.edit.unwrap()
        } else {
            false
        },
        follower_count: followers_res.payload,
        following_count: following_res.payload,
        post_count: posts_res.payload,
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

#[function_component]
fn FollowersView(props: &FollowersProps) -> Html {
    html! {
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
                <div class="flex">
                    <a class="button" href={format!("/~{}", props.user.username)} style="border-right: 0">{"Home"}</a>
                </div>
            </div>

            <div class="toolbar-layout-wrapper">
                <main class="small flex flex-column g-4">
                    <div id="error" class="mdnote note-error full" style="display: none;" />
                    <div id="success" class="mdnote note-note full" style="display: none;" />

                    <div class="flex justify-space-between align-center">
                        <h3 class="no-margin">{&props.user.username}{"'s followers"}</h3>
                    </div>

                    <div class="card secondary round flex flex-column g-4">
                        {for props.followers.iter().map(|u| {
                            let follow_log = serde_json::from_str::<UserFollow>(&u.content).unwrap();

                            html! {
                                <a class="button secondary full round justify-space-between flex-wrap" href={format!("/~{}", follow_log.user)} style="height: max-content !important;">
                                    <span class="flex align-center g-4">
                                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-circle-user-round"><path d="M18 20a6 6 0 0 0-12 0"/><circle cx="12" cy="10" r="4"/><circle cx="12" cy="12" r="10"/></svg>
                                        {follow_log.user}
                                    </span>

                                    <span style="opacity: 75%;">{"Followed "}<span class="date-time-to-localize">{u.timestamp}</span></span>
                                </a>
                            }
                        })}
                    </div>

                    <div class="full flex justify-space-between" id="pages">
                        <a class="button round" href={format!("?offset={}", props.offset - 50)} disabled={props.offset <= 0}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-left"><path d="m12 19-7-7 7-7"/><path d="M19 12H5"/></svg>
                            {"Back"}
                        </a>

                        <a class="button round" href={format!("?offset={}", props.offset + 50)} disabled={props.followers.len() == 0}>
                            {"Next"}
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-right"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>
                        </a>
                    </div>

                    <Footer auth_state={props.auth_state} />
                </main>
            </div>
        </div>
    }
}

fn build_followers_renderer_with_props(props: FollowersProps) -> ServerRenderer<FollowersView> {
    ServerRenderer::<FollowersView>::with_props(|| props)
}

#[get("/~{username:.*}/followers")]
/// Available at "/~{username}/followers"
pub async fn followers_request(
    req: HttpRequest,
    data: web::Data<AppData>,
    info: web::Query<FollowersQueryProps>,
) -> impl Responder {
    // get paste
    let username: String = req.match_info().get("username").unwrap().to_string();
    let username_c = username.clone();

    let user: bundlesdb::DefaultReturn<Option<FullUser<String>>> =
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
    }

    // ...
    let followers_res: bundlesdb::DefaultReturn<Option<Vec<bundlesdb::Log>>> = data
        .db
        .get_user_followers(username_c.clone(), info.offset)
        .await;

    let renderer = build_followers_renderer_with_props(FollowersProps {
        user: unwrap.clone().user,
        followers: followers_res.payload.unwrap(),
        offset: if info.offset.is_some() {
            info.offset.unwrap()
        } else {
            0
        },
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
                <meta property=\"og:description\" content=\"{}\" />",
                &username_c,
                &format!(
                    "{}{}",
                    req.headers().get("Host").unwrap().to_str().unwrap(),
                    req.head().uri.to_string()
                ),
                // extras
                &username_c,
                format!("{}'s followers on ::SITE_NAME::", &username_c)
            ),
        ));
}

#[function_component]
fn FollowingView(props: &FollowingProps) -> Html {
    html! {
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
                <div class="flex">
                    <a class="button" href={format!("/~{}", props.user.username)} style="border-right: 0">{"Home"}</a>
                </div>
            </div>

            <div class="toolbar-layout-wrapper">
                <main class="small flex flex-column g-4">
                    <div id="error" class="mdnote note-error full" style="display: none;" />
                    <div id="success" class="mdnote note-note full" style="display: none;" />

                    <div class="flex justify-space-between align-center">
                        <h3 class="no-margin">{&props.user.username}{"'s following"}</h3>
                    </div>

                    <div class="card secondary round flex flex-column g-4">
                        {for props.following.iter().map(|u| {
                            let follow_log = serde_json::from_str::<UserFollow>(&u.content).unwrap();

                            html! {
                                <a class="button secondary full round justify-space-between flex-wrap" href={format!("/~{}", follow_log.is_following)} style="height: max-content !important;">
                                    <span class="flex align-center g-4">
                                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-circle-user-round"><path d="M18 20a6 6 0 0 0-12 0"/><circle cx="12" cy="10" r="4"/><circle cx="12" cy="12" r="10"/></svg>
                                        {follow_log.is_following}
                                    </span>

                                    <span style="opacity: 75%;">{"Followed "}<span class="date-time-to-localize">{u.timestamp}</span></span>
                                </a>
                            }
                        })}
                    </div>

                    <div class="full flex justify-space-between" id="pages">
                        <a class="button round" href={format!("?offset={}", props.offset - 50)} disabled={props.offset <= 0}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-left"><path d="m12 19-7-7 7-7"/><path d="M19 12H5"/></svg>
                            {"Back"}
                        </a>

                        <a class="button round" href={format!("?offset={}", props.offset + 50)} disabled={props.following.len() == 0}>
                            {"Next"}
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-right"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>
                        </a>
                    </div>

                    <Footer auth_state={props.auth_state} />
                </main>
            </div>
        </div>
    }
}

fn build_following_renderer_with_props(props: FollowingProps) -> ServerRenderer<FollowingView> {
    ServerRenderer::<FollowingView>::with_props(|| props)
}

#[get("/~{username:.*}/following")]
/// Available at "/~{username}/following"
pub async fn following_request(
    req: HttpRequest,
    data: web::Data<AppData>,
    info: web::Query<FollowingQueryProps>,
) -> impl Responder {
    // get paste
    let username: String = req.match_info().get("username").unwrap().to_string();
    let username_c = username.clone();

    let user: bundlesdb::DefaultReturn<Option<FullUser<String>>> =
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
    }

    // ...
    let following_res: bundlesdb::DefaultReturn<Option<Vec<bundlesdb::Log>>> = data
        .db
        .get_user_following(username_c.clone(), info.offset)
        .await;

    let renderer = build_following_renderer_with_props(FollowingProps {
        user: unwrap.clone().user,
        following: following_res.payload.unwrap(),
        offset: if info.offset.is_some() {
            info.offset.unwrap()
        } else {
            0
        },
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
                <meta property=\"og:description\" content=\"{}\" />",
                &username_c,
                &format!(
                    "{}{}",
                    req.headers().get("Host").unwrap().to_str().unwrap(),
                    req.head().uri.to_string()
                ),
                // extras
                &username_c,
                format!("{}'s following on ::SITE_NAME::", &username_c)
            ),
        ));
}

#[function_component]
fn UserSettings(props: &SettingsProps) -> Html {
    let metadata = serde_json::from_str::<UserMetadata>(&props.profile.metadata).unwrap();

    return html! {
        <div>
            <div class="toolbar flex justify-space-between">
                // left
                <div class="flex">
                    <a class="button" href="/" style="border-left: 0">
                        <b>{"::SITE_NAME::"}</b>
                    </a>

                    <a class="button" href={format!("/~{}", props.profile.username)} style="border-left: 0">
                        {props.profile.username.clone()}
                    </a>
                </div>
            </div>

            <div class="toolbar-layout-wrapper">
                <main class="flex flex-column g-4 small">
                    <h2 class="full text-center">{"User Settings"}</h2>

                    <div class="card round secondary flex flex-column g-4">
                        <div class="flex full justify-space-between flex-wrap mobile:justify-center g-4">
                            <div class="flex g-4">
                                <form action="/api/metadata" id="update-form">
                                    <button class="green round secondary">
                                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-save"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
                                        {"Save"}
                                    </button>
                                </form>

                                <button class="secondary round" id="add_field">{"Add Field"}</button>
                            </div>

                            <div class="flex g-4">
                                // <button class="secondary round red" id="delete-user">{"Delete"}</button>
                                <a href={format!("/~{}", props.profile.username)} class="button round secondary">{"Cancel"}</a>
                            </div>
                        </div>

                        <div id="options-field" class="flex flex-wrap mobile:flex-column g-4 full justify-space-between" />
                    </div>

                    <script type="module">
                        {format!("import {{ paste_settings }} from \"/static/js/SettingsEditor.js\";
                        paste_settings({}, \"{}\", document.getElementById(\"options-field\"), \"user\");", serde_json::to_string(&metadata).unwrap(), &props.profile.username)}
                    </script>

                    <Footer auth_state={props.auth_state} />
                </main>
            </div>
        </div>
    };
}

fn build_settings_with_props(props: SettingsProps) -> ServerRenderer<UserSettings> {
    ServerRenderer::<UserSettings>::with_props(|| props)
}

#[get("/~{name:.*}/settings")]
/// Available at "/~{name}/settings"
pub async fn user_settings_request(
    req: HttpRequest,
    data: web::Data<bundlesdb::AppData>,
) -> impl Responder {
    // get user
    let name: String = req.match_info().get("name").unwrap().to_string();
    let name_c = name.clone();

    let profile: bundlesdb::DefaultReturn<Option<FullUser<String>>> =
        data.db.get_user_by_username(name).await;

    if profile.success == false {
        return HttpResponse::NotFound().body(profile.message);
    }

    let profile = profile.payload.unwrap();

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
        return HttpResponse::NotAcceptable().body("An account is required to do this");
    }

    // ...
    let user = token_user.unwrap().payload.unwrap();
    let can_view: bool = (user.user.username == profile.user.username)
        | (user
            .level
            .permissions
            .contains(&String::from("ManageUsers")));

    if can_view == false {
        return HttpResponse::NotFound()
            .body("You do not have permission to manage this user's contents.");
    }

    // ...
    let renderer = build_settings_with_props(SettingsProps {
        profile: profile.clone().user,
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
                <meta property=\"og:title\" content=\"{} (synced user settings) - ::SITE_NAME::\" />",
                &name_c, &name_c
            ),
        ));
}
