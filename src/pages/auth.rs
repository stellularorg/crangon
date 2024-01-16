use actix_web::{get, HttpResponse, Responder};

use yew::prelude::*;
use yew::ServerRenderer;

use crate::components::navigation::Footer;
use crate::utility::format_html;

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

                <Footer />
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

                <script type="module">
                    {"import AuthPages from \"/static/js/AuthPages.js\";"}
                </script>

                <Footer />
            </main>
        </div>
    };
}

#[get("/d/auth/register")]
pub async fn register_request() -> impl Responder {
    let renderer = ServerRenderer::<Register>::new();
    return HttpResponse::Ok().body(format_html(renderer.render().await, ""));
}

#[get("/d/auth/login")]
pub async fn login_request() -> impl Responder {
    let renderer = ServerRenderer::<Login>::new();
    return HttpResponse::Ok().body(format_html(renderer.render().await, ""));
}
