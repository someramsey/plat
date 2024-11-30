use std::iter::Peekable;
use crate::str;
use crate::task::collection::Collection;
use crate::task::data::num::Num;
use crate::task::data::str::{ch_to_str, concat_str, Str};
use crate::task::data::str_expr::{StrExpression, StrExpressionItem};
use crate::task::error::Error;
use crate::task::layers::fragmentize::{Fragment, Fragment};
use crate::task::position::Position;
use std::sync::Arc;
use std::vec::IntoIter;
use crate::task::node::Node;

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


struct TokenizationContext<'a> {
    collection: Collection<Token>,
    iterator: Peekable<IntoIter<Fragment<'a>>>
}

impl TokenizationContext<'_> {
    fn new(iterator: Peekable<IntoIter<Fragment>>) -> TokenizationContext {
        TokenizationContext { iterator, collection: Collection::new() }
    }

    fn next(&mut self) -> Option<Fragment> {
        return self.iterator.next();
    }

    fn peek(&mut self) -> Option<&Fragment> {
        return self.iterator.peek();
    }

    fn push(&mut self, data: Token) {
        self.collection.push(token);
    }
}


pub fn tokenize(fragments: Vec<Node<Fragment>>) -> Collection<Token> {
    let mut collection: Collection<Token> = Collection::new();
    let mut iter = fragments.into_iter().peekable();
    
    fragments.iter()

    while let Some(fragment) = iter.peek() {
        match fragment.data {
            Fragment::Numeric(_) => {}
            Fragment::Symbol(ch) => match ch {
                '"' => capture_string(&mut iter, &mut collection),
                '/' => capture_regex(&mut iter, &mut collection),
                '$' => capture_variable(&mut iter, &mut collection),
                _ => {
                    collection.push(Token {
                        data: TokenData::Symbol(ch),
                        position: fragment.position.clone(),
                    });
                }
            }

            Fragment::AlphaNumeric(str) => {
                collection.push(Token {
                    data: TokenData::Segment(Arc::from(str)),
                    position: fragment.position.clone(),
                });
            }
        }
    }

    return collection;
}

fn capture_variable(iter: &mut Peekable<IntoIter<Node<Fragment>>>, collection: &mut Collection<Token>) {
    iter.next();

    let identifier = match iter.next() {
        Some(next) => next,
        None => {
            collection.throw(Error {
                message: str!("Unexpected EOF while capturing variable"),
                position: Position::new(),
            });

            return;
        }
    };

    match identifier {
        Fragment { data: Fragment::AlphaNumeric(slice), position } => {
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

fn capture_regex(iter: &mut Peekable<IntoIter<Node<Fragment>>>, collection: &mut Collection<Token>) {
    let mut parts: Vec<Str> = Vec::new();

    while let Some(fragment) = iter.next() {
        match fragment.data {
            Fragment::Symbol('\\') => {
                iter.next();
            }

            Fragment::Symbol('/') => {
                collection.push(Token {
                    data: TokenData::Regex(concat_str(parts)),
                    position: fragment.position.clone(),
                });

                return;
            }

            Fragment::Numeric(slice) |
            Fragment::AlphaNumeric(slice) => parts.push(Arc::from(slice)),

            Fragment::Symbol(ch) => parts.push(ch_to_str(ch)),
        }
    }

    collection.throw(Error {
        message: str!("Unexpected EOF while capturing regex"),
        position: Position::new(),
    });
}

fn capture_string(iter: &mut Peekable<IntoIter<Node<Fragment>>>, collection: &mut Collection<Token>) {
    let mut expr = StrExpression::new();

    iter.next();

    while let Some(fragment) = iter.next() {
        match fragment.data {
            Fragment::Symbol(ch) => {
                match ch {
                    '\\' => {
                        iter.next();
                    }

                    '"' => {
                        collection.push(Token {
                            data: TokenData::String(expr),
                            position: fragment.position.clone(),
                        });

                        return;
                    }

                    '$' => {
                        if let Some(next) = iter.next() {
                            if let Fragment::AlphaNumeric(slice) = next.data {
                                expr.push(StrExpressionItem::Variable(Arc::from(slice)));
                            } else {
                                collection.throw(Error {
                                    message: str!("Expected variable identifier after '$'"),
                                    position: next.position.clone(),
                                });
                            }
                        }
                    }

                    _ => {
                        expr.push(StrExpressionItem::Literal(ch_to_str(ch)));
                    }
                }
            }

            Fragment::Numeric(slice) |
            Fragment::AlphaNumeric(slice) =>
                expr.push(StrExpressionItem::Literal(Arc::from(slice))),
        }
    }
}
























