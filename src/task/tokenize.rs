use std::str::Chars;
use std::sync::Arc;
use crate::task::position::Position;

#[derive(Debug)]
pub enum TokenData {
    Symbol(char),
    String(Vec<Arc<str>>),
    Regex(Arc<str>),
    Segment(Arc<str>),
    Variable(Arc<str>),
}

impl TokenData {
    pub fn stringify(&self) -> Arc<str> {
        match self {
            TokenData::Segment(str) => str.clone(),
            TokenData::Regex(str) => Arc::from("regex"),
            TokenData::String(str) => Arc::from(format!("string (\"{}\")", str)),
            TokenData::Symbol(ch) => Arc::from(format!("symbol '{}'", ch)),
            TokenData::Variable(str) => Arc::from(format!("${}", str)),
        }
    }
}

#[derive(Debug)]
pub struct Token {
    pub data: TokenData,
    pub position: Position,
}

enum CaptureState { Symbol, Newline, WhiteSpace, String, Regex, Variable, None }

fn capture(ch: char) -> CaptureState {
    match ch {
        '"' => CaptureState::String,
        '/' => CaptureState::Regex,
        '\n' => CaptureState::Newline,
        '$' => CaptureState::Variable,
        '{' | '}' | ';' | ':' | '|' | '>' => CaptureState::Symbol,
        _ => {
            if ch.is_whitespace() {
                CaptureState::WhiteSpace
            } else {
                CaptureState::None
            }
        }
    }
}

//TODO: refactor to a more function based implementation
pub fn tokenize(data: &str) -> Vec<Token> {
    let len = data.len();
    let mut chars = data.chars();

    let mut head = 0;
    let mut tail = 0;

    let mut column = 0;
    let mut line = 0;

    let mut tokens: Vec<Token> = Vec::new();

    while let Some(ch) = chars.next() {
        head += 1;
        column += 1;

        let state = capture(ch);

        match state {
            CaptureState::None => {
                if head == len && head - tail > 0 {
                    let slice = &data[tail..head];
                    tail = head;

                    tokens.push(Token {
                        data: TokenData::Segment(Arc::from(slice)),
                        position: Position { line, column },
                    });
                }
            }
            _ => {
                if head - tail > 1 {
                    let slice = &data[tail..head - 1];

                    tokens.push(Token {
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
                tokens.push(Token {
                    data: TokenData::Symbol(ch),
                    position: Position { line, column },
                });
            }

            CaptureState::Regex => read_regex(&data, &mut chars, &mut head, &mut tail, &mut column, &mut line, &mut tokens),
            CaptureState::String => read_string(&data, &mut chars, &mut head, &mut tail, &mut column, &mut line, &mut tokens),
            CaptureState::Variable => read_variable(&data, &mut chars, &mut head, &mut tail, &mut column, &mut line, &mut tokens),

            _ => unreachable!("Invalid state"),
        }
    }

    return tokens;
}

fn read_variable(data: &str, chars: &mut Chars, head: &mut usize, tail: &mut usize, column: &mut i32, line: &mut i32, tokens: &mut Vec<Token>) {
    while let Some(ch) = chars.next() {
        *line += 1;
        *column = 0;

        if (!ch.is_alphanumeric()) {
            break;
        }
    }

    let slice = Arc::from(&data[*tail..*head - 1usize]);
    *tail = *head;

    tokens.push(Token {
        data: TokenData::Variable(slice),
        position: Position { line: *line, column: *column },
    });
}

fn read_regex(data: &str, chars: &mut Chars, head: &mut usize, tail: &mut usize, column: &mut i32, line: &mut i32, tokens: &mut Vec<Token>) {
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

    tokens.push(Token {
        data: TokenData::Regex(slice),
        position: Position { line: *line, column: *column },
    });
}

fn read_string(data: &str, chars: &mut Chars, head: &mut usize, tail: &mut usize, column: &mut i32, line: &mut i32, tokens: &mut Vec<Token>) {
    let mut parts: Vec<Arc<str>> = Vec::new();

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
                    parts.push(Arc::from(&data[*tail..*head - 2usize]));
                    *tail = *head;

                    read_interpolated_variable(&data, chars, head, tail, column, line, &mut parts);
                }
            }
        } else if ch == '"' {
            break;
        }
    }

    if *head - *tail > 1 {
        parts.push(Arc::from(&data[*tail..*head - 1usize]));
    }

    *tail = *head;

    tokens.push(Token {
        data: TokenData::String(parts),
        position: Position { line: *line, column: *column },
    });
}

fn read_interpolated_variable(data: &str, chars: &mut Chars, head: &mut usize, tail: &mut usize, column: &mut i32, line: &mut i32, parts: &mut Vec<Arc<str>>) {
    while let Some(ch) = chars.next() {
        *head += 1;
        *column += 1;

        if ch == '\n' {
            *line += 1;
            *column = 0;
        } else if ch == '}' {
            parts.push(Arc::from(&data[*tail..*head - 1usize]));
            *tail = *head;
            break;
        }
    }
}
