use std::any::TypeId;
use std::fmt::format;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::ptr::write;
use crate::task::tokenize::{Token, TokenData, TokenPosition};
use std::slice::Iter;
use std::vec::IntoIter;
use glob::glob;

#[derive(Debug)]
pub enum Instruction {
    Copy {
        origin: Vec<PathBuf>,
        target: Vec<PathBuf>,
    },
    Write {
        value: String,
        pattern: String,
        target: String,
    },
}

pub struct ParseError {
    pub message: String,
    pub position: TokenPosition,
}

struct ParseContext<'a> {
    done: bool,
    failed: bool,
    last: TokenPosition,
    iterator: IntoIter<Token<'a>>,
    instructions: Vec<Instruction>,
    errors: Vec<ParseError>,
}

impl<'a> ParseContext<'a> {
    fn new(iterator: IntoIter<Token>) -> ParseContext {
        ParseContext {
            iterator,
            done: false,
            failed: false,
            last: TokenPosition { line: 0, column: 0 },
            instructions: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        match self.iterator.next() {
            Some(token) => {
                self.last = token.position.clone();
                Some(token)
            }
            None => {
                self.done = true;
                None
            }
        }
    }

    pub fn err(&mut self, message: String, position: TokenPosition) {
        self.errors.push(ParseError { message, position });
        self.failed = true;
    }

    pub fn expect_segment(&mut self, segment: &str) -> bool {
        if let Some(Token { data, position }) = self.next() {
            match data {
                TokenData::Segment(str) if str == segment => {
                    return true;
                }

                _ => self.err(format!("Expected '{}', found {}", segment, data.kind()), position),
            }
        } else {
            self.err(format!("Expected '{segment}'"), self.last.clone());
        }

        false
    }

    pub fn expect_string(&mut self) -> Option<&str> {
        if let Some(Token { data, position }) = self.next() {
            match data {
                TokenData::String(str) => {
                    return Some(str);
                }
                _ => self.err(format!("Expected a string literal, found {}", data.kind()), position),
            }
        } else {
            self.err(String::from("Expected a string literal"), self.last.clone());
        }

        None
    }
}

enum Modifier<'a> {
    At(&'a str),
    To(&'a str),
}

impl Clone for Modifier<'_> {
    fn clone(&self) -> Self {
        match self {
            Modifier::At(str) => Modifier::At(str),
            Modifier::To(str) => Modifier::To(str),
        }
    }
}

pub fn parse(data: Vec<Token>) -> Result<Vec<Instruction>, Vec<ParseError>> {
    let mut iterator = data.into_iter();
    let mut context = ParseContext::new(iterator);

    while !context.done {
        begin_command(&mut context);
    }

    if context.failed {
        Err(context.errors)
    } else {
        Ok(context.instructions)
    }
}

fn begin_command(context: &mut ParseContext) {
    if let Some(Token { data: TokenData::String(command), position }) = context.next() {
        match command {
            _ => context.err(format!("Unknown keyword '{}'", command), position)
        }
    }
}