use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use serde::{Deserialize, Serialize};
use dorsal::DefaultReturn;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Paste {
    pub id: String,
    pub url: String,
    pub content: String,
    pub password: String,
    pub date_published: u128,
    pub date_edited: u128,
    pub metadata: PasteMetadata,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// All of these fields are
pub struct PasteMetadata {
    /// Paste page title
    #[serde(default)]
    pub title: String,
    /// Paste page description
    #[serde(default)]
    pub description: String,
    /// Paste theme color
    #[serde(default, alias = "embed_color")]
    pub theme_color: String,
    /// Paste favicon link
    #[serde(default, deserialize_with = "dumb_property")]
    pub favicon: String,
    /// Paste view password (can be disabled)
    #[serde(default, deserialize_with = "dumb_property")]
    pub view_password: String,
    /// Paste owner username
    #[serde(default)]
    pub owner: String,
    /// Paste template settings
    ///
    /// * blank/no value = not a template and not using a template
    /// * `@` = this paste is a template
    /// * anything else = the URL of the template paste this paste is derived from
    #[serde(default)]
    pub template: String,
}

impl From<Paste> for PasteMetadata {
    /// Convert the given paste into [`PasteMetadata`] which uses the paste as a template
    fn from(value: Paste) -> Self {
        Self {
            template: value.url,
            ..Default::default()
        }
    }
}

fn dumb_property<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_else(|| String::default()))
}

impl Default for PasteMetadata {
    fn default() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            theme_color: String::new(),
            favicon: String::new(),
            view_password: String::new(),
            owner: String::new(),
            template: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublicPaste {
    pub url: String,
    pub content: String,
    pub date_published: u128,
    pub date_edited: u128,
    pub metadata: PasteMetadata,
}

impl From<Paste> for PublicPaste {
    fn from(value: Paste) -> Self {
        Self {
            url: value.url,
            content: value.content,
            date_published: value.date_published,
            date_edited: value.date_edited,
            metadata: value.metadata,
        }
    }
}

// props

#[derive(Serialize, Deserialize, Debug)]
pub struct PasteCreate {
    /// The paste url
    #[serde(default)]
    pub url: String,
    /// The content of the paste
    pub content: String,
    /// The paste edit password
    #[serde(default)]
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PasteClone {
    /// The url of the paste we're using as a template
    pub source: String,
    /// The paste url
    #[serde(default)]
    pub url: String,
    /// The paste edit password
    #[serde(default)]
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PasteDelete {
    /// The password of the paste
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PasteEdit {
    /// The password of the paste
    pub password: String,
    /// The updated content of the paste
    pub new_content: String,
    /// The updated password of the paste
    #[serde(default)]
    pub new_password: String,
    /// The updated url of the paste
    #[serde(default)]
    pub new_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PasteEditMetadata {
    /// The password of the paste
    pub password: String,
    /// The updated metadata of the paste
    pub metadata: PasteMetadata,
}

/// General API errors
pub enum PasteError {
    PasswordIncorrect,
    AlreadyExists,
    ValueError,
    NotFound,
    Other,
}

impl PasteError {
    pub fn to_string(&self) -> String {
        use crate::model::PasteError::*;
        match self {
            PasswordIncorrect => String::from("The given password is invalid."),
            AlreadyExists => String::from("A paste with this URL already exists."),
            ValueError => String::from("One of the field values given is invalid."),
            NotFound => String::from("No paste with this URL has been found."),
            _ => String::from("An unspecified error has occured"),
        }
    }
}

impl IntoResponse for PasteError {
    fn into_response(self) -> Response {
        use crate::model::PasteError::*;
        match self {
            PasswordIncorrect => (
                StatusCode::UNAUTHORIZED,
                Json(DefaultReturn::<u16> {
                    success: false,
                    message: self.to_string(),
                    payload: 401,
                }),
            )
                .into_response(),
            AlreadyExists => (
                StatusCode::BAD_REQUEST,
                Json(DefaultReturn::<u16> {
                    success: false,
                    message: self.to_string(),
                    payload: 400,
                }),
            )
                .into_response(),
            NotFound => (
                StatusCode::NOT_FOUND,
                Json(DefaultReturn::<u16> {
                    success: false,
                    message: self.to_string(),
                    payload: 404,
                }),
            )
                .into_response(),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DefaultReturn::<u16> {
                    success: false,
                    message: self.to_string(),
                    payload: 500,
                }),
            )
                .into_response(),
        }
    }
}
