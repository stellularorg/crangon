use actix_web::{get, HttpResponse, Responder};

use yew::prelude::*;
use yew::ServerRenderer;

use crate::components::navigation::Footer;
use crate::utility::format_html;

#[function_component]
fn Home() -> Html {
    return html! {
        <div class="flex flex-column g-4" style="height: 100dvh;">
            <main style="height: calc(100% - 1rem);">
                <div class="tabbar justify-space-between full">
                    // left
                    <div class="flex">
                        <button id="editor-open-tab-text">{"Text"}</button>
                        <button id="editor-open-tab-preview" class="secondary">
                            {"Preview"}
                        </button>
                    </div>
                </div>

                <div id="-editor" class="tab-container card secondary round" style="border-top-left-radius: 0px; border-top-right-radius: 0px;">
                    <div id="editor-tab-text" class="editor-tab -editor active" style="100%;">
                        {"text input here"}
                    </div>
                </div>

                <Footer />
            </main>
        </div>
    };
}

#[get("/")]
pub async fn home_request() -> impl Responder {
    let renderer = ServerRenderer::<Home>::new();
    return HttpResponse::Ok().body(format_html(renderer.render().await));
}
