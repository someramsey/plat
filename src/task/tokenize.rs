use std::fs;
use std::path::PathBuf;

pub enum Token<'a> {
    Symbol(char),
    String(&'a str),
    Segment(&'a str),
}

enum CaptureState { Symbol, Newline, WhiteSpace, String, None }

fn capture(ch: char) -> CaptureState {
    match ch {
        '"' => CaptureState::String,
        '{' | '}' => CaptureState::Symbol,
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

        if head == len {
            if head - tail > 0 {
                let word = &data[tail..head];
                tail = head;

                tokens.push(Token::Segment(word));
            }
        } else if !matches!(state, CaptureState::None) {
            if head - tail > 1 {
                let word = &data[tail..head - 1];
                tail = head;

                tokens.push(Token::Segment(word));
            }

            tail = head;
        }

        match state {
            CaptureState::Newline => {
                line += 1;
                column = 0;
            }

            CaptureState::Symbol => {
                tail = head;
                tokens.push(Token::Symbol(ch));
            }

            CaptureState::String => {
                for ch in chars.by_ref() {
                    head += 1;
                    column += 1;

                    if ch == '"' {
                        break;
                    }
                }

                tokens.push(Token::String(&data[tail..head - 1]));
                tail = head;
            }

            _ => {}
        }
    }

    return tokens;
}

