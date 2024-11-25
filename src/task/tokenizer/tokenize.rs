//TODO: start throwing errors in places like eof at string capture or reading invalid ranges
use crate::task::collection::Collection;
use crate::task::position::Position;
use crate::task::tokenizer::num::Num;
use crate::task::tokenizer::str::Str;
use crate::task::tokenizer::str_expr::{StrExpression, StrExpressionItem};
use std::str::Chars;
use std::sync::Arc;
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

enum CaptureState { Symbol, Newline, WhiteSpace, String, Regex, Variable, None, Number }

struct Cursor<'a> {
    data: &'a str,
    head: usize,
    tail: usize,
    position: Position,
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a str) -> Self {
        Self { data, head: 0, tail: 0, position: Position::new() }
    }

    pub fn slice(&self) -> Str {
        self.slice_off(0)
    }

    pub fn slice_off(&self, offset: usize) -> Str {
        Arc::from(&self.data[self.tail..self.head - offset])
    }

    pub fn shift(&mut self) {
        self.head += 1;
        self.position.shift();
    }

    pub fn pull(&mut self) {
        self.tail = self.head;
    }

    pub fn selection(&self) -> usize {
        self.head - self.tail
    }
}

fn capture_state(ch: char) -> CaptureState {
    match ch {
        '"' => CaptureState::String,
        '/' => CaptureState::Regex,
        '\n' => CaptureState::Newline,
        '$' => CaptureState::Variable,
        '{' | '}' | ';' | ':' | '|' | '>' => CaptureState::Symbol,
        _ => {
            if ch.is_numeric() {
                CaptureState::Number
            } else if ch.is_whitespace() {
                CaptureState::WhiteSpace
            } else {
                CaptureState::None
            }
        }
    }
}

pub fn tokenize(data: &str) -> Collection<Token> {
    let len = data.len();
    let mut chars = data.chars();

    let mut cursor = Cursor::new(data);
    let mut position = Position::new();

    let mut collection: Collection<Token> = Collection::new();

    while let Some(ch) = chars.next() {
        cursor.shift();

        let state = capture_state(ch);

        match state {
            CaptureState::None => {
                if cursor.head == len && cursor.selection() > 0 {
                    let slice = cursor.slice();
                    cursor.pull();

                    collection.push(Token {
                        data: TokenData::Segment(slice),
                        position: position.clone(),
                    });
                }
            }
            _ => {
                if cursor.selection() > 1 {
                    let slice = cursor.slice_off(1);

                    collection.push(Token {
                        data: TokenData::Segment(Arc::from(slice)),
                        position: position.clone(),
                    });
                }

                cursor.pull();
            }
        }

        match state {
            CaptureState::Newline => position.newline(),

            CaptureState::Symbol => {
                cursor.pull();

                collection.push(Token {
                    data: TokenData::Symbol(ch),
                    position: position.clone(),
                });
            }

            //TODO: add a cursor struct to contain head, tail, column, line vars
            CaptureState::Regex => capture_regex(data, &mut chars, &mut cursor, &mut position, &mut collection),
            CaptureState::String => capture_string(data, &mut chars, &mut cursor, &mut position, &mut collection),
            CaptureState::Variable => capture_variable(data, &mut chars, &mut cursor, &mut position, &mut collection),
            CaptureState::Number => capture_number(data, &mut chars, &mut cursor, &mut position, &mut collection),

            _ => unreachable!("Invalid state"),
        }
    }

    return collection;
}


fn capture_number(data: &str, chars: &mut Chars, cursor: &mut Cursor, position: &mut Position, collector: &mut Collection<Token>) {
    let mut ranged = true;
    let mut mantissa = false;

    for ch in chars.by_ref() {
        position.newline();

        if (!ch.is_numeric()) {
            //TODO: impl
        }
    }

    let slice = cursor.slice_off(1);
    cursor.pull();

    collector.push(Token {
        data: TokenData::Variable(slice),
        position: position.clone(),
    });
}

fn capture_variable(data: &str, chars: &mut Chars, cursor: &mut Cursor, position: &mut Position, collector: &mut Collection<Token>) {
    for ch in chars.by_ref() {
        cursor.shift();

        if (!ch.is_alphanumeric()) {
            break;
        }
    }

    let slice = cursor.slice_off(1);
    cursor.pull();

    collector.push(Token {
        data: TokenData::Variable(slice),
        position: position.clone(),
    });
}

fn capture_regex(data: &str, chars: &mut Chars, cursor: &mut Cursor, position: &mut Position, collector: &mut Collection<Token>) {
    for ch in chars.by_ref() {
        cursor.shift();

        if ch == '\n' {
            position.newline();
        } else if ch == '\\' {
            cursor.shift();
        } else if ch == '/' {
            break;
        }
    }

    let slice = cursor.slice_off(1);
    cursor.pull();

    collector.push(Token {
        data: TokenData::Regex(slice),
        position: position.clone(),
    });
}

fn capture_string(data: &str, chars: &mut Chars, cursor: &mut Cursor, position: &mut Position, collection: &mut Collection<Token>) {
    let mut expression: StrExpression = Vec::new();

    loop {
        match chars.next() {
            Some(ch) => {
                cursor.shift();

                if ch == '\n' {
                    position.newline();
                } else if ch == '\\' {
                    cursor.shift();
                } else if ch == '$' {
                    if cursor.selection() > 0 {
                        let slice = cursor.slice_off(2);
                        expression.push(StrExpressionItem::Literal(slice));
                    }

                    cursor.pull();
                    read_interpolated_variable(&data, chars, cursor, position, &mut expression);
                } else if ch == '"' {
                    break;
                }
            }

            None => {
                collection.throw(Error {
                    message: str!("Unexpected EOF while capturing string"),
                    position: position.clone(),
                });

                return;
            }
        }
    }

    if cursor.selection() > 1 { //TODO: test if 1 is correct instead of 0
        let slice= cursor.slice_off(1);
        expression.push(StrExpressionItem::Literal(slice));
    }

    cursor.pull();

    collection.push(Token {
        data: TokenData::String(expression),
        position: position.clone(),
    });
}

fn read_interpolated_variable(data: &str, chars: &mut Chars, cursor: &mut Cursor, position: &mut Position, expression: &mut StrExpression) {
    for ch in chars.by_ref() {
        cursor.shift();

        if ch == '\n' {
            position.newline();
        } else if ch == '}' {
            let slice = cursor.slice_off(1);
            expression.push(StrExpressionItem::Variable(slice));
            
            cursor.pull();
            break;
        }
    }
}
