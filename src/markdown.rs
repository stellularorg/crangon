use regex::RegexBuilder;

pub fn parse_markdown(input: String) -> String {
    let mut out: String = input;

    // escape < and >
    let binding = RegexBuilder::new("<")
        .build()
        .unwrap()
        .replace_all(&out, "&lt;");
    out = binding.to_string();

    let binding = RegexBuilder::new(">")
        .build()
        .unwrap()
        .replace_all(&out, "&gt;");
    out = binding.to_string();

    // unescape arrow alignment
    let binding = RegexBuilder::new("-&gt;&gt;")
        .build()
        .unwrap()
        .replace_all(&out, "->>");
    out = binding.to_string();

    let binding = RegexBuilder::new("&lt;&lt-")
        .build()
        .unwrap()
        .replace_all(&out, "<<-");
    out = binding.to_string();

    let binding = RegexBuilder::new("-&gt;")
        .build()
        .unwrap()
        .replace_all(&out, "->");
    out = binding.to_string();

    let binding = RegexBuilder::new("&lt;-")
        .build()
        .unwrap()
        .replace_all(&out, "<-");
    out = binding.to_string();

    // allowed elements
    let allowed_elements: Vec<&str> = Vec::from([
        "hue", "sat", "lit", "theme", "comment", "p", "span", "style",
    ]);

    for element in allowed_elements {
        let binding = RegexBuilder::new(&format!("&lt;{}&gt;", element))
            .build()
            .unwrap()
            .replace_all(&out, &format!("<{}>", element));

        out = binding.to_string();

        let binding = RegexBuilder::new(&format!("&lt;/{}&gt;", element))
            .build()
            .unwrap()
            .replace_all(&out, &format!("</{}>", element));

        out = binding.to_string();
    }

    // HTML escapes
    let binding = RegexBuilder::new("(&!)(.*?);")
        .build()
        .unwrap()
        .replace_all(&out, "&$2;");
    out = binding.to_string();

    // return
    return out.to_string();
}
