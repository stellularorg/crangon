use shared::markdown::parse_markdown as shared_parse_markdown;

pub fn parse_markdown(original_in: String) -> String {
    let out = shared_parse_markdown(original_in, Vec::new());

    // return
    out
}
