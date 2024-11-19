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

enum Modifier {
    At(Arc<str>),
    To(Arc<str>),
}

impl Clone for Modifier {
    fn clone(&self) -> Self {
        match self {
            Modifier::At(arc) => Modifier::At(arc.clone()),
            Modifier::To(arc) => Modifier::To(arc.clone()),
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

    pub fn expect_symbol(&mut self, symbol: char) -> bool {
        if let Some(Token { data, position }) = self.next() {
            match data {
                TokenData::Symbol(ch) if ch == symbol => return true,

                _ => {
                    let kind = data.kind();
                    self.err(Arc::from(format!("Expected '{symbol}', found {kind}")), position)
                }
            }
        } else {
            self.err(Arc::from(format!("Expected '{symbol}'")), self.pos.clone());
        }

        return false;
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
        parse_command(&mut context);
    }

    if context.failed {
        Err(context.errors)
    } else {
        Ok(context.instructions)
    }
}

fn begin_chain(context: &mut ParseContext, chain: Vec<Modifier>, str: &str, position: TokenPosition) {
    match str {
        "at" => at_modifier(context, chain),
        "to" => to_modifier(context, chain),
        "copy" => copy_command(context, chain),
        _ => context.err(Arc::from(format!("Unknown command '{str}'")), position),
    }
}

fn parse_command(context: &mut ParseContext) {
    if let Some(Token { data: TokenData::String(str), position }) = context.next() {
        begin_chain(context, Vec::new(), str.as_ref(), position);
    } else {
        context.err(Arc::from("Expected command"), context.pos.clone());
    }
}

fn parse_scope(context: &mut ParseContext, chain: Vec<Modifier>) {
    if !context.expect_symbol('{') {
        return;
    }

    while !context.done {
        if let Some(Token { data, position }) = context.next() {
            match data {
                TokenData::Symbol('}') => break,
                TokenData::String(str) => begin_chain(context, chain.clone(), str.as_ref(), position),
                _ => context.err(Arc::from("Expected command or '}'"), position),
            }
        } else {
            context.err(Arc::from("Unclosed scope"), context.pos.clone());
        }
    }
}

fn at_modifier(context: &mut ParseContext, mut chain: Vec<Modifier>) {
    if let Some(str) = context.expect_string() {
        chain.push(Modifier::At(str));
    }

    parse_scope(context, chain);
}

fn to_modifier(context: &mut ParseContext, mut chain: Vec<Modifier>) {
    if let Some(str) = context.expect_string() {
        chain.push(Modifier::To(str));
    }

    parse_scope(context, chain);
}

fn copy_command(context: &mut ParseContext, chain: Vec<Modifier>) {
    let mut origin: Vec<Arc<str>> = Vec::new();
    let mut target: Vec<Arc<str>> = Vec::new();

    for modifier in chain {
        match modifier {
            Modifier::At(str) => origin.push(str),
            Modifier::To(str) => target.push(str),

            _ => context.err(Arc::from("Invalid modifier for 'copy', expected 'at' or 'to'"), context.pos.clone()),
        }
    }

    while let Some(Token { data, position }) = context.next() {
        match data {
            TokenData::Symbol(';') => break,
            TokenData::Segment(str) => {
                match str.as_ref() {
                    "at" => {
                        if !read_string(context, &mut origin) {
                            break;
                        }
                    }
                    "to" => {
                        if !read_string(context, &mut target) {
                            break;
                        }
                    }

                    _ => context.err(Arc::from(format!("Unknown command '{str}'")), position),
                }
            }
            _ => context.err(Arc::from("Expected command attribute"), position),
        }
    }

    if origin.is_empty() {
        context.err(Arc::from("Expected at least one origin path"), context.pos.clone());
    }

    if target.is_empty() {
        context.err(Arc::from("Expected at least one target path"), context.pos.clone());
    }

    // context.instructions.push(Instruction::Copy { origin, target });
}

fn read_string(context: &mut ParseContext, vec: &mut Vec<Arc<str>>) -> bool {
    if let Some(str) = context.expect_string() {
        vec.push(str);
        true
    } else {
        false
    }
}
