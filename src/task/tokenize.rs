use std::sync::Arc;

pub struct TokenPosition {
    pub line: i32,
    pub column: i32,
}

impl Clone for TokenPosition {
    fn clone(&self) -> Self {
        return TokenPosition { line: self.line, column: self.column };
    }
}

pub enum TokenData {
    Symbol(char),
    String(Arc<str>),
    Segment(Arc<str>),
}

impl TokenData {
    pub fn kind(&self) -> String {
        match self {
            TokenData::Segment(str) => str.to_string(),
            TokenData::String(str) => format!("\"{}\"", str),
            TokenData::Symbol(ch) => format!("symbol '{}'", ch),
        }
    }
}

pub struct Token {
    pub data: TokenData,
    pub position: TokenPosition,
}

enum CaptureState { Symbol, Newline, WhiteSpace, String, None }

fn capture(ch: char) -> CaptureState {
    match ch {
        '"' => CaptureState::String,
        '{' | '}' | ';' => CaptureState::Symbol,
        '\n' => CaptureState::Newline,
        _ => {
            if ch.is_whitespace() {
                CaptureState::WhiteSpace
            } else {
                CaptureState::None
            }
        }
    }
}

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

        if (head == len && head - tail > 0) || (!matches!(state, CaptureState::None) && head - tail > 1) {
            let segment = &data[tail..head];
            tail = head;

            tokens.push(Token {
                data: TokenData::Segment(Arc::from(segment)),
                position: TokenPosition { line, column },
            });

            tail = head;
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
                    position: TokenPosition { line, column },
                });
            }

            CaptureState::String => {
                for ch in chars.by_ref() {
                    head += 1;
                    column += 1;

                    if ch == '"' {
                        break;
                    }
                }

                let slice = &data[tail..head - 1];

                tokens.push(Token {
                    data: TokenData::String(Arc::from(slice)),
                    position: TokenPosition { line, column },
                });
                tail = head;
            }

            _ => {}
        }
    }

    return tokens;
}

