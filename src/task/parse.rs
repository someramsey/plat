use std::any::TypeId;
use std::fmt::format;
use std::io::Chain;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::ptr::write;
use crate::task::tokenize::{Token, TokenData, TokenPosition};
use std::slice::Iter;
use std::sync::Arc;
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
    pub message: Arc<str>,
    pub position: TokenPosition,
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

struct ParseContext {
    done: bool,
    failed: bool,
    pos: TokenPosition,
    iterator: IntoIter<Token>,
    instructions: Vec<Instruction>,
    errors: Vec<ParseError>,
}

impl ParseContext {
    fn new(iterator: IntoIter<Token>) -> ParseContext {
        ParseContext {
            iterator,
            done: false,
            failed: false,
            pos: TokenPosition { line: 0, column: 0 },
            instructions: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        match self.iterator.next() {
            Some(token) => {
                self.pos = token.position.clone();
                Some(token)
            }
            None => {
                self.done = true;
                None
            }
        }
    }

    pub fn err(&mut self, message: Arc<str>, position: TokenPosition) {
        self.errors.push(ParseError { message, position });
        self.failed = true;
    }

    pub fn expect_segment(&mut self, segment: &str) -> bool {
        if let Some(Token { data, position }) = self.next() {
            match data {
                TokenData::Segment(arc) if arc.as_ref() == segment => return true,

                _ => {
                    let kind = data.kind();
                    self.err(Arc::from(format!("Expected '{segment}', found {kind}")), position)
                }
            }
        } else {
            self.err(Arc::from(format!("Expected '{segment}'")), self.pos.clone());
        }

        return false;
    }

    pub fn expect_string(&mut self) -> Option<Arc<str>> {
        if let Some(Token { data, position }) = self.next() {
            match data {
                TokenData::String(str) => return Some(str),

                _ => {
                    let kind = data.kind();
                    self.err(Arc::from(format!("Expected string, found {kind}")), position);
                }
            }
        } else {
            self.err(Arc::from("Expected string literal"), self.pos.clone());
        }

        return None;
    }
}

pub fn parse(data: Vec<Token>) -> Result<Vec<Instruction>, Vec<ParseError>> {
    let mut iterator = data.into_iter();
    let mut context = ParseContext::new(iterator);

    while !context.done {
        begin_command(&mut context, Vec::new());
    }

    if context.failed {
        Err(context.errors)
    } else {
        Ok(context.instructions)
    }
}

fn begin_command(context: &mut ParseContext, chain: Vec<Modifier>) -> () {
    if let Some(Token { data: TokenData::String(str), position }) = context.next() {
        match str.as_ref() {
            "at" => at_modifier(context, chain),
            "to" => to_modifier(context, chain),
            _ => context.err(Arc::from(format!("Unknown command '{str}'")), position),
        }
    } else {
        context.err(Arc::from("Expected command"), context.pos.clone());
    }
}

fn at_modifier(context: &mut ParseContext, mut chain: Vec<Modifier>) {
    if let Some(str) = context.expect_string() {
        chain.push(Modifier::At(str.as_ref()));
    }

    begin_command(context, chain);
}

fn to_modifier(context: &mut ParseContext, mut chain: Vec<Modifier>) {
    if let Some(str) = context.expect_string() {
        chain.push(Modifier::To(str.as_ref()));
    }

    begin_command(context, chain);
}