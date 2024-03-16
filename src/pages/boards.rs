use actix_web::HttpRequest;
use actix_web::{get, web, HttpResponse, Responder};

use yew::prelude::*;
use yew::ServerRenderer;

use crate::components::message::Message;
use crate::components::navigation::Footer;
use crate::db::bundlesdb::{Board, BoardMetadata, BoardPostLog, Log, UserState};
use crate::db::{self, bundlesdb};
use crate::utility::{self, format_html};

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct NewProps {
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct Props {
    pub board: Board<String>,
    pub posts: Vec<Log>,
    pub pinned: Vec<Log>,
    pub offset: i32,
    pub tags: String,
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
pub struct ViewBoardQueryProps {
    pub offset: Option<i32>,
    pub tags: Option<String>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct ViewPostProps {
    pub board: Board<String>,
    pub post: Log,
    pub replies: Vec<Log>,
    pub auth_state: Option<bool>,
    pub user: Option<UserState<String>>,
    pub edit: bool,
    pub edit_tags: bool,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
pub struct ViewBoardPostQueryProps {
    pub edit: Option<bool>,
    pub edit_tags: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct SettingsProps {
    pub board: Board<String>,
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct DashboardProps {
    pub boards: Vec<bundlesdb::BoardIdentifier>,
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct SearchProps {
    pub boards: Vec<bundlesdb::BoardIdentifier>,
    pub tags: String,
    pub offset: i32,
    pub auth_state: Option<bool>,
}

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
pub struct SearchQueryProps {
    pub offset: Option<i32>,
    pub tags: Option<String>,
}

#[function_component]
fn CreateNew(props: &NewProps) -> Html {
    return html! {
        <div class="flex flex-column g-4" style="height: 100dvh;">
            <main class="small flex flex-column g-4 align-center">
                <div class="card secondary round border" style="width: 25rem;" id="forms">
                    <div id="error" class="mdnote note-error full" style="display: none;" />
                    <form class="full flex flex-column g-4" action="/api/board/new" id="create-board">
                        <label for="_name"><b>{"Name"}</b></label>

                        <input
                            type="text"
                            name="_name"
                            id="_name"
                            placeholder="Name"
                            class="full round"
                            minlength={4}
                            maxlength={32}
                            required={true}
                        />

                        <hr />

                        <button class="bundles-primary full round">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-plus"><path d="M5 12h14"/><path d="M12 5v14"/></svg>
                            {"Create Board"}
                        </button>
                    </form>
                </div>

                <script type="module">
                    {"import \"/static/js/NewBoard.js\";"}
                </script>

                <Footer auth_state={props.auth_state} />
            </main>
        </div>
    };
}

fn build_new_renderer_with_props(props: NewProps) -> ServerRenderer<CreateNew> {
    ServerRenderer::<CreateNew>::with_props(|| props)
}

#[get("/b/new")]
/// Available at "/b/new"
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
        // you must have an account to create boards
        return HttpResponse::NotFound().body(
            "You must have an account to create a board.
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
            "<title>New Board - ::SITE_NAME::</title>",
        ));
}

#[function_component]
fn ViewBoard(props: &Props) -> Html {
    let board_m = serde_json::from_str::<BoardMetadata>(&props.board.metadata).unwrap();

    let can_post_from_anonymous =
        board_m.allow_anonymous.is_none() || board_m.allow_anonymous.unwrap() != String::from("no");

    // ...
    return html! {
        <div class="flex flex-column" style="height: 100dvh;">
            <div style="display: none;" id="board-name">{&props.board.name}</div>

            <div class="toolbar flex justify-space-between">
                // left
                <div class="flex">
                    <a class="button" href="/" style="border-left: 0">
                        <b>{"::SITE_NAME::"}</b>
                    </a>

                    <a class="button" href={format!("/b/{}", props.board.name)} style="border-left: 0">
                        {props.board.name.clone()}
                    </a>
                </div>

                // right
                <div class="flex">
                    <a class="button" href={format!("/b/{}/manage", props.board.name)} title="Manage Board">
                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-file-cog"><path d="M4 22h14a2 2 0 0 0 2-2V7l-5-5H6a2 2 0 0 0-2 2v2"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/><circle cx="6" cy="14" r="3"/><path d="M6 10v1"/><path d="M6 17v1"/><path d="M10 14H9"/><path d="M3 14H2"/><path d="m9 11-.88.88"/><path d="M3.88 16.12 3 17"/><path d="m9 17-.88-.88"/><path d="M3.88 11.88 3 11"/></svg>
                    </a>
                </div>
            </div>

            <div class="toolbar-layout-wrapper">
                <main class="flex flex-column g-4 align-center">
                    <div class="full" id="about">
                        {if board_m.about.is_some() {
                            let content = Html::from_html_unchecked(AttrValue::from(
                                crate::markdown::render::parse_markdown(&board_m.about.unwrap())
                            ));

                            html! {{content}}
                        } else {
                            html! {}
                        }}

                        <hr />

                        <div class="flex flex-column g-4">
                            <details class="full round">
                                <summary>{"Board Information"}</summary>

                                <div class="card secondary">
                                    <ul>
                                        <li>{"Created: "}<span class="date-time-to-localize">{&props.board.timestamp}</span></li>
                                        <li>{"Owner: "}<a href={format!("/~{}", &board_m.owner)}>{&board_m.owner}</a></li>

                                        {if board_m.tags.is_some() {
                                            let binding = board_m.tags.unwrap().clone();
                                            let tags = binding.split(" ");

                                            html! { <li>
                                                {"Tags: "}

                                                <span class="g-4 flex-wrap" style="display: inline-flex;">
                                                    {for tags.into_iter().map(|t| {
                                                        html! { <a href={format!("/d/boards/browse?tags={}", t.replace("+", "%2B"))}>{&t}</a> }
                                                    })}
                                                </span>
                                            </li> }
                                        } else {
                                            html! {}
                                        }}
                                    </ul>
                                </div>
                            </details>

                            <details class="full round">
                                <summary>{"Search"}</summary>

                                <div class="card secondary flex flex-column g-4">
                                    <b>{"By Tags"}</b>
                                    <form class="flex g-4 full">
                                        <input
                                            type="text"
                                            name="tags"
                                            id="tags"
                                            placeholder="Tags"
                                            class="round"
                                            value={props.tags.clone()}
                                            maxlength={250}
                                            style="width: calc(100% - 50px);"
                                        />

                                        <button class="round bundles-primary" style="width: 50px;">{"Go"}</button>
                                    </form>
                                </div>
                            </details>
                        </div>

                        <hr />
                    </div>

                    {if (props.auth_state.is_some() && props.auth_state.unwrap() == true) || (can_post_from_anonymous == true) {
                        // ^ signed in OR can_post_from_anonymous is true
                        html! {
                            <div class="full">
                                <div class="card round secondary flex flex-column g-4" id="post">
                                    <div id="error" class="mdnote note-error full" style="display: none;" />

                                    <form id="create-post" class="flex flex-column g-4">
                                        <div class="full flex justify-space-between align-center g-4">
                                            <b>{"Create Post"}</b>

                                            <button class="bundles-primary round">
                                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-plus"><path d="M5 12h14"/><path d="M12 5v14"/></svg>
                                                {"Send"}
                                            </button>
                                        </div>

                                        <textarea
                                            type="text"
                                            name="content"
                                            id="content"
                                            placeholder="Content"
                                            class="full round"
                                            minlength={2}
                                            maxlength={1_000}
                                            required={true}
                                        ></textarea>

                                        <input
                                            type="text"
                                            name="topic"
                                            id="topic"
                                            placeholder="Topic (optional)"
                                            class="round"
                                            maxlength={250}
                                        />
                                    </form>
                                </div>

                                <hr style="var(--u-08) 0 var(--u-04) 0" />
                            </div>
                    }} else {
                        html! {}
                    }}

                    {if props.pinned.len() > 0 {
                        html! {
                            <>
                                {for props.pinned.iter().map(|p| {
                                    html! { <Message post={p.clone()} show_open={true} pinned={true} /> }
                                })}

                                <hr class="full" style="var(--u-08) 0 var(--u-04) 0" />
                            </>
                        }
                    } else {
                        html! {}
                    }}

                    {for props.posts.iter().map(|p| {
                        html! { <Message post={p.clone()} show_open={true} pinned={false} /> }
                    })}

                    <div class="full flex justify-space-between" id="pages">
                        <a class="button round" href={format!("?tags={}&offset={}", props.tags, props.offset - 50)} disabled={props.offset <= 0}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-left"><path d="m12 19-7-7 7-7"/><path d="M19 12H5"/></svg>
                            {"Back"}
                        </a>

                        <a class="button round" href={format!("?tags={}&offset={}", props.tags, props.offset + 50)} disabled={props.posts.len() == 0}>
                            {"Next"}
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-right"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>
                        </a>
                    </div>

                    <script type="module">
                        {"import \"/static/js/BoardView.js\";"}
                    </script>

                    <Footer auth_state={props.auth_state} />
                </main>
            </div>
        </div>
    };
}

fn build_view_renderer_with_props(props: Props) -> ServerRenderer<ViewBoard> {
    ServerRenderer::<ViewBoard>::with_props(|| props)
}

#[get("/b/{name:.*}")]
/// Available at "/b/{name}"
pub async fn view_board_request(
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
    }

    // get board
    let name: String = req.match_info().get("name").unwrap().to_string();

    let board: bundlesdb::DefaultReturn<Option<Board<String>>> =
        data.db.get_board_by_name(name.clone()).await;

    if board.success == false {
        let renderer = ServerRenderer::<crate::pages::errors::_404Page>::new();
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "text/html"))
            .body(utility::format_html(
                renderer.render().await,
                "<title>404: Not Found</title>",
            ));
    }

    // check if board is private
    // if it is, only the owner and users with the "staff" role can view it
    let metadata =
        serde_json::from_str::<bundlesdb::BoardMetadata>(&board.payload.as_ref().unwrap().metadata)
            .unwrap();

    if metadata.is_private == String::from("yes") {
        // anonymous
        if token_user.is_none() {
            return HttpResponse::NotFound()
                .body("You do not have permission to view this board's contents.");
        }

        // not owner
        let user = token_user.unwrap().payload.unwrap();

        if (user.username != metadata.owner) && (user.role != String::from("staff")) {
            return HttpResponse::NotFound()
                .body("You do not have permission to view this board's contents.");
        }
    }

    // ...
    let posts: bundlesdb::DefaultReturn<Option<Vec<Log>>> =
        if info.tags.is_some() && (info.tags.as_ref().unwrap().len() > 0) {
            data.db
                .get_board_posts_by_tag(
                    name.clone(),
                    info.tags.as_ref().unwrap().to_string(),
                    info.offset,
                )
                .await
        } else {
            // all posts
            data.db.get_board_posts(name.clone(), info.offset).await
        };

    let pinned: bundlesdb::DefaultReturn<Option<Vec<Log>>> =
        data.db.get_pinned_board_posts(name.clone()).await;

    // ...
    let renderer = build_view_renderer_with_props(Props {
        board: board.payload.unwrap(),
        posts: posts.payload.unwrap(),
        pinned: pinned.payload.unwrap(),
        offset: if info.offset.is_some() {
            info.offset.unwrap()
        } else {
            0
        },
        tags: if info.tags.is_some() {
            info.tags.as_ref().unwrap().to_string()
        } else {
            String::new()
        },
        auth_state: if req.cookie("__Secure-Token").is_some() {
            Option::Some(true)
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
                <meta property=\"og:title\" content=\"{}\" />
                <meta property=\"og:description\" content=\"{}\" />",
                &name, &name, "View board posts on ::SITE_NAME::"
            ),
        ));
}

#[function_component]
fn ViewBoardPost(props: &ViewPostProps) -> Html {
    let p = &props.post;
    let post = serde_json::from_str::<BoardPostLog>(&p.content).unwrap();
    let board = serde_json::from_str::<BoardMetadata>(&props.board.metadata).unwrap();

    // check if we can manage this post (delete post)
    // must be authenticated AND board owner OR staff OR post author
    let can_manage: bool = props.auth_state.is_some()
        && props.user.is_some()
        && props.user.as_ref().unwrap().username != String::from("Anonymous")
        && ((props.user.as_ref().unwrap().username == board.owner)
            | (props.user.as_ref().unwrap().role == String::from("staff"))
            | (props.user.as_ref().unwrap().username == post.author));

    // check if we can manage this board (pin post)
    // must be authenticated AND board owner OR staff
    let can_manage_board: bool = props.auth_state.is_some()
        && props.user.is_some()
        && props.user.as_ref().unwrap().username != String::from("Anonymous")
        && ((props.user.as_ref().unwrap().username == board.owner)
            | (props.user.as_ref().unwrap().role == String::from("staff")));

    // check if we can edit this post (edit post)
    // must be authenticated AND post author OR staff
    let can_edit: bool = props.auth_state.is_some()
        && props.user.is_some()
        && props.user.as_ref().unwrap().username != String::from("Anonymous")
        && ((props.user.as_ref().unwrap().role == String::from("staff"))
            | (props.user.as_ref().unwrap().username == post.author));

    // ...
    let can_post_from_anonymous =
        board.allow_anonymous.is_none() || board.allow_anonymous.unwrap() != String::from("no");

    // ...
    return html! {
        <div class="flex flex-column" style="height: 100dvh;">
            <div style="display: none;" id="board-name">{&props.board.name}</div>
            <div style="display: none;" id="post-id">{&props.post.id}</div>

            <div class="toolbar flex justify-space-between">
                // left
                <div class="flex">
                    <a class="button" href="/" style="border-left: 0">
                        <b>{"::SITE_NAME::"}</b>
                    </a>

                    <a class="button" href={format!("/b/{}", props.board.name)} style="border-left: 0">
                        {props.board.name.clone()}
                    </a>
                </div>
            </div>

            <div class="toolbar-layout-wrapper">
                <main class="flex flex-column g-4">
                    <div id="error" class="mdnote note-error full" style="display: none;" />
                    <div id="success" class="mdnote note-note full" style="display: none;" />

                    {if (props.edit == false && props.edit_tags == false) | (can_edit == false) {
                        html! { <>
                            {if post.topic.is_some() {
                                html! { <h1 style="margin-top: 0; margin-bottom: 1rem;">{post.topic.unwrap()}</h1> }
                            } else {
                                html! {}
                            }}

                            <Message post={p.clone()} show_open={false} pinned={false} />
                        </> }
                    } else if props.edit_tags == true {
                        // edit tags
                        html! { <div class="card round secondary" id="post">
                            <form id="edit-post-tags" class="flex flex-column g-4" data-endpoint={format!("/api/board/{}/posts/{}/tags", &post.board, &p.id)}>
                                <div class="full flex justify-space-between align-center g-4">
                                    <b>{"Edit Post Tags"}</b>

                                    <button class="bundles-primary round">
                                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-save"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
                                        {"Save"}
                                    </button>
                                </div>

                                <textarea
                                    type="text"
                                    name="tags"
                                    id="tags"
                                    placeholder="Tags"
                                    class="full round"
                                    value={if post.tags.is_some() {
                                        post.tags.unwrap()
                                    } else {
                                        String::new()
                                    }}
                                    minlength={2}
                                    maxlength={1_000}
                                    required={true}
                                ></textarea>
                            </form>
                        </div> }
                    } else {
                        // edit content
                        html! { <div class="card round secondary" id="post">
                            <form id="edit-post" class="flex flex-column g-4" data-endpoint={format!("/api/board/{}/posts/{}/update", &post.board, &p.id)}>
                                <div class="full flex justify-space-between align-center g-4">
                                    <b>{"Edit Post"}</b>

                                    <button class="bundles-primary round">
                                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-save"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
                                        {"Save"}
                                    </button>
                                </div>

                                <textarea
                                    type="text"
                                    name="content"
                                    id="content"
                                    placeholder="Content"
                                    class="full round"
                                    value={post.content}
                                    minlength={2}
                                    maxlength={1_000}
                                    required={true}
                                ></textarea>

                                <input
                                    type="text"
                                    name="topic"
                                    id="topic"
                                    placeholder="Topic (optional)"
                                    class="round"
                                    value={if post.topic.is_some() {
                                        post.topic.unwrap()
                                    } else {
                                        String::new()
                                    }}
                                    maxlength={250}
                                />
                            </form>
                        </div> }
                    }}

                    <div class="flex flex-wrap g-4">
                        {if can_manage {
                            html! {
                                <>
                                    <button class="border round" id="delete-post" data-endpoint={format!("/api/board/{}/posts/{}", &post.board, &p.id)}>{"Delete"}</button>
                                    <a class="button border round" href="?edit_tags=true">{"Edit Tags"}</a>
                                </>
                            }
                        } else {
                            html! {}
                        }}

                        {if can_manage_board {
                            html! {
                                <button class="border round" id="pin-post" data-endpoint={format!("/api/board/{}/posts/{}/pin", &post.board, &p.id)}>{"Pin"}</button>
                            }
                        } else {
                            html! {}
                        }}

                        {if can_edit {
                            html! {
                                <a class="button border round" href="?edit=true">{"Edit"}</a>
                            }
                        } else {
                            html! {}
                        }}
                    </div>

                    {if (props.auth_state.is_some() && props.auth_state.unwrap() == true) || (can_post_from_anonymous == true) {
                        // ^ signed in OR can_post_from_anonymous is true
                        html! {
                            <>
                                <hr style="var(--u-04) 0 var(--u-08) 0" />

                                <div class="full flex flex-column g-4">
                                    <details class="full round" style="display: none;">
                                        <summary>{"About this board"}</summary>

                                        <div class="card secondary full" id="about">
                                            {if board.about.is_some() {
                                                let content = Html::from_html_unchecked(AttrValue::from(
                                                    crate::markdown::render::parse_markdown(&board.about.unwrap())
                                                ));

                                                html! {{content}}
                                            } else {
                                                html! {}
                                            }}
                                        </div>
                                    </details>

                                    <div class="card round secondary" id="post">
                                        <form id="create-post" class="flex flex-column g-4">
                                            <div class="full flex justify-space-between align-center g-4">
                                                <b>{"Reply"}</b>

                                                <button class="bundles-primary round">
                                                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-plus"><path d="M5 12h14"/><path d="M12 5v14"/></svg>
                                                    {"Send"}
                                                </button>
                                            </div>

                                            <textarea
                                                type="text"
                                                name="content"
                                                id="content"
                                                placeholder="Content"
                                                class="full round"
                                                minlength={2}
                                                maxlength={1_000}
                                                required={true}
                                            ></textarea>
                                        </form>
                                    </div>

                                    // <hr style="var(--u-08) 0 var(--u-04) 0" />
                                </div>
                            </>
                    }} else {
                        html! {}
                    }}

                    {for props.replies.iter().map(|p| {
                        html! { <Message post={p.clone()} show_open={true} pinned={false} /> }
                    })}

                    <Footer auth_state={props.auth_state} />
                </main>
            </div>

            <script type="module">
                {"import \"/static/js/ManageBoardPost.js\";"}
            </script>
        </div>
    };
}

fn build_view_post_renderer_with_props(props: ViewPostProps) -> ServerRenderer<ViewBoardPost> {
    ServerRenderer::<ViewBoardPost>::with_props(|| props)
}

#[get("/b/{name:.*}/posts/{id:.*}")]
/// Available at "/b/{name}/posts/{id:.*}"
pub async fn view_board_post_request(
    req: HttpRequest,
    data: web::Data<db::bundlesdb::AppData>,
    info: web::Query<ViewBoardPostQueryProps>,
) -> impl Responder {
    // you're able to do this even if the board is private ON PURPOSE

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

    // get board
    let name: String = req.match_info().get("name").unwrap().to_string();

    let board: bundlesdb::DefaultReturn<Option<Board<String>>> =
        data.db.get_board_by_name(name.clone()).await;

    if board.success == false {
        let renderer = ServerRenderer::<crate::pages::errors::_404Page>::new();
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "text/html"))
            .body(utility::format_html(
                renderer.render().await,
                "<title>404: Not Found</title>",
            ));
    }

    // get post
    let id: String = req.match_info().get("id").unwrap().to_string();
    let post: bundlesdb::DefaultReturn<Option<Log>> = data.db.get_log_by_id(id.clone()).await;

    if post.success == false {
        let renderer = ServerRenderer::<crate::pages::errors::_404Page>::new();
        return HttpResponse::NotFound()
            .append_header(("Content-Type", "text/html"))
            .body(utility::format_html(
                renderer.render().await,
                "<title>404: Not Found</title>",
            ));
    }

    // get replies
    let replies: bundlesdb::DefaultReturn<Option<Vec<Log>>> =
        data.db.get_post_replies(id.clone(), false).await;

    // ...
    let renderer = build_view_post_renderer_with_props(ViewPostProps {
        board: board.payload.unwrap(),
        post: post.payload.clone().unwrap(),
        replies: replies.payload.unwrap(),
        auth_state: if req.cookie("__Secure-Token").is_some() {
            Option::Some(true)
        } else {
            Option::Some(false)
        },
        user: if token_user.is_some() {
            Option::Some(token_user.unwrap().payload.unwrap())
        } else {
            Option::None
        },
        edit: if info.edit.is_some() {
            info.edit.unwrap()
        } else {
            false
        },
        edit_tags: if info.edit_tags.is_some() {
            info.edit_tags.unwrap()
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
                <meta property=\"og:title\" content=\"{}\" />
                <meta property=\"og:description\" content=\"{}\" />",
                &name,
                "View board post",
                format!("View post in board {}", name)
            ),
        ));
}

#[function_component]
fn BoardSettings(props: &SettingsProps) -> Html {
    let metadata = serde_json::from_str::<bundlesdb::BoardMetadata>(&props.board.metadata).unwrap();

    return html! {
        <div>
            <div class="toolbar flex justify-space-between">
                // left
                <div class="flex">
                    <a class="button" href="/" style="border-left: 0">
                        <b>{"::SITE_NAME::"}</b>
                    </a>

                    <a class="button" href={format!("/b/{}", props.board.name)} style="border-left: 0">
                        {props.board.name.clone()}
                    </a>
                </div>

                // right
                <div class="flex">
                    <a class="button" href={format!("/b/{}", props.board.name)}>{"Feed"}</a>
                </div>
            </div>

            <div class="toolbar-layout-wrapper">
                <main class="flex flex-column g-4 small">
                    <h2 class="full text-center">{"Board Settings"}</h2>

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
                                <button class="secondary round red" id="delete-board">{"Delete"}</button>
                                <a href={format!("/b/{}", props.board.name)} class="button round secondary">{"Cancel"}</a>
                            </div>
                        </div>

                        <div id="options-field" class="flex flex-wrap mobile:flex-column g-4 full justify-space-between" />
                    </div>

                    <script type="module">
                        {format!("import {{ paste_settings }} from \"/static/js/SettingsEditor.js\";
                        paste_settings({}, \"{}\", document.getElementById(\"options-field\"), \"board\");", serde_json::to_string(&metadata).unwrap(), &props.board.name)}
                    </script>

                    <Footer auth_state={props.auth_state} />
                </main>
            </div>
        </div>
    };
}

fn build_settings_with_props(props: SettingsProps) -> ServerRenderer<BoardSettings> {
    ServerRenderer::<BoardSettings>::with_props(|| props)
}

#[get("/b/{name:.*}/manage")]
/// Available at "/b/{name}/manage"
pub async fn board_settings_request(
    req: HttpRequest,
    data: web::Data<bundlesdb::AppData>,
) -> impl Responder {
    // get board
    let name: String = req.match_info().get("name").unwrap().to_string();
    let name_c = name.clone();

    let board: bundlesdb::DefaultReturn<Option<Board<String>>> =
        data.db.get_board_by_name(name).await;

    if board.success == false {
        return HttpResponse::NotFound().body(board.message);
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
    } else {
        return HttpResponse::NotAcceptable().body("An account is required to do this");
    }

    // ...
    let metadata =
        serde_json::from_str::<bundlesdb::BoardMetadata>(&board.payload.as_ref().unwrap().metadata)
            .unwrap();

    let user = token_user.unwrap().payload.unwrap();
    let can_view: bool = (user.username == metadata.owner) | (user.role == String::from("staff"));

    if can_view == false {
        return HttpResponse::NotFound()
            .body("You do not have permission to manage this board's contents.");
    }

    // ...
    let renderer = build_settings_with_props(SettingsProps {
        board: board.payload.clone().unwrap(),
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
                <meta property=\"og:title\" content=\"{} (board settings) - ::SITE_NAME::\" />",
                &name_c, &name_c
            ),
        ));
}

#[function_component]
fn SearchByTag(props: &SearchProps) -> Html {
    html! {
        <div class="flex flex-column" style="height: 100dvh;">
            <div class="toolbar flex justify-space-between">
                // left
                <div class="flex">
                    <a class="button" href="/" style="border-left: 0">
                        <b>{"::SITE_NAME::"}</b>
                    </a>

                    <a class="button" href="/d" style="border-left: 0">
                        {"Dashboard"}
                    </a>
                </div>
            </div>

            <div class="toolbar-layout-wrapper">
                <div id="link-header" style="display: flex;" class="flex-column bg-1">
                    <div class="link-header-top"></div>

                    <div class="link-header-middle">
                        <h1 class="no-margin">{"Browse Boards"}</h1>
                    </div>

                    <div class="link-header-bottom">
                        <a href="/d" class="button">{"Home"}</a>
                        <a href="/d/pastes" class="button">{"Pastes"}</a>
                        <a href="/d/atomic" class="button">{"Atomic"}</a>
                        <a href="/d/boards" class="button active">{"Boards"}</a>
                    </div>
                </div>

                <main class="small flex flex-column g-4">
                    <div class="flex justify-space-between align-center mobile:flex-column g-4">
                        <b>{"Search Boards"}</b>

                        <form style="width: 50%;" class="flex g-4 mobile:max">
                            <input
                                type="text"
                                name="tags"
                                id="tags"
                                placeholder="Tags"
                                class="round"
                                value={props.tags.clone()}
                                maxlength={250}
                                style="width: calc(100% - 50px);"
                            />

                            <button class="round bundles-primary" style="width: 50px;">{"Go"}</button>
                        </form>
                    </div>

                    <div class="card round secondary flex g-4 flex-column justify-center" id="boards_list">
                        {for props.boards.iter().map(|b| html! {
                            <a class="button secondary round full justify-start" href={format!("/b/{}", &b.name)} title={b.tags.clone()}>
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-messages-square"><path d="M14 9a2 2 0 0 1-2 2H6l-4 4V4c0-1.1.9-2 2-2h8a2 2 0 0 1 2 2v5Z"/><path d="M18 9h2a2 2 0 0 1 2 2v11l-4-4h-6a2 2 0 0 1-2-2v-1"/></svg>
                                {&b.name}
                            </a>
                        })}
                    </div>

                    <div class="full flex justify-space-between" id="pages">
                        <a class="button round" href={format!("?tags={}&offset={}", props.tags, props.offset - 50)} disabled={props.offset <= 0}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-left"><path d="m12 19-7-7 7-7"/><path d="M19 12H5"/></svg>
                            {"Back"}
                        </a>

                        <a class="button round" href={format!("?tags={}&offset={}", props.tags, props.offset + 50)} disabled={props.boards.len() == 0}>
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

fn build_search_renderer_with_props(props: SearchProps) -> ServerRenderer<SearchByTag> {
    ServerRenderer::<SearchByTag>::with_props(|| props)
}

#[get("/d/boards/browse")]
/// Available at "/d/boards/browse"
pub async fn search_by_tags_request(
    req: HttpRequest,
    data: web::Data<db::bundlesdb::AppData>,
    info: web::Query<SearchQueryProps>,
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
        // you must have an account to use boards
        // we'll likely track bandwidth used by atomic pastes and limit it in the future
        return HttpResponse::NotFound().body(
            "You must have an account to use boards.
You can login at: /d/auth/login
You can create an account at: /d/auth/register",
        );
    }

    // fetch boards
    let boards = if info.tags.is_some() {
        data.db
            .get_boards_by_tags(info.tags.as_ref().unwrap().to_string(), info.offset)
            .await
    } else {
        data.db.get_boards(info.offset).await
    };

    if boards.success == false {
        return HttpResponse::NotFound().body(boards.message);
    }

    // ...
    let renderer = build_search_renderer_with_props(SearchProps {
        boards: boards.payload.unwrap(),
        tags: if info.tags.is_some() {
            info.tags.as_ref().unwrap().to_string()
        } else {
            String::new()
        },
        offset: if info.offset.is_some() {
            info.offset.unwrap()
        } else {
            0
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
            "<title>Browse Boards - ::SITE_NAME::</title>",
        ));
}

#[function_component]
fn Dashboard(props: &DashboardProps) -> Html {
    html! {
        <div class="flex flex-column" style="height: 100dvh;">
            <div class="toolbar flex justify-space-between">
                // left
                <div class="flex">
                    <a class="button" href="/" style="border-left: 0">
                        <b>{"::SITE_NAME::"}</b>
                    </a>

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
                        <a href="/d/atomic" class="button">{"Atomic"}</a>
                        <a href="/d/boards" class="button active">{"Boards"}</a>
                    </div>
                </div>

                <main class="small flex flex-column g-4">
                    <div class="flex justify-space-between align-center">
                        <b>{"My Boards"}</b>

                        <div class="flex g-4 flex-wrap">
                            <a class="button border round" href="/d/boards/browse">
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-search"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
                                {"Browse"}
                            </a>

                            <a class="button bundles-primary round" href="/b/new">
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-plus-square"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M8 12h8"/><path d="M12 8v8"/></svg>
                                {"New"}
                            </a>
                        </div>
                    </div>

                    <div class="card round secondary flex g-4 flex-column justify-center" id="boards_list">
                        {for props.boards.iter().map(|b| html! {
                            <a class="button secondary round full justify-start" href={format!("/b/{}", &b.name)}>
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-messages-square"><path d="M14 9a2 2 0 0 1-2 2H6l-4 4V4c0-1.1.9-2 2-2h8a2 2 0 0 1 2 2v5Z"/><path d="M18 9h2a2 2 0 0 1 2 2v11l-4-4h-6a2 2 0 0 1-2-2v-1"/></svg>
                                {&b.name}
                            </a>
                        })}
                    </div>

                    <Footer auth_state={props.auth_state} />
                </main>
            </div>
        </div>
    }
}

fn build_dashboard_renderer_with_props(props: DashboardProps) -> ServerRenderer<Dashboard> {
    ServerRenderer::<Dashboard>::with_props(|| props)
}

#[get("/d/boards")]
/// Available at "/d/boards"
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
        // you must have an account to use boards
        // we'll likely track bandwidth used by atomic pastes and limit it in the future
        return HttpResponse::NotFound().body(
            "You must have an account to use boards.
You can login at: /d/auth/login
You can create an account at: /d/auth/register",
        );
    }

    // fetch boards
    let boards = data
        .db
        .get_boards_by_owner(token_user.clone().unwrap().payload.unwrap().username)
        .await;

    // ...
    let renderer = build_dashboard_renderer_with_props(DashboardProps {
        boards: boards.payload.unwrap(),
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
            "<title>My Boards - ::SITE_NAME::</title>",
        ));
}
