use std::time::{SystemTime, UNIX_EPOCH};

pub fn unix_epoch_timestamp() -> u128 {
    let right_now = SystemTime::now();
    let time_since = right_now
        .duration_since(UNIX_EPOCH)
        .expect("Time travel is not allowed");
    
    return time_since.as_millis();
}

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
