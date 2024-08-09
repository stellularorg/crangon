//! Responds to API requests
use crate::model::{
    PasteCreate, PasteClone, PasteDelete, PasteEdit, PasteError, PasteEditMetadata, Paste,
    PublicPaste,
};
use crate::database::Database;
use dorsal::DefaultReturn;

use axum::response::IntoResponse;
use axum::{
    extract::{Path, State, Query},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::cookie::CookieJar;

pub fn routes(database: Database) -> Router {
    Router::new()
        .route("/new", post(create_paste))
        .route("/clone", post(clone_paste))
        // pastes
        .route("/:url", get(get_paste_by_url))
        .route("/:url/delete", post(delete_paste_by_url))
        .route("/:url/edit", post(edit_paste_by_url))
        .route("/:url/metadata", post(edit_paste_metadata_by_url))
        // auth
        .route("/auth/callback", get(callback_request))
        .route("/auth/logout", get(logout_request))
        // ...
        .with_state(database)
}

/// Create a new paste (`/api/new`)
async fn create_paste(
    State(database): State<Database>,
    Json(paste_to_create): Json<PasteCreate>,
) -> Result<Json<DefaultReturn<(String, Paste)>>, PasteError> {
    let res = database.create_paste(paste_to_create).await;

    match res {
        Ok(paste) => Ok(Json(DefaultReturn {
            success: true,
            message: String::from("Paste created"),
            payload: paste,
        })),
        Err(e) => Err(e),
    }
}

/// Clone an existing paste (`/api/clone`)
async fn clone_paste(
    State(database): State<Database>,
    Json(paste_to_create): Json<PasteClone>,
) -> Result<Json<DefaultReturn<(String, Paste)>>, PasteError> {
    let res = database.clone_paste(paste_to_create).await;

    match res {
        Ok(paste) => Ok(Json(DefaultReturn {
            success: true,
            message: String::from("Paste cloned"),
            payload: paste,
        })),
        Err(e) => Err(e),
    }
}

/// Delete an existing paste (`/api/:url/delete`)
async fn delete_paste_by_url(
    State(database): State<Database>,
    Path(url): Path<String>,
    Json(paste_to_delete): Json<PasteDelete>,
) -> Result<Json<DefaultReturn<()>>, PasteError> {
    match database
        .delete_paste_by_url(url, paste_to_delete.password)
        .await
    {
        Ok(_) => Ok(Json(DefaultReturn {
            success: true,
            message: String::from("Paste deleted"),
            payload: (),
        })),
        Err(e) => Err(e),
    }
}

/// Edit an existing paste (`/api/:url/edit`)
async fn edit_paste_by_url(
    jar: CookieJar,
    State(database): State<Database>,
    Path(url): Path<String>,
    Json(paste_to_edit): Json<PasteEdit>,
) -> Result<Json<DefaultReturn<()>>, PasteError> {
    match database
        .edit_paste_by_url(
            url,
            paste_to_edit.password,
            paste_to_edit.new_content,
            paste_to_edit.new_url,
            paste_to_edit.new_password,
            // get editing_as
            if let Some(cookie) = jar.get("__Secure-Token") {
                let value = cookie.value_trimmed();

                if database.options.guppy == true {
                    match database.auth.get_user_by_unhashed(value.to_string()).await {
                        Ok(ua) => Option::Some(ua),
                        Err(_) => return Err(PasteError::Other),
                    }
                } else {
                    Option::None
                }
            } else {
                Option::None
            },
        )
        .await
    {
        Ok(_) => Ok(Json(DefaultReturn {
            success: true,
            message: String::from("Paste updated"),
            payload: (),
        })),
        Err(e) => Err(e),
    }
}

/// Edit an existing paste's metadata (`/api/:url/metadata`)
async fn edit_paste_metadata_by_url(
    jar: CookieJar,
    State(database): State<Database>,
    Path(url): Path<String>,
    Json(mut paste_to_edit): Json<PasteEditMetadata>,
) -> Result<Json<DefaultReturn<()>>, PasteError> {
    // if we've been given an authentication cookie (and it's allowed),
    // we'll check the user and then set metadata.owner
    if let Some(cookie) = jar.get("__Secure-Token") {
        let value = cookie.value_trimmed();

        if (database.options.guppy == true) && (database.options.paste_ownership == true) {
            match database.auth.get_user_by_unhashed(value.to_string()).await {
                Ok(ua) => paste_to_edit.metadata.owner = ua.user.username,
                Err(_) => paste_to_edit.metadata.owner = "".to_string(),
            }
        }
    } else {
        // clear owner field if paste is edited by an anonymous user
        paste_to_edit.metadata.owner = "".to_string();
    }

    // ...
    match database
        .edit_paste_metadata_by_url(
            url,
            paste_to_edit.password,
            paste_to_edit.metadata,
            // get editing_as
            if let Some(cookie) = jar.get("__Secure-Token") {
                let value = cookie.value_trimmed();

                if database.options.guppy == true {
                    match database.auth.get_user_by_unhashed(value.to_string()).await {
                        Ok(ua) => Option::Some(ua),
                        Err(_) => return Err(PasteError::Other),
                    }
                } else {
                    Option::None
                }
            } else {
                Option::None
            },
        )
        .await
    {
        Ok(_) => Ok(Json(DefaultReturn {
            success: true,
            message: String::from("Paste updated"),
            payload: (),
        })),
        Err(e) => Err(e),
    }
}

/// Get an existing paste by url (`/api/:url`)
pub async fn get_paste_by_url(
    State(database): State<Database>,
    Path(url): Path<String>,
) -> Result<Json<DefaultReturn<PublicPaste>>, PasteError> {
    match database.get_paste_by_url(url).await {
        Ok(p) => {
            if !p.metadata.view_password.is_empty() {
                return Err(PasteError::Other);
            }

            Ok(Json(DefaultReturn {
                success: true,
                message: String::from("Paste exists"),
                payload: p.into(),
            }))
        }
        Err(e) => Err(e),
    }
}

// general
pub async fn not_found() -> impl IntoResponse {
    Json(DefaultReturn::<u16> {
        success: false,
        message: String::from("Path does not exist"),
        payload: 404,
    })
}

// auth
#[derive(serde::Deserialize)]
pub struct CallbackQueryProps {
    pub uid: String, // this uid will need to be sent to the client as a token
}

pub async fn callback_request(
    State(database): State<Database>,
    Query(params): Query<CallbackQueryProps>,
) -> impl IntoResponse {
    if database.options.guppy == false {
        return (
            [
                ("Content-Type".to_string(), "text/plain".to_string()),
                ("Set-Cookit".to_string(), String::new()),
            ],
            "Server does not support guppy.",
        );
    }

    // return
    (
        [
            ("Content-Type".to_string(), "text/html".to_string()),
            (
                "Set-Cookie".to_string(),
                format!(
                    "__Secure-Token={}; SameSite=Lax; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age={}",
                    params.uid,
                    60 * 60 * 24 * 365
                ),
            ),
        ],
        "<head>
            <meta http-equiv=\"Refresh\" content=\"0; URL=/\" />
        </head>"
    )
}

pub async fn logout_request(State(database): State<Database>, jar: CookieJar) -> impl IntoResponse {
    if database.options.guppy == false {
        return (
            [
                ("Content-Type".to_string(), "text/plain".to_string()),
                ("Set-Cookit".to_string(), String::new()),
            ],
            "Server does not support guppy.",
        );
    }

    // check for cookie
    if let Some(_) = jar.get("__Secure-Token") {
        return (
            [
                ("Content-Type".to_string(), "text/plain".to_string()),
                (
                    "Set-Cookie".to_string(),
                   "__Secure-Token=refresh; SameSite=Strict; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age=0".to_string(),
                ),
            ],
            "You have been signed out. You can now close this tab.",
        );
    }

    // return
    (
        [
            ("Content-Type".to_string(), "text/plain".to_string()),
            ("Set-Cookit".to_string(), String::new()),
        ],
        "Failed to sign out of account.",
    )
}
