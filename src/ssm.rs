// structured style markup (language)
// a SIMPLE regex parsed language which compiles into CSS

#[derive(Debug, Clone)]
pub struct SSMProgram {
    pub viewport_restriction: String,
    pub uses: Vec<SSMUseBlock>,
    pub members: Vec<SSMMemberBlock>,
    pub sets: Vec<SSMSetBlock>,
    pub errors: Vec<SSMError>,
}

#[derive(Debug, Clone)]
pub struct SSMError {
    pub message: String,
    pub flagged: String, // flagged text
}

#[derive(Debug, Clone)]
pub struct SSMUseBlock {
    // a "USE" block defines information about the program being parsed, such as the version:
    // USE ssm 1.0
    pub reference: String,
}

#[derive(Debug, Clone)]
pub struct SSMMemberBlock {
    // a "MEMBER" is a CSS selector reference which stores the selector by a custom name
    pub by: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SSMSetBlock {
    // a "SET" block is used to set a property to a value on a member
    pub property: String,
    pub value: String,
    pub for_member: Option<String>,
    pub at: Option<String>, // (for animations, corresponds to the animation stage)
}

// parse an ssm into a program tree
fn program_tree(input: String) -> SSMProgram {
    let mut program: SSMProgram = SSMProgram {
        viewport_restriction: String::new(),
        uses: Vec::new(),
        members: Vec::new(),
        sets: Vec::new(),
        errors: Vec::new(),
    };

    // ...

    // regex
    let when_regex = regex::RegexBuilder::new(r"^(WHEN)\s(?<RESTRICTION>.*?)($|\%)")
        .multi_line(true)
        .build()
        .unwrap();

    let use_regex = regex::RegexBuilder::new(r"^(USE)\s(?<REFERENCE>.*?)($|\%)")
        .multi_line(true)
        .build()
        .unwrap();

    let member_regex =
        regex::RegexBuilder::new(r"^(MEMBER)\s(?<SELECTOR>.*?)(\sNAMED\s)(?<NAME>.*?)($|\%)")
            .multi_line(true)
            .build()
            .unwrap();

    let set_regex = regex::RegexBuilder::new(
        r"^(SET)\s(?<PROPERTY>.*?)(TO)(?<VALUE>.*?)(FOR)\s(?<MEMBER>.*?)($|\%)",
    )
    .multi_line(true)
    .build()
    .unwrap();

    let set_at_regex = regex::RegexBuilder::new(
        r"^(SET)\s(?<PROPERTY>.*?)(TO)(?<VALUE>.*?)(AT)\s(?<AT>.*?)($|\%)",
    )
    .multi_line(true)
    .build()
    .unwrap();

    // matches
    for capture in when_regex.captures_iter(&input) {
        // WHEN is used to define a custom viewport restriction (media query)
        let restriction = capture.name("RESTRICTION").unwrap().as_str();
        program.viewport_restriction = restriction.to_string();
    }

    for capture in use_regex.captures_iter(&input) {
        let reference = capture.name("REFERENCE").unwrap().as_str();
        program.uses.push(SSMUseBlock {
            reference: reference.to_string(),
        });
    }

    for capture in member_regex.captures_iter(&input) {
        let selector = capture.name("SELECTOR").unwrap().as_str();
        let name = capture.name("NAME").unwrap().as_str();

        // make sure member isn't already registered
        let existing = program.members.iter().find(|m| m.name == name);

        if existing.is_some() {
            program.errors.push(SSMError {
                message: String::from(
                    "a member with this name is already registered in this environment",
                ),
                flagged: name.to_string(),
            });

            continue;
        }

        // ...
        program.members.push(SSMMemberBlock {
            by: selector.to_string(),
            name: name.to_string(),
        });
    }

    for capture in set_regex.captures_iter(&input) {
        let property = capture.name("PROPERTY").unwrap().as_str().trim();
        let value = capture.name("VALUE").unwrap().as_str().trim();
        let member = capture.name("MEMBER").unwrap().as_str().trim();

        // ...
        program.sets.push(SSMSetBlock {
            property: property.to_string(),
            value: value.to_string(),
            for_member: Option::Some(member.to_string()),
            at: Option::None,
        });
    }

    for capture in set_at_regex.captures_iter(&input) {
        let property = capture.name("PROPERTY").unwrap().as_str().trim();
        let value = capture.name("VALUE").unwrap().as_str().trim();
        let at = capture.name("AT").unwrap().as_str().trim();

        program.sets.push(SSMSetBlock {
            property: property.to_string(),
            value: value.to_string(),
            for_member: Option::None,
            at: Option::Some(at.to_string()),
        });
    }

    // return
    return program;
}

// parse an ssm program and return the compiled CSS
pub fn parse_ssm_program(input: String) -> String {
    let mut out: String = String::new();

    // get tree
    let tree: SSMProgram = program_tree(input);
    let mut parsed_sets: Vec<&SSMSetBlock> = Vec::new();

    // ...
    for member in tree.members.iter() {
        // get all sets
        let sets = tree
            .sets
            .iter()
            // TODO: remove the .clone() when filtering sets
            .filter(|s| s.for_member == Option::Some(member.name.clone()));

        // build out
        let mut member_out = format!(
            r"{}[named='{}'] {{}}
{} {{REMOVE}}}}",
            member.by, member.name, member.by
        );

        for set in sets {
            member_out.push_str(&format!("{}: {};", set.property, set.value));
            parsed_sets.push(set); // push set so we don't parse it again
        }

        // add to out
        member_out = member_out.replace("REMOVE}}", "");
        out += &format!("{member_out}}}");
    }

    for set in tree.sets.iter() {
        if parsed_sets.contains(&set) {
            // this set has already been parsed previously in the program, so we're going to ignore it
            continue;
        }

        // build out
        let mut member_out = if set.at.is_some() {
            set.at.as_ref().unwrap().to_owned() + r" {REMOVE}}"
        } else {
            set.for_member.as_ref().unwrap().to_owned()
                + r" {REMOVE}} --ssm-warn: 'unknown member';"
        };

        member_out.push_str(&format!("{}: {};", set.property, set.value));

        // add to out
        member_out = member_out.replace("REMOVE}}", "");
        out += &format!("{member_out}}}");
    }

    // errors
    for error in tree.errors.iter() {
        // add to out
        out += &format!(
            r"body::before {{
                position: absolute;
                bottom: 0;
                left: 0;
                content: '[SSMError]: \'{}\' | {}';
                border-left: solid red 5px;
                background: black;
                padding-left: 1rem;
            }}",
            error.flagged, error.message
        );
    }

    // remove comments
    out = out.replace(r"^@\s(.*?)$", "");

    // handle use
    if tree.uses.len() > 1 {
        // animation definition ("USE ssm::anim {name}")
        let anim_use = tree
            .uses
            .iter()
            .find(|u| u.reference.starts_with("ssm::anim"));

        if anim_use.is_some() {
            let anim_name = anim_use
                .unwrap()
                .reference
                .split("ssm:anim ")
                .skip(1)
                .collect::<String>();

            out = format!("@keyframes {} {{{}}}", anim_name, out);
        }

        // inherit (references that start with "http" are imported)
        for use_statement in tree.uses {
            if !use_statement.reference.starts_with("http") {
                continue;
            };

            out = format!("@import url(\"{}\");{}", use_statement.reference, out);
        }
    }

    // handle viewport_restriction
    if !tree.viewport_restriction.is_empty() {
        out = format!(
            "@media screen and ({}) {{{}}}",
            tree.viewport_restriction, out
        );
    }

    // return
    return out;
}

pub fn parse_ssm_blocks(input: String) -> String {
    // parses all SSM blocks in a Markdown input
    let mut out: String = String::new();

    let ssm_regex = regex::RegexBuilder::new("(ssm\\#)(?<CONTENT>.*?)\\#")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    for capture in ssm_regex.captures_iter(&input.clone()) {
        let content = capture.name("CONTENT").unwrap().as_str();

        // compile
        let css = parse_ssm_program(content.to_string());

        // replace
        out += &css;
    }

    // return
    return out;
}
