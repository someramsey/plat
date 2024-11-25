use std::str::Chars;
use std::sync::Arc;
use crate::task::collection::Collection;
use crate::task::error::Error;
use crate::task::parsers::context::{Node, ParseContext, ParseResult};
use crate::task::position::Position;

pub type Str = Arc<str>;

#[derive(Debug)]
pub enum StrExpressionItem {
    Literal(Str),
    Variable(Str),
}

impl Clone for StrExpressionItem {
    fn clone(&self) -> Self {
        match self {
            StrExpressionItem::Literal(str) => StrExpressionItem::Literal(str.clone()),
            StrExpressionItem::Variable(str) => StrExpressionItem::Variable(str.clone()),
        }
    }
}

pub type StrExpression = Vec<StrExpressionItem>;

#[derive(Debug)]
pub enum Num {
    Integer(i32),
    Decimal(f32),
}

impl Num {
    pub fn stringify(&self) -> Str {
        match self {
            Num::Integer(n) => Arc::from(n.to_string()),
            Num::Decimal(n) => Arc::from(n.to_string()),
        }
    }
}
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

fn capture(ch: char) -> CaptureState {
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


//TODO: Move collector to its owwn module, reuse its implementation from parsercontexts

//TODO: move data types to their own module
//TODO: refactor to a more function based implementation
//TODO: start throwing errors in places like eof at string capture or reading invalid ranges
pub fn tokenize(data: &str) -> Collection<Token> {
    let len = data.len();
    let mut chars = data.chars();

    let mut head = 0;
    let mut tail = 0;

    let mut column = 0;
    let mut line = 0;

    let mut collection: Collection<Token> = Collection::new();

    while let Some(ch) = chars.next() {
        head += 1;
        column += 1;

        let state = capture(ch);

        match state {
            CaptureState::None => {
                if head == len && head - tail > 0 {
                    let slice = &data[tail..head];
                    tail = head;

                    collection.push(Token {
                        data: TokenData::Segment(Arc::from(slice)),
                        position: Position { line, column },
                    });
                }
            }
            _ => {
                if head - tail > 1 {
                    let slice = &data[tail..head - 1];

                    collection.push(Token {
                        data: TokenData::Segment(Arc::from(slice)),
                        position: Position { line, column },
                    });
                }

                tail = head;
            }
        }

        match state {
            CaptureState::Newline => {
                line += 1;
                column = 0;
            }

            CaptureState::Symbol => {
                tail = head;
                collection.push(Token {
                    data: TokenData::Symbol(ch),
                    position: Position { line, column },
                });
            }

            CaptureState::Regex => capture_regex(&data, &mut chars, &mut head, &mut tail, &mut column, &mut line, &mut collection),
            CaptureState::String => capture_string(&data, &mut chars, &mut head, &mut tail, &mut column, &mut line, &mut collection),
            CaptureState::Variable => capture_variable(&data, &mut chars, &mut head, &mut tail, &mut column, &mut line, &mut collection),
            CaptureState::Number => capture_number(&data, &mut chars, &mut head, &mut tail, &mut column, &mut line, &mut collection),

            _ => unreachable!("Invalid state"),
        }
    }

    return collection;
}


fn capture_number(data: &str, chars: &mut Chars, head: &mut usize, tail: &mut usize, column: &mut i32, line: &mut i32, collector: &mut Collection<Token>) {
    let mut ranged = true;
    let mut mantissa = false;

    while let Some(ch) = chars.next() {
        *line += 1;
        *column = 0;

        if (!ch.is_numeric()) {
            if ch == '.' {} else {
                break;
            }
        }
    }

    let slice = Arc::from(&data[*tail..*head - 1usize]);
    *tail = *head;

    collector.push(Token {
        data: TokenData::Variable(slice),
        position: Position { line: *line, column: *column },
    });
}

fn capture_variable(data: &str, chars: &mut Chars, head: &mut usize, tail: &mut usize, column: &mut i32, line: &mut i32, collector: &mut Collection<Token>) {
    while let Some(ch) = chars.next() {
        *line += 1;
        *column = 0;

        if (!ch.is_alphanumeric()) {
            break;
        }
    }

    let slice = Arc::from(&data[*tail..*head - 1usize]);
    *tail = *head;

    collector.push(Token {
        data: TokenData::Variable(slice),
        position: Position { line: *line, column: *column },
    });
}

fn capture_regex(data: &str, chars: &mut Chars, head: &mut usize, tail: &mut usize, column: &mut i32, line: &mut i32, collector: &mut Collection<Token>) {
    while let Some(ch) = chars.next() {
        *head += 1;
        *column += 1;

        if ch == '\n' {
            *line += 1;
            *column = 0;
        } else if ch == '\\' {
            *head += 1;
            *column += 1;
        } else if ch == '/' {
            break;
        }
    }

    let slice = Arc::from(&data[*tail..*head - 1usize]);
    *tail = *head;

    collector.push(Token {
        data: TokenData::Regex(slice),
        position: Position { line: *line, column: *column },
    });
}

fn capture_string(data: &str, chars: &mut Chars, head: &mut usize, tail: &mut usize, column: &mut i32, line: &mut i32, collector: &mut Collection<Token>) {
    let mut expression: StrExpression = Vec::new();

    while let Some(ch) = chars.next() {
        *head += 1;
        *column += 1;

        if ch == '\n' {
            *line += 1;
            *column = 0;
        } else if ch == '\\' {
            *head += 1;
            *column += 1;
        } else if ch == '$' {
            if let Some(next) = chars.next() {
                *head += 1;
                *column += 1;

                if next == '{' {
                    if *head - *tail > 0 {
                        let slice = Arc::from(&data[*tail..*head - 2usize]);
                        expression.push(StrExpressionItem::Literal(slice));
                    }

                    *tail = *head;
                    read_interpolated_variable(&data, chars, head, tail, column, line, &mut expression);
                }
            }
        } else if ch == '"' {
            break;
        }
    }

    if *head - *tail > 1 { //TODO: test if 1 is correct instead of 0
        let slice = Arc::from(&data[*tail..*head - 1usize]);
        expression.push(StrExpressionItem::Literal(slice));
    }

    *tail = *head;

    collector.push(Token {
        data: TokenData::String(expression),
        position: Position { line: *line, column: *column },
    });
}

fn read_interpolated_variable(data: &str, chars: &mut Chars, head: &mut usize, tail: &mut usize, column: &mut i32, line: &mut i32, expression: &mut StrExpression) {
    while let Some(ch) = chars.next() {
        *head += 1;
        *column += 1;

        if ch == '\n' {
            *line += 1;
            *column = 0;
        } else if ch == '}' {
            let slice = Arc::from(&data[*tail..*head - 1usize]);
            expression.push(StrExpressionItem::Variable(slice));
            *tail = *head;
            break;
        }
    }
}
