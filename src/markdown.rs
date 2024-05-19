use comrak::{markdown_to_html, Options};
use regex::RegexBuilder;

pub fn parse_markdown(mut original_in: String) -> String {
    let mut options = Options::default();

    options.extension.table = true;
    options.extension.superscript = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.header_ids = Option::Some(String::new());
    // options.render.unsafe_ = true;
    options.render.escape = true;
    options.parse.smart = false;

    // escape < and >
    // original_in = regex_replace(&original_in, "<", "&lt;");
    // original_in = regex_replace(&original_in, ">", "&gt;");

    // underline
    original_in = regex_replace(
        &original_in,
        "(\\_{2})(.*?)(\\_{2})",
        "<span style=\"text-decoration: underline;\" role=\"underline\">$2</span>",
    );

    // image with sizing
    let image_sizing_regex = RegexBuilder::new("(!)\\[(.*?)\\]\\((.*?)\\)\\:\\{(.*?)x(.*?)\\}")
        .multi_line(true)
        .build()
        .unwrap();

    for capture in image_sizing_regex.captures_iter(&original_in.clone()) {
        let title = capture.get(2).unwrap().as_str();
        let src = capture.get(3).unwrap().as_str();

        let width = capture.get(4).unwrap().as_str();
        let height = capture.get(5).unwrap().as_str();

        let result = &format!("<img alt=\"{title}\" title=\"{title}\" src=\"{src}\" style=\"width: {width}px; height: {height}px;\" />");
        original_in = original_in.replace(capture.get(0).unwrap().as_str(), result);
    }

    // admonitions
    original_in = regex_replace(
        // title and content
        &original_in,
        "^(\\!{3})\\s(?<TYPE>.*?)\\s(?<TITLE>.+)\\n(?<CONTENT>.+)$",
        "<div class=\"mdnote note-$2\">
            <b class=\"mdnote-title\">$3</b>
            <p>$4</p>
        </div>\n",
    );

    original_in = regex_replace(
        // title only
        &original_in,
        "^(\\!{3})\\s(?<TYPE>.*?)\\s(?<TITLE>.*?)$",
        "<div class=\"mdnote note-$2\"><b class=\"mdnote-title\">$3</b></div>\n",
    );

    // ...
    let mut out: String = markdown_to_html(&original_in, &options);
    out = regex_replace(&out, "(&!)(.*?);", "&$2;");
    out = out.replace("&quot;", "\"");

    // css ">"
    out = out.replace("&gt; \\.", "> .");
    out = out.replace("&gt; #", "> #");
    out = out.replace("(&gt;", "(>");
    out = out.replace(" &gt; ", " > ");

    // only a little bit of regex-ing remains now

    // allowed elements
    let allowed_elements: Vec<&str> = Vec::from([
        "hue", "sat", "lit", "theme", "comment", "p", "span", "style", "img", "div", "a", "b", "i",
        "strong", "em", "r", "rf",
    ]);

    for element in allowed_elements {
        out = regex_replace(
            &out,
            &format!("&lt;{}&gt;", element),
            &format!("<{}>", element),
        );

        out = regex_replace(
            &out,
            &format!("&lt;{}(.*?)&gt;", element),
            &format!("<{}$1>", element),
        );

        out = regex_replace(
            &out,
            &format!("&lt;/{}&gt;", element),
            &format!("</{}>", element),
        );
    }

    // spoiler
    out = regex_replace(
        &out,
        "(\\|\\|)\\s*(?<CONTENT>.*?)\\s*(\\|\\|)",
        "<span role=\"spoiler\">$2</span>",
    );

    out = regex_replace(
        &out,
        "(\\!\\&gt;)\\s*(?<CONTENT>.*?)($|\\s\\s)",
        "<span role=\"spoiler\">$2</span>",
    );

    // highlight
    out = regex_replace(
        &out,
        "(\\={2})(.*?)(\\={2})",
        "<span class=\"highlight\">$2</span>",
    );

    // unescape arrow alignment
    out = regex_replace(&out, "-&gt;&gt;", "->>");
    out = regex_replace(&out, "&lt;&lt;-", "<<-");

    out = regex_replace(&out, "-&gt;", "->");
    out = regex_replace(&out, "&lt;-", "<-");

    // arrow alignment (flex)
    let arrow_alignment_flex_regex = RegexBuilder::new("(\\->{2})(.*?)(\\->{2}|<{2}\\-)")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    for capture in arrow_alignment_flex_regex.captures_iter(&out.clone()) {
        let _match = capture.get(0).unwrap().as_str();
        let content = capture.get(2).unwrap().as_str();

        let align = if _match.ends_with(">") {
            "right"
        } else {
            "center"
        };

        out = out.replacen(
            _match,
            &format!("<rf class=\"justify-{align}\">{content}</rf>\n"),
            1,
        );
    }

    // arrow alignment
    let arrow_alignment_regex = RegexBuilder::new("(\\->{1})(.*?)(\\->{1}|<{1}\\-)")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    for capture in arrow_alignment_regex.captures_iter(&out.clone()) {
        let _match = capture.get(0).unwrap().as_str();
        let content = capture.get(2).unwrap().as_str();

        let align = if _match.ends_with(">") {
            "right"
        } else {
            "center"
        };

        out = out.replacen(
            _match,
            &format!("<r class=\"text-{align}\">{content}</r>\n"),
            1,
        );
    }

    // some bbcode stuff
    out = regex_replace(&out, r"\[b\](.*?)\[/b\]", "<strong>$1</strong>"); // bold
    out = regex_replace(&out, r"\[i\](.*?)\[/i\]", "<em>$1</em>"); // italic
    out = regex_replace(&out, r"\[bi\](.*?)\[/bi\]", "<strong><em>$1</em></strong>"); // bold + italic

    out = regex_replace(
        // underline
        &out,
        r"\[u\](.*?)\[/u\]",
        "<span style=\"text-decoration: underline;\" role=\"underline\">$1</span>",
    );

    out = regex_replace_dmnl(&out, r"\[c\](.*?)\[/c\]", "<r class=\"text-center\">$1</r>"); // center
    out = regex_replace_dmnl(&out, r"\[r\](.*?)\[/r\]", "<r class=\"text-right\">$1</r>"); // right

    out = regex_replace_dmnl(
        // text color
        &out,
        r"\[t (.*?)\](.*?)\[/t\]",
        "<span style=\"color: $1;\" role=\"custom-color\">$2</span>",
    );

    out = regex_replace_dmnl(
        // message
        &out,
        r"\[m (.*?)\](.*?)\[/m\]",
        "<span title=\"$1\" role=\"custom-message\">$2</span>",
    );

    out = regex_replace(
        // highlight
        &out,
        r"\[h\](.*?)\[/h\]",
        "<span class=\"highlight\">$1</span>",
    );

    out = regex_replace(
        // highlight
        &out,
        r"\[h\](.*?)\[/h\]",
        "<span class=\"highlight\">$1</span>",
    );

    for i in 1..7 {
        // headings
        out = regex_replace(
            &out,
            &format!(r"\[h{i}\](.*?)\[/h{i}\]"),
            &format!("<h{i} id=\"$1\">$1</h{i}>"),
        );
    }

    out = regex_replace(&out, r"\[/\]", "<br />"); // line break

    out = regex_replace(
        // code
        &out,
        r"\[cl\](.*?)\[/cl\]",
        "<code>$1</code>",
    );

    out = regex_replace_dmnl(
        // fenced code
        &out,
        r"\[src (.*?)\]\n(.*?)\n\[/src\]",
        "<pre class=\"lang-$1\"><code>$2</code></pre>",
    );

    // bath time
    out = regex_replace(&out, "^(on)(.*)\\=(.*)\"$", "");
    out = regex_replace(&out, "(href)\\=\"(javascript\\:)(.*)\"", "");

    out = regex_replace(&out, "(<script.*>)(.*?)(<\\/script>)", "");
    out = regex_replace(&out, "(<script.*>)", "");
    out = regex_replace(&out, "(<link.*>)", "");
    out = regex_replace(&out, "(<meta.*>)", "");

    out = regex_replace(&out, "(%3C)script(%3E)", ""); // iframe <script> in "data:" src
    out = regex_replace(&out, "(%3C)link", "");

    // return
    out
}

fn regex_replace(input: &str, pattern: &str, replace_with: &str) -> String {
    RegexBuilder::new(pattern)
        .multi_line(true)
        .build()
        .unwrap()
        .replace_all(input, replace_with)
        .to_string()
}

fn regex_replace_dmnl(input: &str, pattern: &str, replace_with: &str) -> String {
    RegexBuilder::new(pattern)
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap()
        .replace_all(input, replace_with)
        .to_string()
}

#[allow(dead_code)]
fn regex_replace_exp(input: &str, pattern: &mut RegexBuilder, replace_with: &str) -> String {
    pattern
        .build()
        .unwrap()
        .replace_all(input, replace_with)
        .to_string()
}
