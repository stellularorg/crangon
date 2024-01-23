// the "booklist" is a file stored in the cwd which contains a list of blocked URLs
// these URLs can be claimed, but will always return the same dummy record when fetched from the database
// example booklist:
//
// ;
// url1;url2;url3
//
// example booklist 2:
//
// ,
// url1, url2, url3
//
// the first line of a booklist defines the separator (\n works)

use std::fs;

#[allow(dead_code)]
#[derive(Debug)]
struct BookList {
    separator: String,
    full: String,
}

#[allow(dead_code)]
fn fetch_booklist() -> BookList {
    let content = fs::read_to_string(format!("booklist.txt"));

    if content.is_err() {
        return BookList {
            separator: String::new(),
            full: String::new(),
        };
    }

    // get sep
    let content = content.unwrap();
    let sep = &content.lines().next();

    // return
    return BookList {
        separator: if sep.is_some() {
            sep.unwrap().to_string()
        } else {
            String::from("\n")
        },
        full: content,
    };
}

pub fn check_booklist(looking_for: &String) -> bool {
    // load booklist
    let list = fetch_booklist();

    // check for word
    let split: Vec<String> = list
        .full
        .split(&list.separator)
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    return split.contains(looking_for) | split.contains(&format!("\n{}", looking_for));
}
