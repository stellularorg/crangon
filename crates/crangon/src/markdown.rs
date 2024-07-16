use sauropod::markdown::parse_markdown as shared_parse_markdown;
use regex::RegexBuilder;

pub fn parse_markdown(input: String) -> String {
    shared_parse_markdown(
        input,
        vec![&mut |mut out: String| {
            // handle span block
            out = regex_replace_exp(
                &out,
                RegexBuilder::new(r"\[s\](.*?)\[/s\]")
                    .multi_line(true)
                    .dot_matches_new_line(true),
                "<span>$1</span>",
            );

            // handle vm modification block
            // these contain a json list that will update the attributes of the previous element
            out = regex_replace_exp(
                &out,
                RegexBuilder::new(r"@&lt;\((.*?)\)&gt;")
                    .multi_line(true)
                    .dot_matches_new_line(true),
                "<script type=\"env/group-mod\">$1</script>",
            );

            out = regex_replace_exp(
                &out,
                RegexBuilder::new(r"&lt;\((.*?)\)&gt;")
                    .multi_line(true)
                    .dot_matches_new_line(true),
                "<script type=\"env/mod\">$1</script>",
            );

            // canvas
            let canvas_regex = RegexBuilder::new(r"(canvas\+)\s*(.*?)(:{2})\s*(.*?)\s*(:{2})")
                .multi_line(true)
                .dot_matches_new_line(true)
                .build()
                .unwrap();

            for capture in canvas_regex.captures_iter(&out.clone()) {
                let modifiers: Vec<&str> = capture.get(2).unwrap().as_str().split(" ").collect(); // css style attributes, split by " "
                let content = capture.get(4).unwrap().as_str(); // columns split by "|"

                // build modifiers
                let gap = modifiers.get(0).unwrap_or(&"0.2rem");
                let justify = modifiers.get(1).unwrap_or(&"center");
                let align = modifiers.get(2).unwrap_or(&"center");
                let direction = modifiers.get(3).unwrap_or(&"row");

                // build content
                let mut built = String::new();

                for element in content.split("|") {
                    built += &format!("<div role=\"canvas_element\">{element}</div>");
                }

                // finished
                out = out.replace(
                    capture.get(0).unwrap().as_str(),
                    &format!(
                        "<div role=\"canvas\" style=\"display: flex; gap: {gap}; justify-content: {justify}; align-items: {align}; flex-direction: {direction}\">{built}</div>"
                    ),
                );
            }

            // subtext
            out = regex_replace_exp(
                &out,
                RegexBuilder::new(r"^-#\s*(.*?)$").multi_line(true),
                "<p style=\"opacity: 75%\" role=\"subtext\">$1</p>",
            );

            // return
            out
        }],
    )
}

#[allow(dead_code)]
fn regex_replace_exp(input: &str, pattern: &mut RegexBuilder, replace_with: &str) -> String {
    pattern
        .build()
        .unwrap()
        .replace_all(input, replace_with)
        .to_string()
}
