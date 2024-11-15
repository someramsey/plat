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
impl ParseError {
    pub fn new(message: String, token: Token) -> ParseError {
        return ParseError { message, position: token.position };
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

//TODO: use Result in some functions instead of directly appending to error collection

pub fn parse(data: Vec<Token>) -> Vec<Instruction> {
    let mut iterator = data.into_iter();
    let mut instructions: Vec<Instruction> = Vec::new();
    let mut errors: Vec<ParseError> = Vec::new();

    while let Some(token) = iterator.next() {
        begin_command(token, &mut iterator, &mut instructions, &mut Vec::new(), &mut errors);
    }

    println!("{:?}", &instructions);

    return instructions;
}

fn begin_command<'a>(token: Token, iterator: &mut IntoIter<Token<'a>>, instruction: &mut Vec<Instruction>, modifiers: &mut Vec<Modifier<'a>>, errors: &mut Vec<ParseError>) {
    match token.data {
        TokenData::Segment(str) => {
            match str {
                "at" => at_modifier(token, iterator, instruction, modifiers, errors),
                "copy" => copy_command(token, iterator, instruction, modifiers, errors),
                _ => errors.push(ParseError::new(format!("Invalid token, '{str}' is not recognized as a valid modifier or command"), token))
            }
        }
        _ => errors.push(ParseError::new(String::from("Unexpected token, expected a command or modifier"), token))
    }
}

fn at_modifier<'a>(token: Token, iterator: &mut IntoIter<Token<'a>>, instructions: &mut Vec<Instruction>, modifiers: &mut Vec<Modifier<'a>>, errors: &mut Vec<ParseError>) {
    match iterator.next() {
        Some(token) => {
            match token.data {
                TokenData::String(str) => modifiers.push(Modifier::At(str)),
                _ => {
                    errors.push(ParseError::new(format!("Unexpected token, expected string literal, found {}", token.data.stringify()), token));
                    return;
                }
            }
        }

        None => {
            errors.push(ParseError::new(String::from("Unexpected end of input, expected string literal"), token));
            return;
        }
    }

    match iterator.next() {
        Some(token) => {
            match token.data {
                TokenData::Symbol(ch) => {
                    if ch != '{' {
                        errors.push(ParseError::new(format!("Unexpected token, expected '{{', found {}", ch), token));
                        return;
                    }
                }
                _ => {
                    errors.push(ParseError::new(format!("Unexpected token, expected '{{', found {}", token.data.stringify()), token));
                    return;
                }
            }
        }

        None => {
            errors.push(ParseError::new(String::from("Unexpected end of input, expected '{'"), token));
            return;
        }
    }

    while let Some(token) = iterator.next() {
        match token.data {
            TokenData::Symbol(ch) if ch == '}' => break,
            _ => begin_command(token, iterator, instructions, &mut modifiers.clone(), errors)
        }
    }
}

fn copy_command<'a>(token: Token, iterator: &mut IntoIter<Token<'a>>, instructions: &mut Vec<Instruction>, modifiers: &mut Vec<Modifier<'a>>, errors: &mut Vec<ParseError>) {
    let mut origin: Vec<&str> = Vec::new();
    let mut target: Vec<&str> = Vec::new();

    for modifier in modifiers {
        match modifier {
            Modifier::At(str) => origin.push(str),
            Modifier::To(str) => target.push(str),
            _ => return errors.push(ParseError::new(format!("Invalid command, cannot use 'copy' under this modifier chain, '{}' is not a valid modifier, expected ''at' and 'to' modifiers", modifier.stringify()), token))
        }
    }

    fn collect_attribute<'a>(current: Token, iterator: &mut IntoIter<Token<'a>>, errors: &mut Vec<ParseError>, collector: &mut Vec<&'a str>) {
        match parse_string_literal(iterator, errors) {
            Ok(str) => collector.push(str),
            Err(err) => errors.push(ParseError::new(err, current))
        }
    }

    while let Some(token) = iterator.next() {
        match token.data {
            TokenData::Symbol(str) if str == ';' => break,
            TokenData::Segment(str) => match str {
                "at" => collect_attribute(token, iterator, errors, &mut origin),
                "to" => collect_attribute(token, iterator, errors, &mut target),
                _ => return errors.push(ParseError::new(format!("Unexpected token, expected 'at' or 'to' attributes, found {str}"), token))
            },
            _ => return errors.push(ParseError::new(format!("Unexpected token, expected 'at' or 'to' attributes, found {}", token.data.stringify()), token))
        }
    }

    if origin.is_empty() {
        errors.push(ParseError::new(String::from("Missing 'at' attribute"), token));
        return;
    }

    if target.is_empty() {
        errors.push(ParseError::new(String::from("Missing 'to' attribute"), token));
        return;
    }

    let parsed_origin = match build_path(origin) {
        Ok(paths) => paths,
        Err(err) => {
            errors.push(ParseError::new(err, token));
            return;
        }
    };

    let parsed_target = match build_path(target) {
        Ok(paths) => paths,
        Err(err) => {
            errors.push(ParseError::new(err, token));
            return;
        }
    };

    instructions.push(Instruction::Copy {
        origin: parsed_origin,
        target: parsed_target,
    });
}

fn parse_string_literal<'a>(iterator: &mut IntoIter<Token<'a>>, errors: &mut Vec<ParseError>) -> Result<&'a str, String> {
    return match iterator.next() {
        Some(token) => match token.data {
            TokenData::String(str) => Ok(str),
            _ => Err(format!("Unexpected token, expected string literal, found {}", token.data.stringify()))
        }

        None => Err(String::from("Unexpected end of input, expected string literal"))
    };
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