use actix_web::HttpRequest;
use actix_web::{get, web, HttpResponse, Responder};

use yew::prelude::*;
use yew::ServerRenderer;

use crate::components::navigation::Footer;
use crate::db;
use crate::utility::format_html;

#[derive(Default, Properties, PartialEq, serde::Deserialize)]
struct Props {
    pub editing: Option<String>,
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

                <form class="flex flex-wrap mobile:justify-center justify-space-between g-4 align-center" action="/api/new" id="save-changes" data-edit={if props.editing.is_some() { "true" } else { "false" }}>
                    <div class="mobile:justify-center flex g-4 justify-start">
                        <button class="round">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-plus"><path d="M5 12h14"/><path d="M12 5v14"/></svg>
                            {"Publish"}
                        </button>
                    </div>

                    <div class="mobile:justify-center flex-wrap flex g-4 justify-start">
                        <input
                            class="secondary round"
                            type="text"
                            placeholder="Custom URL"
                            minlength="2"
                            maxlength="500"
                            name="custom_url"
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
                </form>

                <script type="module">
                    {"import CreateEditor from \"/static/js/MarkdownEditor.js\"; CreateEditor(\"editor-tab-text\", ``);"}
                </script>

                <Footer />
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

    if token_cookie.is_some() {
        let res = data
            .db
            .get_user_by_hashed(token_cookie.unwrap().value().to_string())
            .await;

        if res.success == false {
            set_cookie = "__Secure-Token=refresh; SameSite=Strict; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age=0";
        }
    }

    // ...
    let str = &info.editing;
    let renderer = build_renderer_with_props(Props {
        editing: str.to_owned(),
    });

    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .body(format_html(renderer.render().await));
}
