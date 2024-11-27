use crate::str;
use crate::task::collection::Collection;
use crate::task::error::Error;
use crate::task::position::Position;
use crate::task::fragmentize::{Fragment, FragmentData};
use crate::task::tokenizer::num::Num;
use crate::task::tokenizer::str::Str;
use crate::task::tokenizer::str_expr::{StrExpression, StrExpressionItem};
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
    let mut iter = fragments.into_iter();

    while let Some(fragment) = iter.next() {
        capture(fragment, &mut collection, &mut iter);
    }

    return collection;
}

fn capture(fragment: Fragment, mut collection: &mut Collection<Token>, mut iter: &mut IntoIter<Fragment>) {
    if let FragmentData::Symbol(ch) = fragment.data {
        match ch {
            '/' => capture_regex(iter, collection),
            '"' => capture_string(iter, collection),
            '$' => capture_variable(fragment.position, iter, collection),

            _ => collection.push(Token {
                data: TokenData::Symbol(ch),
                position: fragment.position.clone(),
            })
        }
    } else if let FragmentData::Numeric(first_slice) = fragment.data {
        capture_numeric(first_slice, collection, iter);
    } else if let FragmentData::AlphaNumeric(slice) = fragment.data {
        collection.push(Token {
            data: TokenData::Segment(Arc::from(slice)),
            position: fragment.position.clone(),
        });
    }
}

fn capture_variable(last: Position, iter: &mut IntoIter<Fragment>, collection: &mut Collection<Token>) {
    let next = match iter.next() {
        Some(next) => next,
        None => {
            collection.throw(Error {
                message: str!("Unexpected EOF while capturing variable"),
                position: last.clone(),
            });

            return;
        }
    };
    
    match next {
        Fragment { data: FragmentData::AlphaNumeric(slice), position } => {
            collection.push(Token {
                data: TokenData::Variable(Arc::from(slice)),
                position: position.clone(),
            });
        }

        _ => {
            collection.throw(Error {
                message: str!("Expected variable identifier after '$'"),
                position: next.position.clone(),
            });
        }
    }
}

fn capture_numeric(first_slice: &str, mut collection: &mut Collection<Token>, mut iter: &mut IntoIter<Fragment>) {
    let left = match capture_num_value(first_slice, iter, collection) {
        Ok(num) => num,
        Err(error) => {
            collection.throw(error);
            return;
        }
    };

    if !matches!(
        (iter.next(), iter.next()),
        (
            Some(Fragment { data: FragmentData::Symbol('.'), .. }),
            Some(Fragment { data: FragmentData::Symbol('.'), .. })
        )
    ) {
        iter.next_back();
        iter.next_back();

        collection.push(Token {
            data: TokenData::Number(left),
            position: Position::new(),
        });

        return;
    }

    let second_slice = match iter.next() {
        Some(Fragment { data: FragmentData::Numeric(slice), .. }) => slice,
        None | Some(_) => {
            collection.throw(Error {
                message: str!("Expected right side of range after '..'"),
                position: Position::new(),
            });

            return;
        }
    };

    let right = match capture_num_value(second_slice, iter, collection) {
        Ok(num) => num,
        Err(error) => {
            collection.throw(error);
            return;
        }
    };

    collection.push(Token {
        data: TokenData::Range(left, right),
        position: Position::new(),
    });
}

fn capture_num_value(first_slice: &str, mut iter: &mut IntoIter<Fragment>, mut collection: &mut Collection<Token>) -> Result<Num, Error> {
    let last_position = match iter.next() {
        Some(Fragment { data: FragmentData::Symbol('.'), position }) => position,

        Some(_) | None => {
            iter.next_back();

            let value = first_slice.parse::<i32>()
                .expect("Unexpected error, failde to parse integer slice");

            return Ok(Num::Integer(value));
        }
    };

    return match iter.next() {
        Some(next) => match next.data {
            FragmentData::Numeric(second_slice) => {
                let value = format!("{}.{}", first_slice, second_slice)
                    .parse::<f32>()
                    .or(Err(Error {
                        message: str!("Failed to parse number"),
                        position: next.position.clone(),
                    }))?;

                Ok(Num::Decimal(value))
            }

            _ => {
                iter.next_back();

                Err(Error {
                    message: str!("Invalid token, expected numeric or range after '.'"),
                    position: next.position.clone(),
                })
            }
        }

        None => {
            iter.next_back();

            Err(Error {
                message: str!("Unexpected EOF, expected numeric after '.'"),
                position: last_position.clone(),
            })
        }
    };
}


fn capture_regex(iter: &mut IntoIter<Fragment>, collection: &mut Collection<Token>) {
    let mut string: String = String::new();

    while let Some(fragment) = iter.next() {
        match fragment.data {
            FragmentData::Symbol('\\') => {
                iter.next();
            }

            FragmentData::Symbol('/') => {
                collection.push(Token {
                    data: TokenData::Regex(Arc::from(string)),
                    position: fragment.position.clone(),
                });

                return;
            }

            FragmentData::Numeric(slice) |
            FragmentData::AlphaNumeric(slice) => string.push_str(slice),

            FragmentData::Symbol(ch) => string.push(ch),
        }
    }


    collection.throw(Error {
        message: str!("Unexpected EOF while capturing regex"),
        position: Position::new(),
    });
}

fn capture_string(iter: &mut IntoIter<Fragment>, collection: &mut Collection<Token>) {
    let mut expr: StrExpression = Vec::new();

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

            FragmentData::Numeric(slice) |
            FragmentData::AlphaNumeric(slice) => {
                expr.push(StrExpressionItem::Literal(Arc::from(slice)));
            }

            FragmentData::Symbol(slice) => {
                expr.push(StrExpressionItem::Literal(Arc::from(slice.to_string())));
            }
        }
    }

    collection.throw(Error {
        message: str!("Unexpected EOF while capturing string"),
        position: Position::new(),
    });
}