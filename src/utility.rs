use std::time::{SystemTime, UNIX_EPOCH};

use hex_fmt::HexFmt;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::config;

// ids
#[allow(dead_code)]
pub fn uuid() -> String {
    let uuid = Uuid::new_v4();
    return uuid.to_string();
}

#[allow(dead_code)]
pub fn hash(input: String) -> String {
    let mut hasher = <Sha256 as Digest>::new();
    hasher.update(input.into_bytes());

    let res = hasher.finalize();
    return HexFmt(res).to_string();
}

#[allow(dead_code)]
pub fn random_id() -> String {
    return hash(uuid());
}

pub fn unix_epoch_timestamp() -> u128 {
    let right_now = SystemTime::now();
    let time_since = right_now
        .duration_since(UNIX_EPOCH)
        .expect("Time travel is not allowed");

    return time_since.as_millis();
}

// html
pub fn format_html(input: String, head: &str) -> String {
    let embed_in_body_var = std::env::var("BODY_EMBED");
    let embed_in_body = if embed_in_body_var.is_ok() {
        embed_in_body_var.unwrap()
    } else {
        String::new()
    };

    // ...
    let site_name = config::get_var("SITE_NAME");
    let guppy = config::get_var("GUPPY_ROOT");
    let puffer = config::get_var("PUFFER_ROOT");

    // ...
    return format!(
        "<!DOCTYPE html>
<html lang=\"en\">
    <head>
        <meta charset=\"UTF-8\" />
        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />
        <meta http-equiv=\"content-security-policy\" content=\"default-src 'self' blob:; img-src * data:; media-src *; font-src *; style-src 'unsafe-inline' 'self' blob: *; script-src 'self' 'unsafe-inline' blob: *; object-src 'self' blob: *; upgrade-insecure-requests; connect-src *; frame-src 'self' blob: data: *\" />
        
        {}
        <meta name=\"theme-color\" content=\"#ff9999\" />
        <meta property=\"og:type\" content=\"website\" />
        <meta property=\"og:site_name\" content=\"::SITE_NAME::\" />
        {head}

        <link rel=\"stylesheet\" href=\"/static/style.css\" />
        <script type=\"module\" src=\"/static/js/Footer.js\"></script>
    </head>
    <body>
        {input}
        {embed_in_body}
    </body>
</html>",
        // only provide favicon is page doesn't set it manually
        if !head.contains("rel=\"icon\"") {
            "<link rel=\"icon\" href=\"/static/favicon.svg\" />"
        } else {
            ""
        }
    )
    .to_string().replace("::SITE_NAME::", if site_name.is_some() {
        site_name.unwrap()
    } else {
        "Bundlrs".to_string()
    }.as_str()).replace("::GUPPY_ROOT::", if guppy.is_some() {
        guppy.unwrap()
    } else {
        "".to_string()
    }.as_str()).replace("::PUFFER_ROOT::", if puffer.is_some() {
        puffer.unwrap()
    } else {
        "".to_string()
    }.as_str());
}
