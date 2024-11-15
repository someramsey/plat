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
    iterator: &'a mut IntoIter<Token<'a>>,
    errors: &'a mut Vec<ParseError>,
    instructions: &'a mut Vec<Instruction>,
    position: TokenPosition
}

impl ParseContext<'_> {
    pub fn new<'a>(iterator: &'a mut IntoIter<Token<'a>>, position: &TokenPosition, instructions: &'a mut Vec<Instruction>, errors: &'a mut Vec<ParseError>) -> ParseContext<'a> {
        return ParseContext { iterator, instructions, errors, position: position.clone() };
    }

    pub fn next(&mut self) -> Option<Token> {
        if let Some(token) = self.iterator.next() {
            self.position = token.position.clone();
            return Some(token);
        }

        return None;
    }

    pub fn error(&mut self, message: String) {
        self.errors.push(ParseError { message, position: self.position.clone() });
    }

    pub fn instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
}

pub enum Modifier<'a> {
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

impl Modifier<'_> {
    pub fn stringify(&self) -> &str {
        match self {
            Modifier::At(str) => "at",
            Modifier::To(str) => "to",
        }
    }
}

pub fn parse(data: Vec<Token>) -> Vec<Instruction> {
    let mut iterator = data.into_iter();
    let mut instructions: Vec<Instruction> = Vec::new();
    let mut errors: Vec<ParseError> = Vec::new();

    while let Some(token) = iterator.next() {
        let mut context = ParseContext::new(&mut iterator, &token.position, &mut instructions, &mut errors);
        begin_command(token, &mut context, &mut Vec::new());
    }

    println!("{:?}", &instructions);

    return instructions;
}

fn begin_command<'a>(token: Token, context: &mut ParseContext<'a>, modifiers: &mut Vec<Modifier<'a>>) {
    match token.data {
        TokenData::Segment(str) => match str {
            "at" => at_modifier(context, modifiers),
            "copy" => copy_command(context, modifiers),
            _ => context.error(format!("Invalid token, '{str}' is not recognized as a valid modifier or command"))
        }

        _ => context.error(String::from("Unexpected token, expected a command or modifier"))
    }
}

fn at_modifier<'a>(context: &mut ParseContext<'a>, modifiers: &mut Vec<Modifier<'a>>) {
    match context.next() {
        Some(token) => {
            match token.data {
                TokenData::String(str) => modifiers.push(Modifier::At(str)),
                _ => return context.error(format!("Unexpected token, expected string literal, found {}", token.data.stringify()))
            }
        }

        None => return context.error(String::from("Unexpected end of input, expected string literal"))
    }

    if let Err(err) = expect_symbol(context, '{') {
        return context.error(err);
    }

    while let Some(token) = context.next() {
        match token.data {
            TokenData::Symbol(ch) if ch == '}' => break,
            _ => begin_command(token, context, modifiers),
        }
    }
}

fn copy_command<'a>(context: &mut ParseContext<'a>, modifiers: &mut Vec<Modifier<'a>>) {
    let mut origin: Vec<&str> = Vec::new();
    let mut target: Vec<&str> = Vec::new();

    for modifier in modifiers {
        match modifier {
            Modifier::At(str) => origin.push(str),
            Modifier::To(str) => target.push(str),
            _ => return context.error(format!("Invalid command, cannot use 'copy' under this modifier chain, '{}' is not a valid modifier, expected ''at' and 'to' modifiers", modifier.stringify()))
        }
    }

    while let Some(token) = context.next() {
        match token.data {
            TokenData::Symbol(str) if str == ';' => break,
            TokenData::Segment(str) => match str {
                "at" => match expect_string(context) {
                    Ok(str) => origin.push(str),
                    Err(err) => return context.error(err)
                },

                "to" => match expect_string(context) {
                    Ok(str) => target.push(str),
                    Err(err) => return context.error(err)
                },

                _ => return context.error(format!("Unexpected token, expected 'at' or 'to' attributes, found {str}"))
            },
            _ => return context.error(format!("Unexpected token, expected 'at' or 'to' attributes, found {}", token.data.stringify()))
        }
    }

    if origin.is_empty() {
        context.error(String::from("Missing 'at' attribute"));
        return;
    }

    if target.is_empty() {
        context.error(String::from("Missing 'to' attribute"));
        return;
    }

    let parsed_origin = match build_path(origin) {
        Ok(paths) => paths,
        Err(err) => return context.error(err)
    };

    let parsed_target = match build_path(target) {
        Ok(paths) => paths,
        Err(err) => return context.error(err)
    };

    context.instruction(Instruction::Copy {
        origin: parsed_origin,
        target: parsed_target,
    });
}

fn build_path(paths: Vec<&str>) -> Result<Vec<PathBuf>, String> {
    let joined = paths.iter()
        .map(|s| s.trim_matches('/'))
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("/");

    let mut result = Vec::new();

    for entry in glob(&joined).map_err(|e| e.to_string())? {
        match entry {
            Ok(path) => result.push(path),
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok(result)
}

fn expect_symbol(context: &mut ParseContext, symbol: char) -> Result<char, String> {
    return match context.next() {
        Some(token) => match token.data {
            TokenData::Symbol(ch) => {
                if ch != symbol {
                    return Err(format!("Unexpected token, expected '{symbol}', found {ch}"));
                }

                return Ok(ch);
            }

            _ => Err(format!("Unexpected token, expected '{symbol}', found {}", token.data.stringify()))
        },

        None => Err(format!("Unexpected end of input, expected '{symbol}'"))
    };
}

fn expect_string<'a>(context: &mut ParseContext<'a>) -> Result<&'a str, String> {
    return match context.next() {
        Some(token) => match token.data {
            TokenData::String(str) => Ok(str),

            _ => Err(format!("Unexpected token, expected string literal, found {}", token.data.stringify()))
        }

        None => Err(String::from("Unexpected end of input, expected string literal"))
    };
}