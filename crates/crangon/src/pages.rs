use askama_axum::Template;
use axum::{
    extract::{Path, State, Query},
    response::{Html, Json, IntoResponse},
    routing::{get, post, get_service},
    Router,
};
use axum_extra::extract::cookie::CookieJar;

use tower_http::services::ServeDir;
use pastemd::{database::Database, model::Paste};
use crate::markdown::parse_markdown;
use serde::{Serialize, Deserialize};

pub fn routes(database: Database) -> Router {
    Router::new()
        .route("/:url/edit/config", get(config_editor_request))
        .route("/:url/edit", get(editor_request))
        .route("/:url", get(view_paste_request))
        .route("/api/render", post(render_markdown))
        // serve static dir
        .nest_service("/static", get_service(ServeDir::new("./static")))
        // ...
        .with_state(database)
}

#[derive(Template)]
#[template(path = "homepage.html")]
struct HomepageTemplate {}

pub async fn homepage() -> impl IntoResponse {
    Html(HomepageTemplate {}.render().unwrap())
}

#[derive(Template)]
#[template(path = "paste_view.html")]
struct PasteViewTemplate {
    paste: Paste,
    rendered: String,
    title: String,
    views: i32,
    head_stuff: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PasteViewQuery {
    #[serde(default)]
    view_password: String,
}

#[derive(Template)]
#[template(path = "paste_password.html")]
struct PastePasswordTemplate {
    paste: Paste,
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorViewTemplate {
    error: String,
}

pub async fn view_paste_request(
    jar: CookieJar,
    Path(url): Path<String>,
    State(database): State<Database>,
    Query(query_params): Query<PasteViewQuery>,
) -> impl IntoResponse {
    match database.get_paste_by_url(url).await {
        Ok(p) => {
            // get user from token
            let auth_user = match jar.get("__Secure-Token") {
                Some(c) => match database
                    .auth
                    .get_user_by_unhashed(c.value_trimmed().to_string())
                    .await
                {
                    Ok(ua) => Some(ua),
                    Err(_) => None,
                },
                None => None,
            };

            // check for view password
            if database.options.view_password == true {
                match query_params.view_password.is_empty() {
                    false => {
                        if !p.metadata.view_password.is_empty()
                            && (query_params.view_password != p.metadata.view_password)
                        {
                            return Html(PastePasswordTemplate { paste: p }.render().unwrap());
                        }
                    }
                    true => {
                        if !p.metadata.view_password.is_empty() {
                            return Html(PastePasswordTemplate { paste: p }.render().unwrap());
                        }
                    }
                }
            }

            // push view
            // we could not support paste views by just.. not doing this
            if let Err(e) = database.incr_views_by_url(p.url.clone(), auth_user).await {
                return Html(
                    ErrorViewTemplate {
                        error: e.to_string(),
                    }
                    .render()
                    .unwrap(),
                );
            }

            // ...
            let rendered = parse_markdown(p.content.clone());
            Html(
                PasteViewTemplate {
                    paste: p.clone(),
                    rendered,
                    title: match p.metadata.title.is_empty() {
                        true => p.url.clone(),
                        false => p.metadata.title,
                    },
                    views: database.get_views_by_url(p.url).await,
                    head_stuff: format!(
                        "<meta property=\"og:description\" content=\"{}\" />
                        <meta name=\"theme-color\" content=\"{}\" />
                        <link rel=\"icon\" href=\"{}\" />",
                        if p.metadata.description.is_empty() {
                            // paste preview text
                            p.content
                                .chars()
                                .take(100)
                                .collect::<String>()
                                .replace("\"", "'")
                        } else {
                            p.metadata.description
                        },
                        if p.metadata.theme_color.is_empty() {
                            "#6ee7b7"
                        } else {
                            &p.metadata.theme_color
                        },
                        if p.metadata.favicon.is_empty() {
                            "/static/favicon.svg"
                        } else {
                            &p.metadata.favicon
                        }
                    ),
                }
                .render()
                .unwrap(),
            )
        }
        Err(e) => Html(
            ErrorViewTemplate {
                error: e.to_string(),
            }
            .render()
            .unwrap(),
        ),
    }
}

#[derive(Template)]
#[template(path = "paste_editor.html")]
struct EditorTemplate {
    paste: Paste,
    passwordless: bool,
}

pub async fn editor_request(
    jar: CookieJar,
    Path(url): Path<String>,
    State(database): State<Database>,
    Query(query_params): Query<PasteViewQuery>,
) -> impl IntoResponse {
    // get user from token
    let auth_user = match jar.get("__Secure-Token") {
        Some(c) => match database
            .auth
            .get_user_by_unhashed(c.value_trimmed().to_string())
            .await
        {
            Ok(ua) => ua.user.username,
            Err(_) => String::new(),
        },
        None => String::new(),
    };

    // ...
    match database.get_paste_by_url(url).await {
        Ok(p) => {
            // check for view password
            if database.options.view_password == true {
                match query_params.view_password.is_empty() {
                    false => {
                        if !p.metadata.view_password.is_empty()
                            && (query_params.view_password != p.metadata.view_password)
                        {
                            return Html(PastePasswordTemplate { paste: p }.render().unwrap());
                        }
                    }
                    true => {
                        if !p.metadata.view_password.is_empty() {
                            return Html(PastePasswordTemplate { paste: p }.render().unwrap());
                        }
                    }
                }
            }

            // ...
            let passwordless = !p.metadata.owner.is_empty() && auth_user == p.metadata.owner;
            Html(
                EditorTemplate {
                    paste: p,
                    passwordless,
                }
                .render()
                .unwrap(),
            )
        }
        Err(e) => Html(
            ErrorViewTemplate {
                error: e.to_string(),
            }
            .render()
            .unwrap(),
        ),
    }
}

#[derive(Template)]
#[template(path = "paste_metadata.html")]
struct ConfigEditorTemplate {
    paste: Paste,
    paste_metadata: String,
    auth_user: String,
    passwordless: bool,
}

pub async fn config_editor_request(
    jar: CookieJar,
    Path(url): Path<String>,
    State(database): State<Database>,
    Query(query_params): Query<PasteViewQuery>,
) -> impl IntoResponse {
    // get user from token
    let auth_user = match jar.get("__Secure-Token") {
        Some(c) => match database
            .auth
            .get_user_by_unhashed(c.value_trimmed().to_string())
            .await
        {
            Ok(ua) => ua.user.username,
            Err(_) => String::new(),
        },
        None => String::new(),
    };

    // ...
    match database.get_paste_by_url(url).await {
        Ok(p) => {
            // check for view password
            if database.options.view_password == true {
                match query_params.view_password.is_empty() {
                    false => {
                        if !p.metadata.view_password.is_empty()
                            && (query_params.view_password != p.metadata.view_password)
                        {
                            return Html(PastePasswordTemplate { paste: p }.render().unwrap());
                        }
                    }
                    true => {
                        if !p.metadata.view_password.is_empty() {
                            return Html(PastePasswordTemplate { paste: p }.render().unwrap());
                        }
                    }
                }
            }

            // ...
            let passwordless = !p.metadata.owner.is_empty() && auth_user == p.metadata.owner;
            Html(
                ConfigEditorTemplate {
                    paste: p.clone(),
                    paste_metadata: match serde_json::to_string(&p.metadata) {
                        Ok(m) => m,
                        Err(_) => {
                            return Html(
                                ErrorViewTemplate {
                                    error: pastemd::model::PasteError::Other.to_string(),
                                }
                                .render()
                                .unwrap(),
                            )
                        }
                    },
                    auth_user,
                    passwordless,
                }
                .render()
                .unwrap(),
            )
        }
        Err(e) => Html(
            ErrorViewTemplate {
                error: e.to_string(),
            }
            .render()
            .unwrap(),
        ),
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct RenderMarkdown {
    pub content: String,
}

/// Render markdown body
async fn render_markdown(Json(req): Json<RenderMarkdown>) -> Result<String, ()> {
    Ok(parse_markdown(req.content.clone()))
}
