use regex::RegexBuilder;

#[allow(dead_code)]
struct Heading<'l> {
    pub text: &'l str,
    pub level: usize,
    pub id: String,
}

pub fn parse_markdown(input: &String) -> String {
    let mut out: String = input.to_owned();

    // escape < and >
    out = regex_replace(&out, "<", "&lt;");
    out = regex_replace(&out, ">", "&gt;");

    // unescape arrow alignment
    out = regex_replace(&out, "-&gt;&gt;", "->>");
    out = regex_replace(&out, "&lt;&lt;-", "<<-");

    out = regex_replace(&out, "-&gt;", "->");
    out = regex_replace(&out, "&lt;-", "<-");

    // allowed elements
    let allowed_elements: Vec<&str> = Vec::from([
        "hue", "sat", "lit", "theme", "comment", "p", "span", "style",
    ]);

    for element in allowed_elements {
        out = regex_replace(
            &out,
            &format!("&lt;{}&gt;", element),
            &format!("<{}>", element),
        );

        out = regex_replace(
            &out,
            &format!("&lt;/{}&gt;", element),
            &format!("<{}>", element),
        );
    }

    // HTML escapes
    out = regex_replace(&out, "(&!)(.*?);", "&$2;");

    // backslash line continuation
    // out = regex_replace(&out, "\\\n", "");

    // fenced code blocks
    let mut fenced_code_block_count: i32 = 0;
    let fenced_code_block_regex = RegexBuilder::new("^(`{3})(.*?)\\n(.*?)(`{3})$")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    for capture in fenced_code_block_regex.captures_iter(&out.clone()) {
        let lang = capture.get(2).unwrap().as_str();
        let content = capture.get(3).unwrap().as_str();

        fenced_code_block_count += 1;

        // build line numbers
        let mut line_numbers: String = String::new();
        let mut _current_ln: i32 = 0;

        for line in content.split("\n") {
            if line.is_empty() {
                continue;
            };

            _current_ln += 1;

            line_numbers = format!(
                "{}<a class=\"line-number\" href=\"#B{}L{}\" id=\"B{}L{}\">{}</a>\n",
                line_numbers,
                fenced_code_block_count,
                _current_ln,
                fenced_code_block_count,
                _current_ln,
                _current_ln
            );
        }

        // replace
        out = regex_replace_one(&out, capture.get(1).unwrap().as_str(), &format!("<pre class=\"flex\" style=\"position: relative;\">
            <div class=\"line-numbers code\">{line_numbers}</div>
            <code class=\"language-${lang}\" id=\"B{fenced_code_block_count}C\" style=\"display: block;\">{content}</code>
            <button 
                onclick=\"window.navigator.clipboard.writeText(document.getElementById('B${fenced_code_block_count}C').innerText);\"
                class=\"secondary copy-button\"
                title=\"Copy Code\"
            >
                <svg
                    xmlns=\"http://www.w3.org/2000/svg\"
                    viewBox=\"0 0 16 16\"
                    width=\"16\"
                    height=\"16\"
                >
                    <path d=\"M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25Z\"></path>
                    <path d=\"M5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z\"></path>
                </svg>
            </button>
        </pre>\n"));
    }

    // inline code block
    out = regex_replace(&out, "(`{1})(.*?)(`{1})", "<code>$2</code>");

    // headings
    let table_of_contents: &mut Vec<Heading> = &mut Vec::new();
    let heading_regex = RegexBuilder::new("^(\\#+)\\s(.*?)$")
        .multi_line(true)
        .build()
        .unwrap();

    for capture in heading_regex.captures_iter(&out.clone()) {
        let heading_type = capture.get(1).unwrap().as_str().len();
        let content = capture.get(2).unwrap().as_str();

        // get suffix
        // (get all headings with the same text, suffix is the number of those)
        // (helps prevent duplicate ids)
        let same_headings = table_of_contents.iter().filter(|h| h.text == content);
        let count = same_headings.count() as i32;

        let suffix = if &count == &0 {
            "".to_string()
        } else {
            format!("-{}", count)
        };

        // add to TOC
        let heading_id = regex_replace(
            &format!("{content}{suffix}").to_lowercase(),
            "[^A-Za-z0-9-]",
            "",
        );

        table_of_contents.push(Heading {
            text: content,
            level: heading_type,
            id: heading_id.clone(),
        });

        // return
        out = regex_replace_one(
            &out,
            capture.get(0).unwrap().as_str(),
            format!("<h{heading_type} id=\"{heading_id}\">{content}</h{heading_type}>").as_str(),
        )
    }

    // remove frontmatter
    regex_replace_exp(
        &out,
        RegexBuilder::new("^(\\-{3})F\\n(?<CONTENT>.*?)\\n(\\-{3})F$")
            .multi_line(true)
            .dot_matches_new_line(true),
        "",
    );

    // horizontal rule
    out = out.replace("(*{3,})", "<hr />");
    regex_replace_exp(
        &out,
        RegexBuilder::new("^\\-{3}\\s*$").multi_line(true),
        "\n<hr />\n",
    );

    regex_replace_exp(
        &out,
        RegexBuilder::new("^\\_{3}\\s*$").multi_line(true),
        "\n<hr />\n",
    );

    // special custom element syntax (rs)
    let custom_element_regex =
        RegexBuilder::new("(e\")\\[\\s(?<NAME>.*?)\\s(?<ATRS>.*?)(\\s\\])\"")
            .multi_line(true)
            .build()
            .unwrap();

    for capture in custom_element_regex.captures_iter(&out.clone()) {
        let name = capture.name("NAME").unwrap().as_str();

        let atrs = capture.name("ATRS").unwrap().as_str();
        let mut atrs_split: Vec<String> = atrs.split("+").map(|s| s.to_string()).collect();

        // make sure everything exists (before we try to call .unwrap on them!)
        if atrs_split.get(0).is_none() {
            atrs_split.insert(0, String::from(""))
        }

        if atrs_split.get(1).is_none() {
            atrs_split.insert(1, String::from(""))
        }

        if atrs_split.get(2).is_none() {
            atrs_split.insert(2, String::from(""))
        }

        if atrs_split.get(3).is_none() {
            atrs_split.insert(3, String::from(""))
        }

        // possibilities
        let possible_error_block =
            &"\n!!! error parsing error: invalid element class in element block".to_string();

        let possible_theme_block = &format!("<theme>{}</theme>", atrs_split.get(0).unwrap());
        let possible_hsl_block = &format!(
            "<{}>{}</{}>",
            atrs_split.get(0).unwrap(),
            atrs_split.get(1).unwrap(),
            atrs_split.get(0).unwrap()
        );

        let possible_html_block = &format!("<{}>", atrs_split.get(0).unwrap());
        let possible_chtml_block = &format!("</{}>", atrs_split.get(0).unwrap());

        let possible_class_block = &format!("<span class=\"{}\">", atrs.replace("+", " "));
        let possible_id_block = &format!("<span id=\"{}\">", atrs_split.get(0).unwrap());
        let possible_close_block = &format!("</span>");

        // build result
        let result = match name {
            // theming
            "theme" => &possible_theme_block,
            "hsl" => &possible_hsl_block,
            // html
            "html" => possible_html_block,
            "chtml" => possible_chtml_block,
            "class" => possible_class_block,
            "id" => possible_id_block,
            "close" => possible_close_block,
            // (error message by default)
            &_ => possible_error_block,
        };

        // replace
        out = out.replace(capture.get(0).unwrap().as_str(), &result);
    }

    // text color thing
    out = regex_replace_exp(
        &out,
        RegexBuilder::new("\\%(?<COLOR>.*?)\\%\\s*(?<CONTENT>.*?)\\s*(\\%\\%)").multi_line(true),
        "<span style=\"color: $1;\" role=\"custom-color\">$2</span>",
    );

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

    // admonitions
    out = regex_replace(
        // title and content
        &out,
        "^(\\!{3})\\s(?<TYPE>.*?)\\s(?<TITLE>.+)\\n(?<CONTENT>.+)$",
        "<div class=\"mdnote note-$2\">
            <b class=\"mdnote-title\">$3</b>
            <p>$4</p>
        </div>",
    );

    out = regex_replace(
        // title only
        &out,
        "^(\\!{3})\\s(?<TYPE>.*?)\\s(?<TITLE>.*?)$",
        "<div class=\"mdnote note-$2\"><b class=\"mdnote-title\">$3</b></div>",
    );

    // highlight
    out = regex_replace(
        &out,
        "(\\={2})(.*?)(\\={2})",
        "<span class=\"highlight\">$2</span>",
    );

    // we have to do this ourselves because the next step would make it not work!
    out = regex_replace(
        &out,
        "(\\*{3})(.*?)(\\*{3})",
        "<strong><em>$2</em></strong>",
    );

    // manual bold/italics
    out = regex_replace(&out, "(\\*{2})(.*?)(\\*{2})", "<strong>$2</strong>");
    out = regex_replace(&out, "(\\*{1})(.*?)(\\*{1})", "<em>$2</em>");

    // strikethrough
    out = regex_replace(&out, "(\\~{2})(.*?)(\\~{2})", "<del>$2</del>");

    // underline
    out = regex_replace(
        &out,
        "(\\_{2})(.*?)(\\_{2})",
        "<span style=\"text-decoration: underline;\" role=\"underline\">$2</span>",
    );

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

        out = out.replace(
            _match,
            &format!("<rf style=\"justify-content: {align}\">{content}</rf>"),
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

        out = out.replace(
            _match,
            &format!("<r style=\"text-align: {align}\">{content}</r>"),
        );
    }

    // image with sizing
    out = regex_replace(
        &out,
        "(!)\\[(.*?)\\]\\((.*?)\\)\\:\\{(.*?)x(.*?)\\}",
        "<img alt=\"$2\" title=\"$2\" src=\"$3\" style=\"width: $4px; height: $5px\" />",
    );

    // normal image
    out = regex_replace(
        &out,
        "(!)\\[(.*?)\\]\\((.*?)\\)",
        "<img alt=\"$2\" title=\"$2\" src=\"$3\" />",
    );

    // anchor (auto)
    out = regex_replace(
        &out,
        "[^\"](https\\:\\/\\/)(.*?)\\s",
        "<a href=\"https://$2\">https://$2</a>",
    );

    // anchor (attributes)
    out = regex_replace(
        &out,
        "\\[(?<TEXT>.*?)\\]\\((?<URL>.*?)\\)\\:\\{(?<ATTRS>.+)\\}",
        "<a href=\"$1\" $3>$1</a>",
    );

    // anchor
    out = regex_replace(
        &out,
        "\\[(?<TEXT>.*?)\\]\\((?<URL>.*?)\\)",
        "<a href=\"$1\">$1</a>",
    );

    // bath time
    out = regex_replace(&out, "^(on)(.*)\\=(.*)\"$", "");
    out = regex_replace(&out, "(href)\\=\"(javascript\\:)(.*)\"", "");

    out = regex_replace(&out, "(<script.*>)(.*?)(<\\/script>)", "");
    out = regex_replace(&out, "(<script.*>)", "");
    out = regex_replace(&out, "(<link.*>)", "");
    out = regex_replace(&out, "(<meta.*>)", "");

    // auto paragraph
    out = regex_replace_exp(
        &out,
        RegexBuilder::new("^(.*?)\\n{2,}")
            .multi_line(true)
            .dot_matches_new_line(true),
        "<p>\n$1\n</p>",
    );

    out = regex_replace_exp(
        &out,
        RegexBuilder::new("(\\w|\\s|>)\\s*\\n")
            .multi_line(true)
            .dot_matches_new_line(true),
        "$1<br />",
    );

    // return
    return out.to_string();
}

#[allow(dead_code)]
fn regex_replace(input: &str, pattern: &str, replace_with: &str) -> String {
    return RegexBuilder::new(pattern)
        .multi_line(true)
        .build()
        .unwrap()
        .replace_all(input, replace_with)
        .to_string();
}

#[allow(dead_code)]
fn regex_replace_exp(input: &str, pattern: &mut RegexBuilder, replace_with: &str) -> String {
    return pattern
        .build()
        .unwrap()
        .replace_all(input, replace_with)
        .to_string();
}

#[allow(dead_code)]
fn regex_replace_one(input: &str, pattern: &str, replace_with: &str) -> String {
    return RegexBuilder::new(pattern)
        .build()
        .unwrap()
        .replace(input, replace_with)
        .to_string();
}
