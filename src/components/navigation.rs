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

            <div class="__footernav" style="padding: 0; margin: 0;">
                <div class="item">
                    <a href={info} class="flex align-center g-4">
                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-info"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>
                        {"info"}
                    </a>
                </div>

                {if props.auth_state.is_none() | (props.auth_state.is_some() && props.auth_state.unwrap() == false) {
                    html! { <>
                        <div class="item">
                            <a href="/" class="flex align-center g-4">
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-circle-plus"><circle cx="12" cy="12" r="10"/><path d="M8 12h8"/><path d="M12 8v8"/></svg>
                                {"new"}
                            </a>
                        </div>

                        <div class="item">
                            <a href="::GUPPY_ROOT::/d/auth/register" class="flex align-center g-4" data-wants-redirect="true">
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-at-sign"><circle cx="12" cy="12" r="4"/><path d="M16 8v5a3 3 0 0 0 6 0v-1a10 10 0 1 0-4 8"/></svg>
                                {"register"}
                            </a>
                        </div>

                        <div class="item">
                            <a href="::GUPPY_ROOT::/d/auth/login" class="flex align-center g-4" data-wants-redirect="true">
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-log-in"><path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"/><polyline points="10 17 15 12 10 7"/><line x1="15" x2="3" y1="12" y2="12"/></svg>
                                {"login"}
                            </a>
                        </div>
                    </> }
                } else {
                    html! { <>
                        <div class="item">
                            <a href="/d" class="flex align-center g-4">
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-layout-dashboard"><rect width="7" height="9" x="3" y="3" rx="1"/><rect width="7" height="5" x="14" y="3" rx="1"/><rect width="7" height="9" x="14" y="12" rx="1"/><rect width="7" height="5" x="3" y="16" rx="1"/></svg>
                                {"dashboard"}
                            </a>
                        </div>

                        <div class="item">
                            <a href="/api/auth/logout" class="flex align-center g-4">
                                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-log-out"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" x2="9" y1="12" y2="12"/></svg>
                                {"logout"}
                            </a>
                        </div>
                    </> }
                }}
            </div>

            <p style="font-size: 12px; margin: 0.4rem 0 0 0;">
                <a href="https://code.stellular.org/stellular/bundlrs">{"::SITE_NAME::"}</a>
                {" - Markdown Social"}
            </p>

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

#[function_component]
pub fn GlobalMenu(props: &FooterProps) -> Html {
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or(String::from("source"));

    let info_req = std::env::var("INFO");
    let mut info: String = String::new();

    if info_req.is_err() && info.is_empty() {
        info = "/pub/info".to_string();
    } else {
        info = info_req.unwrap();
    }

    html! {
        <div class="link-list" style="display: none; box-shadow: 2px 2px 4px hsla(0, 0%, 0%, 25%);" id="upper:globalmenu">
            <div class="option small full flex align-center g-4 justify-space-between">
                <a href="/" style="color: inherit;"><b>{"::SITE_NAME::"}</b></a>
                <a href="https://code.stellular.org/stellular/bundlrs" style="color: var(--text-color);">{version}</a>
            </div>

            <div class="option full flex flex-column g-4">
                <h6 class="no-margin">{"LINKS"}</h6>

                {if props.auth_state.is_none() | (props.auth_state.is_some() && props.auth_state.unwrap() == false) {
                    html! { <>
                        <a href="/" class="button full round border justify-start">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-circle-plus"><circle cx="12" cy="12" r="10"/><path d="M8 12h8"/><path d="M12 8v8"/></svg>
                            {"new"}
                        </a>

                        <a href="::GUPPY_ROOT::/d/auth/register" class="button green full round border justify-start" data-wants-redirect="true">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-at-sign"><circle cx="12" cy="12" r="4"/><path d="M16 8v5a3 3 0 0 0 6 0v-1a10 10 0 1 0-4 8"/></svg>
                            {"register"}
                        </a>

                        <a href="::GUPPY_ROOT::/d/auth/login" class="button green full round border justify-start" data-wants-redirect="true">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-log-in"><path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"/><polyline points="10 17 15 12 10 7"/><line x1="15" x2="3" y1="12" y2="12"/></svg>
                            {"login"}
                        </a>
                    </> }
                } else {
                    html! { <>
                        <a href="/d" class="button full round border justify-start">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-layout-dashboard"><rect width="7" height="9" x="3" y="3" rx="1"/><rect width="7" height="5" x="14" y="3" rx="1"/><rect width="7" height="9" x="14" y="12" rx="1"/><rect width="7" height="5" x="3" y="16" rx="1"/></svg>
                            {"dashboard"}
                        </a>

                        <a href="/api/auth/logout" class="button red full round border justify-start">
                            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-log-out"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" x2="9" y1="12" y2="12"/></svg>
                            {"logout"}
                        </a>
                    </> }
                }}
            </div>

            <div class="option full flex flex-column g-4">
                <h6 class="no-margin">{"HELP"}</h6>

                <a href={info} class="button full round border justify-start">
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-info"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>
                    {"info"}
                </a>

                <a href={"/api/docs/bundlrs/index.html"} class="button full round border justify-start">
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-braces"><path d="M8 3H7a2 2 0 0 0-2 2v5a2 2 0 0 1-2 2 2 2 0 0 1 2 2v5c0 1.1.9 2 2 2h1"/><path d="M16 21h1a2 2 0 0 0 2-2v-5c0-1.1.9-2 2-2a2 2 0 0 1-2-2V5a2 2 0 0 0-2-2h-1"/></svg>
                    {"api"}
                </a>
            </div>
        </div>
    }
}
