use std::time::{SystemTime, UNIX_EPOCH};

use hex_fmt::HexFmt;
use sha2::{Digest, Sha256};
use uuid::Uuid;

// ids
#[allow(dead_code)]
pub fn hash(input: String) -> String {
    let mut hasher = <Sha256 as Digest>::new();
    hasher.update(input.into_bytes());

    let res = hasher.finalize();
    return HexFmt(res).to_string();
}

#[allow(dead_code)]
pub fn random_id() -> String {
    let uuid = Uuid::new_v4();
    return hash(uuid.to_string());
}

pub fn unix_epoch_timestamp() -> u128 {
    let right_now = SystemTime::now();
    let time_since = right_now
        .duration_since(UNIX_EPOCH)
        .expect("Time travel is not allowed");

    return time_since.as_millis();
}

// html
pub fn format_html(input: String) -> String {
    return format!(
        "<!DOCTYPE html>
<html lang=\"en\">
    <head>
        <meta charset=\"UTF-8\" />
        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />
        <title>Document</title>

        <link rel=\"stylesheet\" href=\"/static/style.css\" />
    </head>
    <body>{input}</body>
</html>"
    )
    .to_string();
}
