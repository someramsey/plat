//TODO: start throwing errors in places like eof at string capture or reading invalid ranges
use crate::task::collection::Collection;
use crate::task::position::Position;
use crate::task::tokenizer::num::Num;
use crate::task::tokenizer::str::Str;
use crate::task::tokenizer::str_expr::{StrExpression, StrExpressionItem};
use std::str::Chars;
use std::sync::Arc;
use std::vec::IntoIter;
use crate::str;
use crate::task::error::Error;

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

enum FragmentData<'a> {
    AlphaNumeric(&'a str),
    Numeric(&'a str),
    Symbol(&'a str),
}

struct Fragment<'a> {
    pub data: FragmentData<'a>,
    pub position: Position,
}

enum State {
    None,
    Symbol,
    Whitespace,
    Alphanumeric,
    Numeric,
}

fn fragmentize(data: &str) -> Vec<Fragment> {
    let mut fragments: Vec<Fragment> = Vec::new();

    let mut head = 0;
    let mut tail = 0;

    let mut position = Position::new();

    let mut state = State::None;
    let mut chars = data.chars();

    for ch in chars.by_ref() {
        match state {
            State::None => {
                if ch.is_whitespace() {
                    state = State::Whitespace;
                } else if ch.is_numeric() {
                    state = State::Numeric;
                } else if ch.is_alphanumeric() {
                    state = State::Alphanumeric;
                } else {
                    state = State::Symbol;
                }
            }

            State::Whitespace => {
                if ch.is_numeric() {
                    state = State::Numeric;
                } else if ch.is_alphanumeric() {
                    state = State::Alphanumeric;
                } else if !ch.is_whitespace() {
                    state = State::Symbol;
                }

                tail = head;
            }

            State::Symbol => {
                if ch.is_whitespace() {
                    state = State::Whitespace;
                } else if ch.is_numeric() {
                    state = State::Numeric;
                } else if ch.is_alphanumeric() {
                    state = State::Alphanumeric;
                }

                fragments.push(Fragment {
                    data: FragmentData::Symbol(&data[tail..head]),
                    position: position.clone(),
                });

                tail = head;
            }
            State::Alphanumeric => {
                if !ch.is_alphanumeric() {
                    if ch.is_whitespace() {
                        state = State::Whitespace;
                    } else {
                        state = State::Symbol;
                    }

                    fragments.push(Fragment {
                        data: FragmentData::AlphaNumeric(&data[tail..head]),
                        position: position.clone(),
                    });

                    tail = head;
                }
            }

            State::Numeric => {
                if !ch.is_numeric() {
                    if ch.is_alphabetic() {
                        state = State::Alphanumeric;
                    } else if ch.is_whitespace() {
                        state = State::Whitespace;
                    } else {
                        state = State::Symbol;
                    }


                    fragments.push(Fragment {
                        data: FragmentData::Numeric(&data[tail..head]),
                        position: position.clone(),
                    });

                    tail = head;
                }
            }
        }

        head += 1;
    }

    match state {
        State::Alphanumeric => {
            fragments.push(Fragment {
                data: FragmentData::AlphaNumeric(&data[tail..head]),
                position: position.clone(),
            });
        }

        State::Numeric => {
            fragments.push(Fragment {
                data: FragmentData::Numeric(&data[tail..head]),
                position: position.clone(),
            });
        }

        _ => {}
    }

    return fragments;
}

pub fn tokenize(data: &str) -> Collection<Token> {
    let mut collection: Collection<Token> = Collection::new();

    let fragments = fragmentize(data);
    let mut iter = fragments.into_iter();

    while let Some(fragment) = iter.next() {
        if let FragmentData::Symbol(ch) = fragment.data {
            if ch == "/" {
                capture_regex(&mut iter, &mut collection);
            } else if ch == "\"" {
                capture_string(&mut iter, &mut collection);
            }
        }
    }

    return collection;
}

fn capture_regex(iter: &mut IntoIter<Fragment>, collection: &mut Collection<Token>) {
    let mut string: String = String::new();

    while let Some(fragment) = iter.next() {
        match fragment.data {
            FragmentData::Symbol("\\") => {
                iter.next();
            },

            FragmentData::Symbol("//") => {
                collection.push(Token {
                    data: TokenData::Regex(Arc::from(string)),
                    position: fragment.position.clone(),
                });

                return;
            }

            FragmentData::Symbol(slice) |
            FragmentData::Numeric(slice) |
            FragmentData::AlphaNumeric(slice) => {
                string.push_str(slice);
            }
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
            FragmentData::Symbol("\\") => {
                iter.next();
            },

            FragmentData::Symbol("\"") => {
                collection.push(Token {
                    data: TokenData::String(expr),
                    position: fragment.position.clone(),
                });

                return;
            }

            FragmentData::Symbol("$") => {
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

            FragmentData::Symbol(slice) |
            FragmentData::Numeric(slice) |
            FragmentData::AlphaNumeric(slice) => {
                expr.push(StrExpressionItem::Literal(Arc::from(slice)));
            }
        }
    }

    collection.throw(Error {
        message: str!("Unexpected EOF while capturing string"),
        position: Position::new(),
    });
}


//
// fn capture_state(ch: char) -> CaptureState {
//     match ch {
//         '"' => CaptureState::String,
//         '/' => CaptureState::Regex,
//         '\n' => CaptureState::Newline,
//         '$' => CaptureState::Variable,
//         '{' | '}' | ';' | ':' | '|' | '>' => CaptureState::Symbol,
//         _ => {
//             if ch.is_numeric() {
//                 CaptureState::Number
//             } else if ch.is_whitespace() {
//                 CaptureState::WhiteSpace
//             } else {
//                 CaptureState::None
//             }
//         }
//     }
// }
//
// pub fn tokenize(data: &str) -> Collection<Token> {
//     let len = data.len();
//     let mut chars = data.chars();
//
//     let mut cursor = Cursor::new(data);
//     let mut collection: Collection<Token> = Collection::new();
//
//     while let Some(ch) = chars.next() {
//         cursor.shift();
//
//         let state = capture_state(ch);
//
//         match state {
//             CaptureState::None => {
//                 if cursor.head == len && cursor.selection() > 0 {
//                     let slice = cursor.slice();
//                     cursor.pull();
//
//                     collection.push(Token {
//                         data: TokenData::Segment(slice),
//                         position: cursor.position.clone(),
//                     });
//                 }
//
//                 continue;
//             }
//             _ => {
//                 if cursor.selection() > 1 {
//                     let slice = cursor.slice_off(1);
//
//                     collection.push(Token {
//                         data: TokenData::Segment(Arc::from(slice)),
//                         position: cursor.position.clone(),
//                     });
//                 }
//
//                 cursor.pull();
//             }
//         }
//
//         match state {
//             CaptureState::Newline => cursor.newline(),
//
//             CaptureState::Symbol => {
//                 cursor.pull();
//
//                 collection.push(Token {
//                     data: TokenData::Symbol(ch),
//                     position: cursor.position.clone(),
//                 });
//             }
//
//             //TODO: add a cursor struct to contain head, tail, column, line vars
//             CaptureState::Regex => capture_regex(data, &mut chars, &mut cursor, &mut collection),
//             CaptureState::String => capture_string(data, &mut chars, &mut cursor, &mut collection),
//             CaptureState::Variable => capture_variable(data, &mut chars, &mut cursor, &mut collection),
//             CaptureState::Number => capture_number(data, &mut chars, &mut cursor, &mut collection),
//
//             _ => unreachable!("Invalid state, (char: '{}', state: {:?})", ch, state),
//         }
//     }
//
//     return collection;
// }
//
//
// fn capture_number(data: &str, chars: &mut Chars, cursor: &mut Cursor, collector: &mut Collection<Token>) {
//     let mut ranged = true;
//     let mut mantissa = false;
//
//     for ch in chars.by_ref() {
//         cursor.newline();
//
//         if (!ch.is_numeric()) {
//             //TODO: impl
//         }
//     }
//
//     let slice = cursor.slice_off(1);
//     cursor.pull();
//
//     collector.push(Token {
//         data: TokenData::Variable(slice),
//         position: cursor.position.clone(),
//     });
// }
//
// fn capture_variable(data: &str, chars: &mut Chars, cursor: &mut Cursor, collector: &mut Collection<Token>) {
//     for ch in chars.by_ref() {
//         cursor.shift();
//
//         if (!ch.is_alphanumeric()) {
//             break;
//         }
//     }
//
//     let slice = cursor.slice_off(1);
//     cursor.pull();
//
//     collector.push(Token {
//         data: TokenData::Variable(slice),
//         position: cursor.position.clone(),
//     });
// }
//
// fn capture_regex(data: &str, chars: &mut Chars, cursor: &mut Cursor, collector: &mut Collection<Token>) {
//     for ch in chars.by_ref() {
//         cursor.shift();
//
//         if ch == '\n' {
//             cursor.newline();
//         } else if ch == '\\' {
//             cursor.shift();
//         } else if ch == '/' {
//             break;
//         }
//     }
//
//     let slice = cursor.slice_off(1);
//     cursor.pull();
//
//     collector.push(Token {
//         data: TokenData::Regex(slice),
//         position: cursor.position.clone(),
//     });
// }
//
// fn capture_string(data: &str, chars: &mut Chars, cursor: &mut Cursor, collection: &mut Collection<Token>) {
//     let mut expression: StrExpression = Vec::new();
//
//     loop {
//         match chars.next() {
//             Some(ch) => {
//                 cursor.shift();
//
//                 if ch == '\n' {
//                     cursor.newline();
//                 } else if ch == '\\' {
//                     cursor.shift();
//                 } else if ch == '$' {
//                     if cursor.selection() > 0 {
//                         let slice = cursor.slice_off(2);
//                         expression.push(StrExpressionItem::Literal(slice));
//                     }
//
//                     cursor.pull();
//                     read_interpolated_variable(&data, chars, cursor, &mut expression);
//                 } else if ch == '"' {
//                     break;
//                 }
//             }
//
//             None => {
//                 collection.throw(Error {
//                     message: str!("Unexpected EOF while capturing string"),
//                     position: cursor.position.clone(),
//                 });
//
//                 return;
//             }
//         }
//     }
//
//     if cursor.selection() > 1 { //TODO: test if 1 is correct instead of 0
//         let slice= cursor.slice_off(1);
//         expression.push(StrExpressionItem::Literal(slice));
//     }
//
//     cursor.pull();
//
//     collection.push(Token {
//         data: TokenData::String(expression),
//         position: cursor.position.clone(),
//     });
// }
//
// fn read_interpolated_variable(data: &str, chars: &mut Chars, cursor: &mut Cursor, expression: &mut StrExpression) {
//     for ch in chars.by_ref() {
//         cursor.shift();
//
//         if ch == '\n' {
//             cursor.newline();
//         } else if ch == '}' {
//             let slice = cursor.slice_off(1);
//             expression.push(StrExpressionItem::Variable(slice));
//
//             cursor.pull();
//             break;
//         }
//     }
// }
