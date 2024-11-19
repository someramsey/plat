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

#[derive(Debug)]
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
                println!("{:?}", token);
                self.pos = token.position.clone();
                Some(token)
            }
            None => {
                self.done = true;
                None
            }
        }
    }

    pub fn err_at(&mut self, message: Arc<str>, position: TokenPosition) {
        self.errors.push(ParseError { message, position });
        self.failed = true;
    }

    pub fn err(&mut self, message: Arc<str>) {
        self.err_at(message, self.pos.clone());
    }

    pub fn expect_symbol(&mut self, symbol: char) -> bool {
        if let Some(Token { data, position }) = self.next() {
            match data {
                TokenData::Symbol(ch) if ch == symbol => return true,

                _ => {
                    let kind = data.kind();
                    self.err_at(Arc::from(format!("Expected '{symbol}', found {kind}")), position)
                }
            }
        } else {
            self.err(Arc::from(format!("Expected '{symbol}'")));
        }

        return false;
    }

    pub fn expect_segment(&mut self, segment: &str) -> bool {
        if let Some(Token { data, position }) = self.next() {
            match data {
                TokenData::Segment(arc) if arc.as_ref() == segment => return true,

                _ => {
                    let kind = data.kind();
                    self.err_at(Arc::from(format!("Expected '{segment}', found {kind}")), position)
                }
            }
        } else {
            self.err(Arc::from(format!("Expected '{segment}'")));
        }

        return false;
    }

    pub fn expect_string(&mut self) -> Option<Arc<str>> {
        if let Some(Token { data, position }) = self.next() {
            match data {
                TokenData::String(str) => return Some(str),

                _ => {
                    let kind = data.kind();
                    self.err_at(Arc::from(format!("Expected string, found {kind}")), position);
                }
            }
        } else {
            self.err(Arc::from("Expected string literal"));
        }

        return None;
    }
}

pub fn parse(data: Vec<Token>) -> Result<Vec<Instruction>, Vec<ParseError>> {
    let mut iterator = data.into_iter();
    let mut context = ParseContext::new(iterator);

    while !context.done {
        if let Some(Token { data: TokenData::Segment(str), position }) = context.next() {
            begin_chain(&mut context, Vec::new(), str.as_ref(), position);
        }
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
        _ => context.err_at(Arc::from(format!("Unknown command '{str}'")), position),
    }
}

fn begin_scope(context: &mut ParseContext, chain: Vec<Modifier>) {
    if !context.expect_symbol('{') {
        return;
    }

    while !context.done {
        if let Some(Token { data, position }) = context.next() {
            match data {
                TokenData::Symbol('}') => break,
                TokenData::Segment(str) => begin_chain(context, chain.clone(), str.as_ref(), position),
                _ => context.err_at(Arc::from("Expected command or '}'"), position),
            }
        } else {
            context.err(Arc::from("Unclosed scope"));
        }
    }
}

fn at_modifier(context: &mut ParseContext, mut chain: Vec<Modifier>) {
    if let Some(str) = context.expect_string() {
        chain.push(Modifier::At(str));
    }

    begin_scope(context, chain);
}

fn to_modifier(context: &mut ParseContext, mut chain: Vec<Modifier>) {
    if let Some(str) = context.expect_string() {
        chain.push(Modifier::To(str));
    }

    begin_scope(context, chain);
}

fn copy_command(context: &mut ParseContext, chain: Vec<Modifier>) {
    let mut origin: Vec<Arc<str>> = Vec::new();
    let mut target: Vec<Arc<str>> = Vec::new();

    for modifier in chain {
        match modifier {
            Modifier::At(str) => origin.push(str),
            Modifier::To(str) => target.push(str),

            _ => context.err(Arc::from("Invalid modifier for 'copy', expected 'at' or 'to'")),
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

                    _ => context.err_at(Arc::from(format!("Unknown command '{str}'")), position),
                }
            }
            _ => context.err_at(Arc::from("Expected command attribute"), position),
        }
    }

    if origin.is_empty() {
        context.err(Arc::from("Expected at least one origin path"));
    }

    if target.is_empty() {
        context.err(Arc::from("Expected at least one target path"));
    }

    let parsed_origin = match build_path(origin) {
        Ok(paths) => paths,
        Err(err) => return context.err(err)
    };

    let parsed_target = match build_path(target) {
        Ok(paths) => paths,
        Err(err) => return context.err(err)
    };

    context.instructions.push(Instruction::Copy { origin: parsed_origin, target: parsed_target });
}

fn build_path(paths: Vec<Arc<str>>) -> Result<Vec<PathBuf>, Arc<str>> {
    let joined = paths.iter()
        .map(|s| s.trim_matches('/'))
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("/");

    let mut result = Vec::new();

    //TODO: Glob gives results relative to cwd unless it has an absolute path
    for entry in glob(&joined).map_err(|e| e.to_string())? {
        match entry {
            Ok(path) => result.push(path),
            Err(e) => return Err(Arc::from(e.to_string())),
        }
    }

    Ok(result)
}

fn read_string(context: &mut ParseContext, vec: &mut Vec<Arc<str>>) -> bool {
    if let Some(str) = context.expect_string() {
        vec.push(str);
        true
    } else {
        false
    }
}
