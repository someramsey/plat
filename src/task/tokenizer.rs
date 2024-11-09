use std::fs;
use std::path::PathBuf;

pub enum Token<'a> {
    Symbol(char),
    String(&'a str),
    Segment(&'a str),
}

pub struct Tokenizer<'a> {
    data: &'a str,
    chars: std::str::Chars<'a>,
    pub len: usize,
    pub tail: usize,
    pub head: usize,
    pub col: usize,
    pub line: usize,
    pub ended: bool,
}

enum CaptureState { Symbol, Newline, WhiteSpace, String, None }

impl<'a> Tokenizer<'a> {
    pub fn new(data: &'a str) -> Self {
        Self {
            data,
            chars: data.chars(),
            len: data.len(),
            tail: 0,
            head: 0,
            col: 0,
            line: 0,
            ended: false,
        }
    }

    fn capture(&mut self, ch: char) -> CaptureState {
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

    pub fn next(&mut self) -> Option<Token> {
        while let Some(ch) = self.chars.next() {
            self.head += 1;
            self.col += 1;

            let state = self.capture(ch);

            if self.head == self.len {
                self.ended = true;

                if self.head - self.tail > 0 {
                    let word = &self.data[self.tail..self.head];
                    self.tail = self.head;

                    return Some(Token::Segment(word));
                }
            } else if !matches!(state, CaptureState::None) {
                if self.head - self.tail > 1 {
                    let word = &self.data[self.tail..self.head - 1];
                    self.tail = self.head;

                    return Some(Token::Segment(word));
                }

                self.tail = self.head;
            }

            match state {
                CaptureState::Newline => {
                    self.line += 1;
                    self.col = 0;
                }

                CaptureState::Symbol => {
                    self.tail = self.head;
                    return Some(Token::Symbol(ch));
                }

                CaptureState::String => {
                    loop {
                        if let Some(ch) = self.chars.next() {
                            self.head += 1;
                            self.col += 1;

                            if ch == '"' {
                                break;
                            }
                        } else {
                            self.ended = true;
                            return None;
                        }
                    }

                    if self.head == self.len {
                        self.ended = true;
                    }

                    let str = Some(Token::String(&self.data[self.tail..self.head - 1]));
                    self.tail = self.head;

                    return str;
                }

                _ => {}
            }
        }

        None
    }
}