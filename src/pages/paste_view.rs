use actix_web::HttpRequest;
use actix_web::{get, HttpResponse, Responder};

use yew::prelude::*;
use yew::ServerRenderer;

use crate::db::bundlesdb::Paste;
use crate::utility::format_html;

use crate::components::navigation::Footer;

#[derive(Default, Properties, PartialEq)]
struct Props {
    pub paste: Paste,
}

#[function_component]
fn PasteView(props: &Props) -> Html {
    return html! {
        <main class="flex flex-column g-4" style="height: 100dvh;">
            <div 
                id="editor-tab-preview"
                class="card round border secondary tab-container secondary round" 
                style="height: max-content; max-height: initial; margin-bottom: 0px;"
            >
                {&props.paste.custom_url}
            </div>

            <Footer />
        </main>
    };
}

fn build_renderer_with_props(props: Props) -> ServerRenderer<PasteView> {
    return ServerRenderer::<PasteView>::with_props(|| props);
}

#[get("/{url:.*}")]
pub async fn paste_view_request(req: HttpRequest) -> impl Responder {
    // get paste
    let url: String = req.match_info().get("url").unwrap().to_string();

    // ...
    let renderer = build_renderer_with_props(Props {
        paste: crate::db::bundlesdb::create_dummy(Option::Some(&url)),
    });

    let render = renderer.render();
    return HttpResponse::Ok().body(format_html(render.await));
}
