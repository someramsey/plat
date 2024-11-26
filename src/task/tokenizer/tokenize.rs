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
//numeric
//numeric.numeric
//numeric..numeric
//numeric.numeric..numeric
//numeric.numeric..numeric.numeric
fn capture(fragment: Fragment, mut collection: &mut Collection<Token>, mut iter: &mut IntoIter<Fragment>) {
    if let FragmentData::Symbol(ch) = fragment.data {
        if ch == '/' {
            capture_regex(&mut iter, &mut collection);
        } else if ch == '"' {
            capture_string(&mut iter, &mut collection);
        } else {
            collection.push(Token {
                data: TokenData::Symbol(ch),
                position: fragment.position.clone(),
            });
        }
    } else if let FragmentData::Numeric(first_slice) = fragment.data {
        if let Some(next) = iter.next() {
            if let FragmentData::Symbol('.') = next.data {
                //expect numeric or range
                if let Some(second) = iter.next() {
                    match second.data {
                        //numeric
                        FragmentData::Numeric(second_slice) => {
                            //check range after decimal
                            if let Some(next) = iter.next() {
                                if let FragmentData::Symbol('.') = next.data {
                                    if let Some(next) = iter.next() {
                                        if let FragmentData::Symbol('.') = next.data {
                                            //RIGHT
                                            //expect numeric
                                            if let Some(next) = iter.next() {
                                                if let FragmentData::Numeric(third_slice) = next.data {
                                                    if let Some(next) = iter.next() {
                                                        if let FragmentData::Symbol('.') = next.data {
                                                            if let Some(next) = iter.next() {
                                                                if let FragmentData::Numeric(fourth_slice) = next.data {
                                                                    println!("range {}.{}..{}.{}", first_slice, second_slice, third_slice, fourth_slice);
                                                                } else {
                                                                    collection.throw(Error {
                                                                        message: str!("Expected numeric after '.'"),
                                                                        position: second.position.clone(),
                                                                    });

                                                                    capture(next, collection, iter);
                                                                }
                                                            } else {
                                                                collection.throw(Error {
                                                                    message: str!("Expected numeric after '.'"),
                                                                    position: second.position.clone(),
                                                                });
                                                            }
                                                        } else {
                                                            println!("range {}.{}..{}", first_slice, second_slice, third_slice);

                                                            capture(next, collection, iter);
                                                        }
                                                    } else {
                                                        println!("range {}.{}..{}", first_slice, second_slice, third_slice);
                                                    }
                                                } else {
                                                    collection.throw(Error {
                                                        message: str!("Expected numeric after '..'"),
                                                        position: second.position.clone(),
                                                    });

                                                    capture(next, collection, iter);
                                                }
                                            } else {
                                                collection.throw(Error {
                                                    message: str!("Expected numeric after '..'"),
                                                    position: second.position.clone(),
                                                });
                                            }
                                        } else {
                                            collection.throw(Error {
                                                message: str!("Expected '.'"),
                                                position: second.position.clone(),
                                            });

                                            capture(next, collection, iter);
                                        }
                                    } else {
                                        collection.throw(Error {
                                            message: str!("Expected '.'"),
                                            position: second.position.clone(),
                                        });
                                    }
                                } else {
                                    println!("decimal {}.{}", first_slice, second_slice);
                                    capture(next, collection, iter);
                                }
                            } else {
                                println!("decimal {}.{}", first_slice, second_slice);
                            }
                        }

                        //range
                        FragmentData::Symbol('.') => {
                            if let Some(next) = iter.next() {
                                if let FragmentData::Symbol('.') = next.data {
                                    //RIGHT
                                    //expect numeric
                                    if let Some(next) = iter.next() {
                                        if let FragmentData::Numeric(third_slice) = next.data {
                                            if let Some(next) = iter.next() {
                                                if let FragmentData::Symbol('.') = next.data {
                                                    if let Some(next) = iter.next() {
                                                        if let FragmentData::Numeric(fourth_slice) = next.data {
                                                            println!("range {}..{}.{}", first_slice, third_slice, fourth_slice);
                                                        } else {
                                                            collection.throw(Error {
                                                                message: str!("Expected numeric after '.'"),
                                                                position: second.position.clone(),
                                                            });

                                                            capture(next, collection, iter);
                                                        }
                                                    } else {
                                                        collection.throw(Error {
                                                            message: str!("Expected numeric after '.'"),
                                                            position: second.position.clone(),
                                                        });
                                                    }
                                                } else {
                                                    println!("range {}..{}", first_slice, third_slice);

                                                    capture(next, collection, iter);
                                                }
                                            } else {
                                                println!("range {}..{}", first_slice, third_slice);
                                            }
                                        } else {
                                            collection.throw(Error {
                                                message: str!("Expected numeric after '..'"),
                                                position: second.position.clone(),
                                            });

                                            capture(next, collection, iter);
                                        }
                                    } else {
                                        collection.throw(Error {
                                            message: str!("Expected numeric after '..'"),
                                            position: second.position.clone(),
                                        });
                                    }
                                } else {
                                    collection.throw(Error {
                                        message: str!("Expected '.'"),
                                        position: second.position.clone(),
                                    });

                                    capture(next, collection, iter);
                                }
                            } else {
                                collection.throw(Error {
                                    message: str!("Expected '.'"),
                                    position: second.position.clone(),
                                });
                            }
                        }

                        _ => {
                            collection.throw(Error {
                                message: str!("Expected numeric after '.'"),
                                position: second.position.clone(),
                            });

                            capture(next, collection, iter);
                        }
                    }
                } else {
                    collection.throw(Error {
                        message: str!("Unexpected EOF after '.'"),
                        position: fragment.position.clone(),
                    });
                }
            } else {
                println!("integer {}", first_slice);
                capture(next, collection, iter);
            }
        } else {
            println!("integer {}", first_slice);
        }
    }
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