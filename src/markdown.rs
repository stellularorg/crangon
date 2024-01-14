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

    for capture in fenced_code_block_regex.captures(&out.clone()).iter() {
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
        out = regex_replace(&out, capture.get(1).unwrap().as_str(), &format!("<pre class=\"flex\" style=\"position: relative;\">
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
    // TODO: fix
    let table_of_contents: &mut Vec<Heading> = &mut Vec::new();
    let heading_regex = RegexBuilder::new("^(\\#+)\\s(.*?)$")
        .multi_line(true)
        .build()
        .unwrap();

    for capture in heading_regex.captures(&out.clone()).iter() {
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
        out = regex_replace(
            &out,
            capture.get(0).unwrap().as_str(),
            format!("<h{heading_type} id=\"{heading_id}\">{content}</h{heading_type}>").as_str(),
        )
    }

    // return
    return out.to_string();
}

#[allow(dead_code)]
fn regex_replace(input: &str, pattern: &str, replace_with: &str) -> String {
    return RegexBuilder::new(pattern)
        .build()
        .unwrap()
        .replace_all(input, replace_with)
        .to_string();
}
