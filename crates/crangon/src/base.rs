//! Base values for the base template (`templates/base.html`)
use std::env;

#[derive(Debug, Clone)]
pub struct BaseStore {
    /// `SITE_NAME` variable
    pub site_name: String,
    /// `INFO_URL` variable, "what" in the footer
    pub info_url: String,
    /// `BODY_EMBED` variable, HTML that is embedded on every page
    pub body_embed: String,
    /// `GUPPY_ROOT` variable, for guppy auth (disabled if not provided)
    pub guppy_root: String,
    /// `SECRET` variable, "true" makes the footer not link to the source
    pub secret: bool,
}

impl BaseStore {
    pub fn new() -> Self {
        Self {
            site_name: match env::var("SITE_NAME") {
                Ok(s) => s,
                Err(_) => String::from("Crangon"),
            },
            info_url: match env::var("INFO_URL") {
                Ok(s) => s,
                Err(_) => String::new(),
            },
            body_embed: match env::var("BODY_EMBED") {
                Ok(s) => s,
                Err(_) => String::new(),
            },
            guppy_root: match env::var("GUPPY_ROOT") {
                Ok(s) => s,
                Err(_) => String::new(),
            },
            secret: match env::var("SECRET") {
                Ok(s) => s == "true",
                Err(_) => false,
            },
        }
    }
}
