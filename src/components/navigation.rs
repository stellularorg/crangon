use yew::prelude::*;

#[function_component]
pub fn Footer() -> Html {
    let info_req = std::env::var("INFO");
    let mut info: String = String::new();

    if info_req.is_err() && info.is_empty() {
        info = "/pub/info".to_string();
    } else {
        info = info_req.unwrap();
    }

    // ...
    return html! {
        <div class="flex justify-center align-center flex-column">
            <hr class="small" style="width:425px; max-width:100%; margin-top:1rem;" />

            <ul class="__footernav" style="padding: 0; margin: 0;">
                <li><a href="/">{"new"}</a></li>
                <li><a href="/s">{"settings"}</a></li>
                <li><a href="/search">{"search"}</a></li>
                <li><a href={info}>{"info"}</a></li>
            </ul>

            <p style="font-size: 12px; margin: 0.4rem 0 0 0;">
                <a href="https://codeberg.org/SentryTwo/bundlrs">{"bundlrs"}</a>
                {" - Markdown Delivery Service"}
            </p>

            <style>
                {
                    ".__footernav {
                        display: flex;
                        gap: 0.25rem;
                    }
                    
                    .__footernav li {
                        list-style-type: \"·\";
                        padding: 0 0.25rem;
                    }

                    .__footernav li:first-child {
                    margin-left: -0.25rem;
                    }
                    
                    .__footernav li:first-child {
                        list-style-type: none;
                    }
                    
                    .__footer_cardbtn {
                        width: calc(33% - 0.25rem);
                        height: 10rem !important;
                        display: flex;
                        flex-direction: column;
                        justify-content: center;
                        align-items: center;
                        gap: 0.5rem;
                        border-radius: 0.4rem;
                    }"
                }
            </style>

            // <ThemeButton>
        </div>
    };
}
