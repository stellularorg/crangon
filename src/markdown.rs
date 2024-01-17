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
            &format!("</{}>", element),
        );
    }

    // HTML escapes
    out = regex_replace(&out, "(&!)(.*?);", "&$2;");

    // backslash line continuation
    out = out.replace("\\\n", "");

    // fenced code blocks
    let mut fenced_code_block_count: i32 = 0;
    let fenced_code_block_regex = RegexBuilder::new("^(`{3})(.*?)\\n(.*?)(`{3})$")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    for capture in fenced_code_block_regex.captures_iter(&out.clone()) {
        let lang = capture.get(2).unwrap().as_str();
        let mut content = capture.get(3).unwrap().as_str().to_string();

        fenced_code_block_count += 1;

        // run replacements
        content = content.replace("*", "&!temp-ast;");
        content = content.replace("`", "&!temp-back;");
        content = content.replace("\\n", "&nbsp;1;\\n");
        content = content.replace("#", "&#35;");
        content = content.replace("(", "&lpar;");

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
        out = out.replace( capture.get(0).unwrap().as_str(), &format!("<pre class=\"flex\" style=\"position: relative;\">
            <div class=\"line-numbers code\">{line_numbers}</div>
            <code class=\"language-${lang}\" id=\"B{fenced_code_block_count}C\" style=\"display: block;\">{content}</code>
            <button 
                onclick=\"window.navigator.clipboard.writeText(document.getElementById('B{fenced_code_block_count}C').innerText);\"
                class=\"secondary copy-button\"
                title=\"Copy Code\"
            >
                <svg 
                    xmlns=\"http://www.w3.org/2000/svg\" 
                    width=\"18\" 
                    height=\"18\" 
                    viewBox=\"0 0 24 24\" 
                    fill=\"none\" 
                    stroke=\"currentColor\" 
                    stroke-width=\"2\" 
                    stroke-linecap=\"round\" 
                    stroke-linejoin=\"round\" 
                    class=\"lucide lucide-clipboard-copy\"
                >
                    <rect 
                        width=\"8\" 
                        height=\"4\" 
                        x=\"8\" 
                        y=\"2\" 
                        rx=\"1\" 
                        ry=\"1\"
                    />
                    
                    <path d=\"M8 4H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2\" />
                    <path d=\"M16 4h2a2 2 0 0 1 2 2v4\" />
                    <path d=\"M21 14H11\" />
                    <path d=\"m15 10-4 4 4 4\" />
                </svg>
            </button>
        </pre>"));
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
            format!("<h{heading_type} id=\"{heading_id}\">{content}</h{heading_type}>\n").as_str(),
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
    out = regex_replace(&out, "^\\*{3,}", "\n<hr />\n");
    out = regex_replace(&out, "^\\-{3,}", "\n<hr />\n");
    out = regex_replace(&out, "^\\_{3,}", "\n<hr />\n");

    // list
    // lists **need** to know what's on the next and previous lines to know if they need to open or close,
    // this means we need to iterate through lines for lists and make sure they start with "-" (trimmed)
    // lists are the only thing matched by this parser that don't use regex
    let cl = out.clone();
    let out_lines: Vec<String> = cl.split("\n").map(|s| s.to_string()).collect();

    for (i, line) in out_lines.iter().enumerate() {
        if !line.trim().starts_with("-") | line.trim().starts_with("->") {
            continue;
        }

        let mut split: Vec<&str> = line.split("-").collect::<Vec<&str>>();
        let level = split.get(0).unwrap().len() / 4;

        // ...
        let previous: Option<&String> = if i > 0 {
            let l = out_lines.get(i - 1);
            let l_spl = if l.is_some() {
                l.unwrap().split("-").collect::<Vec<&str>>()
            } else {
                Vec::new()
            };

            if l.is_some() && l_spl.get(1).is_some() {
                l
            } else {
                Option::None
            }
        } else {
            Option::None
        };

        let next: Option<&String> = if i < out_lines.len() {
            let l = out_lines.get(i + 1);
            let l_spl = if l.is_some() {
                l.unwrap().split("-").collect::<Vec<&str>>()
            } else {
                Vec::new()
            };

            if l.is_some() && l_spl.get(1).is_some() {
                l
            } else {
                Option::None
            }
        } else {
            Option::None
        };

        // ...
        split.remove(0); // remove everything before the first "-"
        let result: String = format!(
            "{}<li style=\"margin-left: {}px;\">{}</li>{}",
            // if previous doesn't exist, this is the start of the list
            if previous.is_none() { "<ul>" } else { "" },
            // ...
            level * 40,
            split.join("-"), // join split back
            // if next doesn't exist, this is the end of the list
            if next.is_none() { "</ul>\n" } else { "" }
        );

        // ...
        out = out.replace(line, &result);
    }

    // special custom element syntax (rs)
    let custom_element_regex = RegexBuilder::new("(e\\#)(?<NAME>.*?)\\s(?<ATRS>.*?)\\#")
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
        RegexBuilder::new("\\%(?<COLOR>.*?)\\%\\s*(?<CONTENT>.*?)\\s*(\\%\\%)")
            .multi_line(true)
            .dot_matches_new_line(true),
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
        </div>\n",
    );

    out = regex_replace(
        // title only
        &out,
        "^(\\!{3})\\s(?<TYPE>.*?)\\s(?<TITLE>.*?)$",
        "<div class=\"mdnote note-$2\"><b class=\"mdnote-title\">$3</b></div>\n",
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

    // undo code replacements
    out = out.replace("&!temp-ast;", "*");
    out = out.replace("&!temp-back;", "`");
    out = out.replace("&nbsp;1;\n", "&nbsp;\n");

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
            &format!("<rf style=\"justify-content: {align}\">{content}</rf>\n"),
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
            &format!("<r style=\"text-align: {align}\">{content}</r>\n"),
        );
    }

    // image with sizing
    let image_sizing_regex = RegexBuilder::new("(!)\\[(.*?)\\]\\((.*?)\\)\\:\\{(.*?)x(.*?)\\}")
        .multi_line(true)
        .build()
        .unwrap();

    for capture in image_sizing_regex.captures_iter(&out.clone()) {
        let title = capture.get(2).unwrap().as_str();
        let src = capture.get(3).unwrap().as_str();

        let width = capture.get(4).unwrap().as_str();
        let height = capture.get(5).unwrap().as_str();

        let result = &format!("<img alt=\"{title}\" title=\"{title}\" src=\"{src}\" style=\"width: {width}px; height: {height}px;\" />");
        out = out.replace(capture.get(0).unwrap().as_str(), result);
    }

    // normal image
    out = regex_replace(
        &out,
        "(!)\\[(.*?)\\]\\((.*?)\\)",
        "<img alt=\"$2\" title=\"$2\" src=\"$3\" />",
    );

    // anchor (auto)
    out = regex_replace(
        &out,
        "([^\"\\(]|^)(https\\:\\/\\/)(.*?)\\s",
        "<a href=\"https://$3\">https://$3</a>",
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
        "<a href=\"$2\">$1</a>",
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
        RegexBuilder::new("^(.*?)\\s*(\\n{2,})")
            .multi_line(true)
            .dot_matches_new_line(true),
        "<p>\n$1\n</p>",
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
