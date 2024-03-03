use yew::prelude::*;

#[derive(Properties, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FooterProps {
    pub auth_state: Option<bool>,
}

#[function_component]
pub fn Footer(props: &FooterProps) -> Html {
    let info_req = std::env::var("INFO");
    let mut info: String = String::new();

    if info_req.is_err() && info.is_empty() {
        info = "/pub/info".to_string();
    } else {
        info = info_req.unwrap();
    }

    // ...
    return html! {
        <div class="flex justify-center align-center flex-column full">
            <hr class="small" style="width:425px; max-width:100%; margin-top:1rem;" />

            <ul class="__footernav" style="padding: 0; margin: 0;">
                <li><a href="/">{"new"}</a></li>
                <li><a href="/d/settings">{"settings"}</a></li>
                <li><a href={info}>{"info"}</a></li>

                if props.auth_state.is_some() {
                    if props.auth_state.unwrap() == false {
                        <li><a href="/d/auth/register">{"register"}</a></li>
                        <li><a href="/d/auth/login">{"login"}</a></li>
                    } else {
                        <li><a href="/d">{"dashboard"}</a></li>
                        <li><a href="/api/auth/logout">{"logout"}</a></li>
                    }
                }
            </ul>

            <p style="font-size: 12px; margin: 0.4rem 0 0 0;">
                <a href="https://code.stellular.org/SentryTwo/bundlrs">{"::SITE_NAME::"}</a>
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

            // theme
            <div style="position: relative; width: 100%;">
                <div style="position: absolute; bottom: 8px; right: 0;">
                    <a
                        id="theme_button"
                        href="javascript:window.toggle_theme()"
                        title="Toggle Theme"
                        style="color: var(--text-color-faded);"
                    >
                        <div id="theme-icon-sun">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-sun"><circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/></svg>
                        </div>

                        <div id="theme-icon-moon">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-moon"><path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/></svg>
                        </div>
                    </a>
                </div>
            </div>
        </div>
    };
}
