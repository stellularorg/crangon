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
