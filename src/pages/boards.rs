use actix_web::HttpRequest;
use actix_web::{get, web, HttpResponse, Responder};

use yew::prelude::*;
use yew::ServerRenderer;

use crate::components::navigation::Footer;
use crate::db::bundlesdb::{Board, Log};
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
    pub auth_state: Option<bool>,
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
    return html! {
        <div class="flex flex-column g-4" style="height: 100dvh;">
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

                </div>
            </div>

            <div class="toolbar-layout-wrapper">
                <main class="small flex flex-column g-4 align-center">
                    <script type="module">
                        {"import \"/static/js/NewBoard.js\";"}
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

    let posts: bundlesdb::DefaultReturn<Option<Vec<Log>>> =
        data.db.get_board_posts(name.clone()).await;

    // ...
    let renderer = build_view_renderer_with_props(Props {
        board: board.payload.unwrap(),
        posts: posts.payload.unwrap(),
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
            &format!("<title>{}</title>", &name),
        ));
}
