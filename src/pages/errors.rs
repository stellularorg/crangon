use crate::components::navigation::Footer;
use yew::prelude::*;

#[function_component]
pub fn _404Page() -> Html {
    return html! {
        <main class="flex flex-column g-4 align-center">
            <h4>{"Error"}</h4>
            <h5 style="font-weight: normal; margin-top: 0;">{"404 Not Found"}</h5>

            <Footer auth_state={Option::None} />
        </main>
    };
}
