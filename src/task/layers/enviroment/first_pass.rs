use std::fmt::format;
use crate::task::error::Error;
use crate::task::layers::tokenize::Token;
use crate::task::nodes::collection::NodeCollection;
use crate::task::nodes::iterator::NodeIter;
use crate::task::nodes::node::Node;
use crate::task::position::Position;
use crate::task::value::range::RangeValue;
use crate::task::value::string::StringExpression;
use crate::task::value::{Value, ALL_VALUES};
use crate::{expect_node, node, some_node};

const VALIDATOR_VALUE_TYPES: &str = "Range, Regex or Symbol";

type Expression<'a> = Box<[Node<Compound<'a>>]>;

#[derive(Debug)]
pub enum FieldType {
    Text,
    Integer,
    Decimal,
    Switch,
}

#[derive(Debug)]
pub enum Validator {
    Range(i32, i32),
    Regex(Box<str>),
    Switch(Box<[StringExpression]>),
}

#[derive(Debug)]
pub enum MatchPattern<'a> {
    Any,
    Variable(&'a str),
    Value(Value),
}

#[derive(Debug)]
pub struct MatchCase<'a> {
    patterns: Box<[MatchPattern<'a>]>,
    expression: Expression<'a>,
}

#[derive(Debug)]
pub enum Compound<'a> {
    Declaration {
        identifier: &'a str,
        field_type: FieldType,
    },
    Prompt(StringExpression),
    Validator(Validator),
    Match {
        identifier: &'a str,
        cases: Box<[MatchCase<'a>]>,
    },
}

pub fn first_pass(tokens: Vec<Node<Token>>) -> NodeCollection<Compound> {
    let mut iter = NodeIter::new(tokens);
    let mut collection = NodeCollection::new();

    while let some_node!(data, position) = iter.next() {
        top_most(&mut iter, &mut collection, data, position);
    }

    return collection;
}

fn top_most<'a>(mut iter: &mut NodeIter<Token<'a>>, mut collection: &mut NodeCollection<Compound<'a>>, data: Token<'a>, position: Position) {
    match data {
        Token::Identifier(identifier) => field_declaration(iter, collection, identifier, position),
        Token::Symbol('>') => field_prompt(iter, collection, position),
        Token::Symbol(':') => field_validator(iter, collection),
        Token::Segment("match") => match_statement(iter, collection, position),

        _ => collection.throw(Error::Invalid { received: format!("{}", data), position }),
    }
}

fn match_case_expression<'a>(iter: &mut NodeIter<Token<'a>>, collection: &mut NodeCollection<Compound>, position: Position) -> Expression<'a> {
    let mut scope_collection: NodeCollection<Compound<'a>> = NodeCollection::new();

    loop {
        match iter.next() {
            Some(node!(data, position)) => {
                match data {
                    Token::Symbol('}') => break,
                    _ => top_most(iter, &mut scope_collection, data, position)
                }
            }

            None => {
                collection.throw(Error::EndOfFile { expected: String::from("'}'") });
                break;
            }
        }
    }

    return match scope_collection {
        NodeCollection::Ok(expression) => expression.into_boxed_slice(),
        NodeCollection::Failed(errors) => {
            collection.throw_all(errors);
            Box::new([])
        }
    };
}
fn match_statement<'a>(iter: &mut NodeIter<Token<'a>>, collection: &mut NodeCollection<Compound<'a>>, position: Position) {
    let identifier = match expect_node!(iter.next(), some_node!(Token::Identifier(identifier)) => identifier) {
        Ok(identifier) => identifier,
        Err(err) => {
            collection.throw(err);
            return;
        }
    };

    if let Err(err) = expect_node!(iter.next(), some_node!(Token::Symbol('{'))) {
        collection.throw(err);
        return;
    }

    let mut patterns: Vec<MatchPattern> = Vec::new();
    let mut cases: Vec<MatchCase> = Vec::new();
    let mut seperator = false;

    fn push_pattern(collection: &mut NodeCollection<Compound>, received: String, position: Position, seperator: &mut bool, callback: impl FnOnce()) {
        if *seperator {
            collection.throw(Error::Unexpected {
                expected: String::from("'|'"),
                received, position,
            });
        } else {
            *seperator = true;
            callback();
        }
    }

    loop {
        match iter.next() {
            Some(node!(data, position)) => {
                match data {
                    Token::Symbol('}') => break,
                    Token::Symbol('{') => {
                        let boxed_patterns = patterns.into_boxed_slice();
                        let expression = match_case_expression(iter, collection, position);

                        cases.push(MatchCase { patterns: boxed_patterns, expression });

                        seperator = false;
                        patterns = Vec::new();
                    }

                    Token::Symbol('|') => {
                        if seperator {
                            seperator = false;
                        } else {
                            collection.throw(Error::Unexpected {
                                expected: String::from("Pattern"),
                                received: String::from("'|'"),
                                position,
                            });
                        }
                    }

                    Token::Symbol('*') => push_pattern(collection, format!("{}", data), position, &mut seperator, || patterns.push(MatchPattern::Any)),
                    Token::Identifier(identifier) => push_pattern(collection, format!("{}", data), position, &mut seperator, || patterns.push(MatchPattern::Variable(identifier))),
                    Token::Value(value) => push_pattern(collection, format!("{}", value), position, &mut seperator, || patterns.push(MatchPattern::Value(value))),

                    _ => {
                        collection.throw(Error::Invalid {
                            received: format!("{}", data),
                            position,
                        });

                        return;
                    }
                }
            }

            None => {
                collection.throw(Error::EndOfFile { expected: String::from("'}'") });
                return;
            }
        }
    }

    collection.try_collect(|| Node::new(Compound::Match { identifier, cases: cases.into_boxed_slice() }, position));
}

fn field_validator(mut iter: &mut NodeIter<Token>, mut collection: &mut NodeCollection<Compound>) {
    if !matches!(iter.peek(), some_node!(Token::Symbol(':'))) {
        return;
    }

    iter.skip();

    match iter.next() {
        some_node!(data, position) => match data {
            Token::Value(Value::Range(RangeValue(begin, end))) => collection.try_collect(|| Node::new(Compound::Validator(Validator::Range(begin, end)), position)),
            Token::Value(Value::Regex(regex)) => collection.try_collect(|| Node::new(Compound::Validator(Validator::Regex(regex)), position)),
            Token::Symbol('[') => {
                let options = collect_switch_options(iter, collection, &position);

                if let Some(options) = options {
                    collection.try_collect(|| Node::new(Compound::Validator(Validator::Switch(options.into_boxed_slice())), position));
                }
            }

            _ => collection.throw(Error::Unexpected {
                expected: String::from(VALIDATOR_VALUE_TYPES),
                received: format!("{}", data),
                position,
            }),
        }

        None => collection.throw(Error::EndOfFile {
            expected: String::from("Range, Regex or Symbol"),
        })
    }
}

fn field_prompt(iter: &mut NodeIter<Token>, collection: &mut NodeCollection<Compound>, position: Position) {
    let prompt = match expect_node!(iter.next(), some_node!(Token::Value(Value::String(str))) => str) {
        Ok(prompt) => prompt,
        Err(err) => {
            collection.throw(err);
            return;
        }
    };

    collection.try_collect(|| Node::new(Compound::Prompt(prompt), position));
}

fn field_declaration<'a>(iter: &mut NodeIter<Token>, collection: &mut NodeCollection<Compound<'a>>, identifier: &'a str, position: Position) {
    if let Err(err) = expect_node!(iter.next(), some_node!(Token::Symbol(':'))) {
        collection.throw(err);
        return;
    }

    if let NodeCollection::Ok(ref mut tokens) = collection {
        let type_name = match expect_node!(iter.next(), some_node!(Token::Segment(type_name)) => type_name) {
            Ok(field_type) => field_type,
            Err(err) => {
                collection.throw(err);
                return;
            }
        };

        let field_type = match type_name {
            "Text" => FieldType::Text,
            "Integer" => FieldType::Integer,
            "Decimal" => FieldType::Decimal,
            "Switch" => FieldType::Switch,
            _ => {
                collection.throw(Error::Other { message: String::from("Text, Integer, Decimal or Switch"), position });
                return;
            }
        };

        tokens.push(Node::new(Compound::Declaration { identifier, field_type }, position));
    } else {
        iter.skip();
    }
}

fn collect_switch_options(iter: &mut NodeIter<Token>, collection: &mut NodeCollection<Compound>, position: &Position) -> Option<Vec<StringExpression>> {
    let mut options = Vec::new();
    let mut coma = false;

    loop {
        match iter.next() {
            some_node!(data, position) => match data {
                Token::Symbol(']') => break,

                Token::Value(Value::String(expr)) => {
                    if coma {
                        collection.throw(Error::Unexpected {
                            expected: String::from("','"),
                            received: String::from("String"),
                            position,
                        });
                    } else {
                        coma = true;
                        options.push(expr)
                    }
                }

                Token::Symbol(',') => {
                    if coma {
                        coma = false;
                    } else {
                        collection.throw(Error::Unexpected {
                            expected: String::from("String"),
                            received: String::from("','"),
                            position,
                        });
                    }
                }

                _ => {
                    collection.throw(Error::Unexpected {
                        expected: String::from(VALIDATOR_VALUE_TYPES),
                        received: format!("{}", data),
                        position,
                    });

                    return None;
                }
            },

            None => {
                collection.throw(Error::EndOfFile { expected: String::from(VALIDATOR_VALUE_TYPES) });
                return None;
            }
        }
    }

    return Some(options);
}
