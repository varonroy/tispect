use pest::{error::Error, iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "value/value.pest"]
struct ValueParser;

use super::{Value, ValueArray, ValueObject};

fn parse_value<'a>(pair: Pair<'a, Rule>) -> Value<'a> {
    match pair.as_rule() {
        Rule::object => Value::Object(ValueObject {
            collapse: false,
            map: pair
                .into_inner()
                .map(|pair| {
                    let mut inner_rules = pair.into_inner();
                    let name = inner_rules
                        .next()
                        .unwrap()
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str();
                    let value = parse_value(inner_rules.next().unwrap());
                    (name, value)
                })
                .collect(),
        }),
        Rule::array => Value::Array(ValueArray {
            collapse: false,
            arr: pair.into_inner().map(parse_value).collect(),
        }),
        Rule::string => Value::String(pair.into_inner().next().unwrap().as_str()),
        Rule::number => Value::Number(pair.as_span().as_str()),
        Rule::bool => Value::Bool(pair.as_str().parse().unwrap()),
        Rule::null => Value::Null,
        Rule::json
        | Rule::EOI
        | Rule::pair
        | Rule::value
        | Rule::inner
        | Rule::char
        | Rule::WHITESPACE => unreachable!(),
    }
}

pub fn parse<'a>(s: &'a str) -> Result<Value<'a>, Error<Rule>> {
    let json = ValueParser::parse(Rule::json, s)?.next().unwrap();
    Ok(parse_value(json))
}
