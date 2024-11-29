use std::iter::Peekable;
use crate::str;
use crate::task::collection::Collection;
use crate::task::data::num::Num;
use crate::task::data::str::{ch_to_str, concat_str, Str};
use crate::task::data::str_expr::{StrExpression, StrExpressionItem};
use crate::task::error::Error;
use crate::task::layers::fragmentize::{Fragment, FragmentData};
use crate::task::position::Position;
use std::sync::Arc;
use std::vec::IntoIter;

#[derive(Debug)]
pub enum TokenData {
    Segment(Str),
    Variable(Str),
    Symbol(char),
    String(StrExpression),
    Number(Num),
    Regex(Str),
    Range(Num, Num),
}

impl TokenData {
    pub fn stringify(&self) -> Str {
        match self {
            TokenData::Segment(str) => str.clone(),
            TokenData::Symbol(ch) => Arc::from(format!("symbol '{}'", ch)),
            TokenData::Number(num) => num.stringify(),
            TokenData::String(str) => Arc::from("string"),
            TokenData::Regex(str) => Arc::from(format!("regex (\"{}\")", str)),
            TokenData::Variable(str) => Arc::from(format!("${}", str)),
            TokenData::Range(start, end) => Arc::from(format!("range {}..{}", start.stringify(), end.stringify())),
        }
    }
}

#[derive(Debug)]
pub struct Token {
    pub data: TokenData,
    pub position: Position,
}

pub fn tokenize(fragments: Vec<Fragment>) -> Collection<Token> {
    let mut collection: Collection<Token> = Collection::new();
    let mut iter = fragments.into_iter().peekable();

    while let Some(fragment) = iter.peek() {
        match fragment.data {

            FragmentData::Numeric(_) => {}
            FragmentData::Symbol(ch) => match ch {
                '"' => capture_string(&mut iter, &mut collection),
                '/' => capture_regex(&mut iter, &mut collection),
                '$' => capture_variable(&mut iter, &mut collection),
                _ => {}
            }

            FragmentData::AlphaNumeric(str) => {
                collection.push(Token {
                    data: TokenData::Segment(Arc::from(str)),
                    position: fragment.position.clone(),
                });
            },
        }
    }

    return collection;
}

fn capture_variable(iter: &mut Peekable<IntoIter<Fragment>>, collection: &mut Collection<Token>) {
    let identifier = match iter.next() {
        Some(next) => next,
        None => {
            collection.throw(Error {
                message: str!("Unexpected EOF while capturing variable"),
                position: last.clone(),
            });

            return;
        }
    };

    match identifier {
        Fragment { data: FragmentData::AlphaNumeric(slice), position } => {
            collection.push(Token {
                data: TokenData::Variable(Arc::from(slice)),
                position: position.clone(),
            });
        }

        _ => {
            collection.throw(Error {
                message: str!("Expected variable identifier after '$'"),
                position: identifier.position.clone(),
            });
        }
    }
}

fn capture_regex(iter: &mut Peekable<IntoIter<Fragment>>, collection: &mut Collection<Token>) {
    let mut parts: Vec<Str> = Vec::new();

    while let Some(fragment) = iter.next() {
        match fragment.data {
            FragmentData::Symbol('\\') => {
                iter.next();
            }

            FragmentData::Symbol('/') => {
                collection.push(Token {
                    data: TokenData::Regex(concat_str(parts)),
                    position: fragment.position.clone(),
                });

                return;
            }

            FragmentData::Numeric(slice) |
            FragmentData::AlphaNumeric(slice) => parts.push(Arc::from(slice)),

            FragmentData::Symbol(ch) => parts.push(ch_to_str(ch)),
        }
    }

    collection.throw(Error {
        message: str!("Unexpected EOF while capturing regex"),
        position: Position::new(),
    });
}

fn capture_string(iter: &mut Peekable<IntoIter<Fragment>>, collection: &mut Collection<Token>) {
    let mut expr = StrExpression::new();

    iter.next();

    while let Some(fragment) = iter.next() {
        match fragment.data {
            FragmentData::Symbol('\\') => {
                iter.next();
            }

            FragmentData::Symbol('"') => {
                collection.push(Token {
                    data: TokenData::String(expr),
                    position: fragment.position.clone(),
                });

                return;
            }

            FragmentData::Symbol('$') => {
                if let Some(next) = iter.next() {
                    if let FragmentData::AlphaNumeric(slice) = next.data {
                        expr.push(StrExpressionItem::Variable(Arc::from(slice)));
                    } else {
                        collection.throw(Error {
                            message: str!("Expected variable identifier after '$'"),
                            position: next.position.clone(),
                        });
                    }
                }
            }

            FragmentData::Symbol(ch) =>
                expr.push(StrExpressionItem::Literal(Arc::from(ch.to_string()))),

            FragmentData::Numeric(slice) |
            FragmentData::AlphaNumeric(slice) =>
                expr.push(StrExpressionItem::Literal(Arc::from(slice))),
        }
    }


}
























