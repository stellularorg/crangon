use actix_web::HttpResponse;
use actix_web::{get, web, HttpRequest, Responder};

use yew::prelude::*;
use yew::ServerRenderer;

use crate::db::bundlesdb::{self, AppData, Paste};
use crate::markdown;
use crate::utility::format_html;

use crate::components::navigation::Footer;

#[derive(Default, Properties, PartialEq)]
struct Props {
    pub paste: Paste<String>,
}

#[function_component]
fn PasteView(props: &Props) -> Html {
    let content = Html::from_html_unchecked(AttrValue::from(markdown::parse_markdown(
        &props.paste.content,
    )));

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
            </div>

            <Footer />

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
        return HttpResponse::NotFound().body(paste.message);
    }

    // ...
    let renderer = build_renderer_with_props(Props {
        paste: paste.payload.unwrap(),
    });

    let render = renderer.render();
    return HttpResponse::Ok().body(format_html(
        render.await,
        &format!("<title>{}</title>
<meta property=\"og:url\" content=\"{}\" />
<meta property=\"og:title\" content=\"{}\" />
<meta property=\"og:description\" content=\"Bundlrs doesn't support description yet! Don't worry, this is coming soon.\" />
", &url_c, &format!("{}{}", req.headers().get("Host").unwrap().to_str().unwrap(), req.head().uri.to_string()), &url_c),
    ));
}
