use std::{env, ops::Index};

pub fn collect_arguments() -> Vec<String> {
    return env::args().collect::<Vec<String>>();
}

pub fn get_named_argument(args: &Vec<String>, name: &str) -> Option<String> {
    for (i, v) in args.iter().enumerate() {
        // if name does not match, continue
        if v != &format!("--{}", name) {
            continue;
        };

        // return value
        let val: &String = args.index(i + 1);

        // ...make sure val exists (return None if it doesn't!)
        if val.is_empty() {
            return Option::None;
        }

        return Option::Some(String::from(val));
    }

    return Option::None;
}