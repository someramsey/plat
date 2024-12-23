use crate::task::error::Error;
use crate::task::layers::tokenize::Token;
use crate::task::nodes::collection::NodeCollection;
use crate::task::nodes::iterator::NodeIter;
use crate::task::nodes::node::Node;
use crate::task::position::Position;
use crate::task::value::string::StringExpression;
use crate::task::value::Value;
use crate::{expect_node, node, some_node};
use crate::task::value::range::RangeValue;

pub enum FieldType {
    Text,
    Integer,
    Decimal,
    Switch,
}

impl FieldType {
    pub fn from_str(value: &str) -> Result<Self, &str> {
        match value {
            "Text" => Ok(Self::Text),
            "Integer" => Ok(Self::Integer),
            "Decimal" => Ok(Self::Decimal),
            "Switch" => Ok(Self::Switch),
            _ => Err("Text, Integer, Decimal or Switch"),
        }
    }
}

pub enum Validator {
    Range(i32, i32),
    Regex(Box<str>),
    Switch(Box<[StringExpression]>),
}

pub enum MatchPattern<'a> {
    Variable(&'a str),
    Value(Value),
}
pub struct MatchCase<'a> {
    pattern: MatchPattern<'a>,
    expression: Box<[Compound<'a>]>,
}
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
        match data {
            Token::Identifier(identifier) => declaration(&mut iter, &mut collection, identifier, position),
            Token::Symbol('>') => prompt(&mut iter, &mut collection, position),
            Token::Symbol(':') => validator(&mut iter, &mut collection),

            _ => collection.throw(Error::Invalid { received: Box::from(format!("{}", data)), position }),
        }
    }

    return collection;
}

fn validator(mut iter: &mut NodeIter<Token>, mut collection: &mut NodeCollection<Compound>) {
    if !matches!(iter.peek(), some_node!(Token::Symbol(':'))) {
        //No error needed in case of a single colon
        iter.skip();
        return;
    }

    match iter.next() {
        some_node!(data, position) => match data {
            Token::Value(Value::Range(RangeValue(begin, end))) => collection.try_collect(|| Node::new(Compound::Validator(Validator::Range(begin, end)), position)),
            Token::Value(Value::Regex(regex)) => collection.try_collect(|| Node::new(Compound::Validator(Validator::Regex(regex)), position)),
            Token::Symbol('[') => {
                let options = collect_switch_options(&mut iter, &mut collection, &position);
                
                if let Some(options) = options {
                    collection.try_collect(|| Node::new(Compound::Validator(Validator::Switch(options.into_boxed_slice())), position));
                }
            }

            _ => collection.throw(Error::Unexpected {
                expected: Box::from("Range, Regex or Symbol"),
                received: Box::from(format!("{}", data)),
                position,
            }),
        }

        None => collection.throw(Error::EndOfFile {
            expected: Box::from("Range, Regex or Symbol")
        })
    }
}

fn prompt(iter: &mut NodeIter<Token>, collection: &mut NodeCollection<Compound>, position: Position) {
    let prompt = match expect_node!(iter.next(), some_node!(Token::Value(Value::String(str))) => str) {
        Ok(prompt) => prompt,
        Err(err) => {
            collection.throw(err);
            return;
        }
    };

    collection.try_collect(|| Node::new(Compound::Prompt(prompt), position));
}

fn declaration<'a>(iter: &mut NodeIter<Token>, collection: &mut NodeCollection<Compound<'a>>, identifier: &'a str, position: Position) {
    if let Err(err) = expect_node!(iter.next(), some_node!(Token::Symbol(':'))) {
        collection.throw(err);
        return;
    }

    if let NodeCollection::Ok(ref mut tokens) = collection {
        let field_type = match expect_node!(iter.next(), some_node!(Token::Segment(type_name)) => type_name) {
            Ok(identifier) => match FieldType::from_str(identifier) {
                Ok(field_type) => field_type,
                Err(err) => {
                    collection.throw(Error::Other { message: Box::from(err), position });
                    return;
                }
            },

            Err(err) => {
                collection.throw(err);
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
                            expected: Box::from("','"),
                            received: Box::from("String"),
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
                            expected: Box::from("String"),
                            received: Box::from("','"),
                            position,
                        });
                    }
                }

                _ => {
                    collection.throw(Error::Unexpected {
                        expected: Box::from("Range, Regex or Symbol"),
                        received: Box::from(format!("{}", data)),
                        position,
                    });

                    return None;
                }
            },

            None => {
                collection.throw(Error::EndOfFile { expected: Box::from("Range, Regex or Symbol") });
                return None;
            }
        }
    }

    return Some(options);
}
