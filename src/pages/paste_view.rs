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
    pub paste: Paste,
}

#[function_component]
fn PasteView(props: &Props) -> Html {
    let content = Html::from_html_unchecked(AttrValue::from(markdown::parse_markdown(
        &props.paste.content,
    )));

    return html! {
        <main class="flex flex-column g-4" style="height: 100dvh;">
            <div
                id="editor-tab-preview"
                class="card round border secondary tab-container secondary round"
                style="height: max-content; max-height: initial; margin-bottom: 0px;"
            >
                {content}
            </div>

            <Footer />
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
    let paste: bundlesdb::DefaultReturn<Option<Paste>> = if url == String::from("d") {
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
    return HttpResponse::Ok().body(format_html(render.await));
}
