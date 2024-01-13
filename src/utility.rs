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
