use actix_web::HttpRequest;
use actix_web::{get, web, HttpResponse, Responder};

use yew::prelude::*;
use yew::ServerRenderer;

use crate::components::navigation::Footer;
use crate::db::bundlesdb::{DefaultReturn, FullUser};
use crate::db::{self, bundlesdb};
use crate::utility::format_html;

use crate::pages::boards::ViewBoardQueryProps;

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct DashboardProps {
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct BoardsProps {
    pub offset: i32,
    pub posts: Vec<bundlesdb::Log>,
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct UsersProps {
    pub username: String,
    pub user: Option<FullUser<String>>,
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
pub struct UsersQueryProps {
    pub username: Option<String>,
}

#[function_component]
fn Dashboard(props: &DashboardProps) -> Html {
    return html! {
        <div class="flex flex-column" style="height: 100dvh;">
            <div class="toolbar flex justify-space-between">
                // left
                <div class="flex">
                    <a class="button" href="/" style="border-left: 0">
                        <b>{"::SITE_NAME::"}</b>
                    </a>

                    <a class="button" href="/d/staff" style="border-left: 0">
                        {"Staff"}
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
                        <a href="/d/staff" class="button active">{"Home"}</a>
                        <a href="/d/staff/users" class="button">{"Users"}</a>
                        <a href="/d/staff/boards" class="button">{"Boards"}</a>
                    </div>
                </div>

                <main class="small flex flex-column g-4">

                    <Footer auth_state={props.auth_state} />
                </main>
            </div>
        </div>
    };
}

fn build_dashboard_renderer_with_props(props: DashboardProps) -> ServerRenderer<Dashboard> {
    ServerRenderer::<Dashboard>::with_props(|| props)
}

#[get("/d/staff")]
/// Available at "/d/staff"
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
        // you must have an account to use the staff dashboard
        return HttpResponse::NotFound().body(
            "You must have an account to use the staff dashboard.
You can login at: /d/auth/login
You can create an account at: /d/auth/register",
        );
    }

    // validate role
    let user = token_user.unwrap().payload.unwrap();
    if !user
        .level
        .permissions
        .contains(&String::from("StaffDashboard"))
    {
        return HttpResponse::NotFound().body("You do not have permission to do this");
    }

    // ...
    let renderer = build_dashboard_renderer_with_props(DashboardProps {
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
            "<title>Staff Dashboard - ::SITE_NAME::</title>",
        ));
}

#[function_component]
fn BoardsDashboard(props: &BoardsProps) -> Html {
    return html! {
        <div class="flex flex-column" style="height: 100dvh;">
            <div class="toolbar flex justify-space-between">
                // left
                <div class="flex">
                    <a class="button" href="/" style="border-left: 0">
                        <b>{"::SITE_NAME::"}</b>
                    </a>

                    <a class="button" href="/d/staff" style="border-left: 0">
                        {"Staff"}
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
                        <a href="/d/staff" class="button">{"Home"}</a>
                        <a href="/d/staff/users" class="button">{"Users"}</a>
                        <a href="/d/staff/boards" class="button active">{"Boards"}</a>
                    </div>
                </div>

                <main class="small flex flex-column g-4">
                    <b>{"Recent Posts"}</b>

                    <div class="full flex justify-space-between" id="pages">
                        <a class="button round" href={format!("?offset={}", props.offset - 50)} disabled={props.offset <= 0}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-left"><path d="m12 19-7-7 7-7"/><path d="M19 12H5"/></svg>
                            {"Back"}
                        </a>

                        <a class="button round" href={format!("?offset={}", props.offset + 50)} disabled={props.posts.len() == 0}>
                            {"Next"}
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-right"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>
                        </a>
                    </div>

                    <div class="card round secondary flex g-4 flex-column justify-center" id="boards_list">
                        {for props.posts.iter().map(|p| {
                            let post = serde_json::from_str::<bundlesdb::BoardPostLog>(&p.content).unwrap();

                            html! {
                                <a class="button secondary round full justify-start" href={format!("/b/{}/posts/{}", &post.board, &p.id)}>
                                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-message-square-text"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/><path d="M13 8H7"/><path d="M17 12H7"/></svg>
                                    {&post.board}

                                    <span style="opacity: 75%;" class="date-time-to-localize">
                                        {&p.timestamp}
                                    </span>
                                </a>
                            }
                        })}
                    </div>

                    <Footer auth_state={props.auth_state} />
                </main>
            </div>
        </div>
    };
}

fn build_boards_dashboard_renderer_with_props(
    props: BoardsProps,
) -> ServerRenderer<BoardsDashboard> {
    ServerRenderer::<BoardsDashboard>::with_props(|| props)
}

#[get("/d/staff/boards")]
/// Available at "/d/staff/boards"
pub async fn staff_boards_dashboard_request(
    req: HttpRequest,
    data: web::Data<db::bundlesdb::AppData>,
    info: web::Query<ViewBoardQueryProps>,
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
        // you must have an account to use the staff dashboard
        return HttpResponse::NotFound().body(
            "You must have an account to use the staff dashboard.
You can login at: /d/auth/login
You can create an account at: /d/auth/register",
        );
    }

    // validate role
    let user = token_user.unwrap().payload.unwrap();

    if !user
        .level
        .permissions
        .contains(&String::from("StaffDashboard"))
    {
        return HttpResponse::NotFound().body("You do not have permission to do this");
    }

    // get posts
    let posts: bundlesdb::DefaultReturn<Option<Vec<bundlesdb::Log>>> =
        data.db.fetch_most_recent_posts(info.offset).await;

    // ...
    let renderer = build_boards_dashboard_renderer_with_props(BoardsProps {
        offset: if info.offset.is_some() {
            info.offset.unwrap()
        } else {
            0
        },
        posts: posts.payload.unwrap(),
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
            "<title>Staff Dashboard - ::SITE_NAME::</title>",
        ));
}

#[function_component]
fn UsersDashboard(props: &UsersProps) -> Html {
    return html! {
        <div class="flex flex-column" style="height: 100dvh;">
            <div class="toolbar flex justify-space-between">
                // left
                <div class="flex">
                    <a class="button" href="/" style="border-left: 0">
                        <b>{"::SITE_NAME::"}</b>
                    </a>

                    <a class="button" href="/d/staff" style="border-left: 0">
                        {"Staff"}
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
                        <a href="/d/staff" class="button">{"Home"}</a>
                        <a href="/d/staff/users" class="button active">{"Users"}</a>
                        <a href="/d/staff/boards" class="button">{"Boards"}</a>
                    </div>
                </div>

                <main class="small flex flex-column g-4">
                    <div id="error" class="mdnote note-error full" style="display: none;" />
                    <div id="success" class="mdnote note-note full" style="display: none;" />

                    <div class="flex justify-space-between align-center mobile:flex-column g-4">
                        <b>{"Manage Users"}</b>

                        <form style="width: 50%;" class="flex g-4 mobile:max">
                            <input
                                type="text"
                                name="username"
                                id="username"
                                placeholder="Username"
                                class="round"
                                value={props.username.clone()}
                                maxlength={250}
                                style="width: calc(100% - 50px);"
                            />

                            <button class="round bundles-primary" style="width: 50px;">{"Go"}</button>
                        </form>
                    </div>

                    {if props.user.is_some() {
                        let user = props.user.as_ref().unwrap();

                        if user.level.permissions.contains(&String::from("StaffDashboard")) {
                            return html! { <span>{"Cannot manage users of role \"staff\""}</span> };
                        }

                        html! {
                            <div class="card full round secondary flex flex-column g-4">
                                <h4 class="no-margin">{&user.user.username}</h4>

                                <hr />

                                <div class="flex full g-4 flex-wrap justify-space-between align-center">
                                    <h6 class="no-margin">{"Banhammer"}</h6>
                                    <button class="round bundles-primary" id="hammer-time" data-endpoint={format!("/api/auth/users/{}/ban", &user.user.username)}>{"Ban User"}</button>
                                </div>
                            </div>
                        }
                    } else {
                        html! {}
                    }}

                    <script type="module">
                        {"import \"/static/js/SDManageUser.js\";"}
                    </script>

                    <Footer auth_state={props.auth_state} />
                </main>
            </div>
        </div>
    };
}

fn build_users_dashboard_renderer_with_props(props: UsersProps) -> ServerRenderer<UsersDashboard> {
    ServerRenderer::<UsersDashboard>::with_props(|| props)
}

#[get("/d/staff/users")]
/// Available at "/d/staff/users"
pub async fn staff_users_dashboard_request(
    req: HttpRequest,
    data: web::Data<db::bundlesdb::AppData>,
    info: web::Query<UsersQueryProps>,
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
        // you must have an account to use the staff dashboard
        return HttpResponse::NotFound().body(
            "You must have an account to use the staff dashboard.
You can login at: /d/auth/login
You can create an account at: /d/auth/register",
        );
    }

    // validate role
    let user = token_user.unwrap().payload.unwrap();

    if !user
        .level
        .permissions
        .contains(&String::from("StaffDashboard"))
    {
        return HttpResponse::NotFound().body("You do not have permission to do this");
    }

    // get user
    let user: bundlesdb::DefaultReturn<Option<FullUser<String>>> = if info.username.is_some() {
        data.db
            .get_user_by_username(info.username.as_ref().unwrap().to_owned())
            .await
    } else {
        DefaultReturn {
            success: false,
            message: String::new(),
            payload: Option::None,
        }
    };

    // ...
    let renderer = build_users_dashboard_renderer_with_props(UsersProps {
        username: if info.username.is_some() {
            info.username.as_ref().unwrap().to_owned()
        } else {
            String::new()
        },
        user: user.payload,
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
            "<title>Staff Dashboard - ::SITE_NAME::</title>",
        ));
}
