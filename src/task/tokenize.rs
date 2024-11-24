use std::str::Chars;
use std::sync::Arc;
use crate::task::position::Position;

pub type Str = Arc<str>;
pub type StrExpression = Vec<Str>;

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
            TokenData::Regex(str) => Arc::from(format!("regex (\"{}\")", str)),
            TokenData::String(str) => Arc::from(format!("string (\"{}\")", str)),
            TokenData::Symbol(ch) => Arc::from(format!("symbol '{}'", ch)),
            TokenData::Variable(str) => Arc::from(format!("${}", str)),
            TokenData::Number(num) => num.stringify(),
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

//TODO: refactor to a more function based implementation
//TODO: add context obj and return a result to allow tokenization errors
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

            CaptureState::Regex => capture_regex(&data, &mut chars, &mut head, &mut tail, &mut column, &mut line, &mut tokens),
            CaptureState::String => capture_string(&data, &mut chars, &mut head, &mut tail, &mut column, &mut line, &mut tokens),
            CaptureState::Variable => capture_variable(&data, &mut chars, &mut head, &mut tail, &mut column, &mut line, &mut tokens),
            CaptureState::Number => capture_number(&data, &mut chars, &mut head, &mut tail, &mut column, &mut line, &mut tokens),

            _ => unreachable!("Invalid state"),
        }
    }

    return tokens;
}


fn capture_number(data: &str, chars: &mut Chars, head: &mut usize, tail: &mut usize, column: &mut i32, line: &mut i32, tokens: &mut Vec<Token>) {
    let mut ranged = true;
    let mut mantissa = false;

    while let Some(ch) = chars.next() {
        *line += 1;
        *column = 0;



        if (!ch.is_numeric()) {
            if ch == '.' {

            } else {
                break;
            }
        }
    }

    let slice = Arc::from(&data[*tail..*head - 1usize]);
    *tail = *head;

    tokens.push(Token {
        data: TokenData::Variable(slice),
        position: Position { line: *line, column: *column },
    });
}

fn capture_variable(data: &str, chars: &mut Chars, head: &mut usize, tail: &mut usize, column: &mut i32, line: &mut i32, tokens: &mut Vec<Token>) {
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

fn capture_regex(data: &str, chars: &mut Chars, head: &mut usize, tail: &mut usize, column: &mut i32, line: &mut i32, tokens: &mut Vec<Token>) {
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

fn capture_string(data: &str, chars: &mut Chars, head: &mut usize, tail: &mut usize, column: &mut i32, line: &mut i32, tokens: &mut Vec<Token>) {
    let mut parts: Vec<Str> = Vec::new();

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

fn read_interpolated_variable(data: &str, chars: &mut Chars, head: &mut usize, tail: &mut usize, column: &mut i32, line: &mut i32, parts: &mut Vec<Str>) {
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
