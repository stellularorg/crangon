use actix_web::HttpResponse;
use actix_web::{get, web, HttpRequest, Responder};

use yew::prelude::*;
use yew::ServerRenderer;

use crate::db::bundlesdb::{self, AppData, Paste};
use crate::utility::format_html;
use crate::{markdown, utility};

use crate::components::navigation::Footer;

#[derive(Default, Properties, PartialEq)]
struct Props {
    pub paste: Paste<String>,
    pub auth_state: Option<bool>,
}

#[function_component]
fn PasteView(props: &Props) -> Html {
    let content = Html::from_html_unchecked(AttrValue::from(markdown::parse_markdown(
        &props.paste.content,
    )));

    let metadata = serde_json::from_str::<bundlesdb::PasteMetadata>(&props.paste.metadata).unwrap();

    return html! {
        <main class="flex flex-column g-4">
            <div id="secret" />

            <div
                id="editor-tab-preview"
                class="card round secondary tab-container secondary round"
                style="height: max-content; max-height: initial; margin-bottom: 0px;"
            >
                {content}
            </div>

            <div class="flex justify-space-between g-4 full">
                <a class="button round" href={format!("/?editing={}", &props.paste.custom_url)}>
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-pencil"><path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/><path d="m15 5 4 4"/></svg>
                    {"Edit"}
                </a>

                <div class="flex flex-column g-2 text-right" style="color: var(--text-color-faded);">
                    <span class="flex justify-center g-4">
                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-cake-slice"><circle cx="9" cy="7" r="2"/><path d="M7.2 7.9 3 11v9c0 .6.4 1 1 1h16c.6 0 1-.4 1-1v-9c0-2-3-6-7-8l-3.6 2.6"/><path d="M16 13H3"/><path d="M16 17H3"/></svg>
                        {"Pub: "}<span class="date-time-to-localize">{&props.paste.pub_date}</span>
                    </span>

                    <span class="flex justify-center g-4">
                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-pencil"><path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/><path d="m15 5 4 4"/></svg>
                        {"Edit: "}<span class="date-time-to-localize">{&props.paste.edit_date}</span>
                    </span>

                    if &metadata.owner.is_empty() == &false {
                        <span>{"Owner: "} <span id="data-time-to-localize">{&metadata.owner}</span></span>
                    }

                    <span>{"Views: "}{&props.paste.views}</span>
                </div>
            </div>

            <Footer auth_state={props.auth_state} />

            <script type="module">
                {"import ClientFixMarkdown from \"/static/js/ClientFixMarkdown.js\"; ClientFixMarkdown();"}
            </script>
        </main>
    };
}

fn build_renderer_with_props(props: Props) -> ServerRenderer<PasteView> {
    return ServerRenderer::<PasteView>::with_props(|| props);
}

#[get("/{url:.*}")]
pub async fn paste_view_request(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    // get paste
    let url: String = req.match_info().get("url").unwrap().to_string();
    let url_c = url.clone();

    let paste: bundlesdb::DefaultReturn<Option<Paste<String>>> = if url == String::from("d") {
        bundlesdb::create_dummy(Option::Some("dummy-paste"))
    } else {
        // fetch paste
        data.db.get_paste_by_url(url).await
    };

    if paste.success == false {
        let renderer = ServerRenderer::<crate::pages::errors::_404Page>::new();
        return HttpResponse::NotFound().body(utility::format_html(
            renderer.render().await,
            "<title>404: Not Found</title>",
        ));
    }

    let unwrap = paste.payload.as_ref().unwrap();
    let metadata = serde_json::from_str::<bundlesdb::PasteMetadata>(&unwrap.metadata).unwrap();

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

        // count view (this will check for an existing view!)
        let payload = &token_user.as_ref().unwrap().payload;
        data.db
            .add_view_to_url(&url_c, &payload.as_ref().unwrap().username)
            .await;
    }

    // ...
    let paste_preview_text = unwrap.content.chars().take(100).collect();
    let embed_color_unwrap = metadata.embed_color.as_ref();
    let favicon_unwrap = metadata.favicon.as_ref();

    // ...
    let renderer = build_renderer_with_props(Props {
        paste: unwrap.clone(),
        auth_state: if req.cookie("__Secure-Token").is_some() {
            Option::Some(req.cookie("__Secure-Token").is_some())
        } else {
            Option::Some(false)
        },
    });

    let render = renderer.render();
    return HttpResponse::Ok()
        .append_header(("Set-Cookie", set_cookie))
        .body(format_html(
            render.await,
            &format!(
                "<title>{}</title>
                <meta property=\"og:url\" content=\"{}\" />
                <meta property=\"og:title\" content=\"{}\" />
                <meta property=\"og:description\" content=\"{}\" />
                <meta name=\"theme-color\" content=\"{}\" />
                <link rel=\"icon\" href=\"{}\" />",
                &url_c,
                &format!(
                    "{}{}",
                    req.headers().get("Host").unwrap().to_str().unwrap(),
                    req.head().uri.to_string()
                ),
                if metadata.title.is_empty() {
                    &url_c
                } else {
                    &metadata.title
                },
                if metadata.description.is_empty() {
                    &paste_preview_text
                } else {
                    &metadata.description
                },
                // optionals
                if metadata.embed_color.is_none() {
                    "#ff9999"
                } else {
                    &embed_color_unwrap.unwrap()
                },
                if metadata.favicon.is_none() {
                    "/static/favicon.svg"
                } else {
                    &favicon_unwrap.unwrap()
                }
            ),
        ));
}
