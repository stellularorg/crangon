use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "markdown/argon.pest"]
pub struct ArParser;

pub fn parse(input: &str) -> Pair<'_, Rule> {
    let res = ArParser::parse(Rule::FILE, input);

    if res.is_err() {
        let err_unwrap = res.err().unwrap();
        panic!("({})\n{}", err_unwrap.variant, err_unwrap);
    }

    // return
    return res.unwrap().next().unwrap();
}
