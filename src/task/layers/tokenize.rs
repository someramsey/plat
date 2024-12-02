use crate::{expect, check};
use crate::task::collection::Collection;
use crate::task::data::num::Num;
use crate::task::data::str::{ch_to_str, concat_str, Str};
use crate::task::data::str_expr::{StrExpression, StrExpressionItem};
use crate::task::error::{Error, ErrorCause, ErrorContext};
use crate::task::layers::fragmentize::Fragment;
use crate::task::node::{Node, NodeIter};
use std::sync::Arc;

#[derive(Debug)]
pub enum Token {
    Segment(Str),
    Variable(Str),
    Symbol(char),
    String(StrExpression),
    Number(Num),
    Regex(Str),
    Range(Num, Num),
}

impl Token {
    pub fn stringify(&self) -> Str {
        match self {
            Token::Segment(str) => str.clone(),
            Token::Symbol(ch) => Arc::from(format!("symbol '{}'", ch)),
            Token::Number(num) => num.stringify(),
            Token::String(str) => Arc::from("string"),
            Token::Regex(str) => Arc::from(format!("regex (\"{}\")", str)),
            Token::Variable(str) => Arc::from(format!("${}", str)),
            Token::Range(start, end) => Arc::from(format!("range {}..{}", start.stringify(), end.stringify())),
        }
    }
}

pub fn tokenize(fragments: Vec<Node<Fragment>>) -> Collection<Node<Token>> {
    let mut collection: Collection<Node<Token>> = Collection::new();
    let mut iter = NodeIter::new(&fragments);

    while let Some(fragment) = iter.current {
        match fragment.data {
            Fragment::Numeric(slice) => {
                // capture_num(&mut iter);
                if let Ok(num) = Num::from_str(slice) {
                    
                }
            }

            Fragment::Symbol(ch) => {
                let capture_result = match ch {
                    '"' => capture_string(&mut iter),
                    '/' => capture_regex(&mut iter),
                    '$' => capture_variable(&mut iter),
                    _ => Ok(Token::Symbol(ch))
                };

                match capture_result {
                    Ok(token) => collection.push(Node::new(token, fragment.position.clone())),
                    Err(err) => collection.throw(err)
                }
            }

            Fragment::AlphaNumeric(str) => {
                collection.push(Node::new(
                    Token::Segment(Arc::from(str)),
                    fragment.position.clone()
                ));
            }
        }

        iter.next();
    }

    return collection;
}

fn capture_variable(iter: &mut NodeIter<Fragment>) -> Result<Token, Error> {
    expect!(iter, Fragment::AlphaNumeric(x) => x)
        .map_err(|context| Error::with_context("Expected variable identifier after '$'", context))
        .map(|identifier| Token::Variable(Arc::from(identifier)))
}

fn capture_regex(iter: &mut NodeIter<Fragment>) -> Result<Token, Error> {
    let mut parts: Vec<Str> = Vec::new();

    while let Some(fragment) = iter.current {
        match fragment.data {
            Fragment::Symbol('\\') => {
                iter.next();
            }

            Fragment::Symbol('/') => {
                return Ok(Token::Regex(concat_str(parts)));
            }

            Fragment::Numeric(slice) |
            Fragment::AlphaNumeric(slice) => parts.push(Arc::from(slice)),

            Fragment::Symbol(ch) => parts.push(ch_to_str(ch)),
        }

        iter.next();
    }

    Err(Error::new("Failed to capture regex", iter.position.clone(), ErrorCause::EndOfFile))
}

fn capture_string(iter: &mut NodeIter<Fragment>) -> Result<Token, Error> {
    let mut expr = StrExpression::new();

    while let Some(fragment) = iter.current {
        match fragment.data {
            Fragment::Symbol(ch) => {
                match ch {
                    '"' => return Ok(Token::String(expr)),

                    '\\' => {
                        iter.next();
                    }

                    '$' => {
                        let slice = expect!(iter, Fragment::AlphaNumeric(x) => x)
                            .map_err(|context| Error::with_context("Expected variable identifier after '$'", context))?;

                        expr.push(StrExpressionItem::Variable(Arc::from(slice)));
                    }

                    _ => expr.push(StrExpressionItem::Literal(ch_to_str(ch)))
                }
            }

            Fragment::Numeric(slice) |
            Fragment::AlphaNumeric(slice) =>
                expr.push(StrExpressionItem::Literal(Arc::from(slice))),
        }

        iter.next();
    }

    Err(Error::new("Failed to capture string", iter.position.clone(), ErrorCause::EndOfFile))
}


