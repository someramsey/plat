use std::fmt::format;
use std::ops::Add;
use std::path::Path;
use std::ptr::write;
use crate::task::tokenize::{Token, TokenData, TokenPosition};
use std::slice::Iter;
use std::vec::IntoIter;

pub enum Instruction {
    Copy {
        origin: String,
        target: String,
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
        begin_command(token, &mut iterator, &mut Vec::new(), &mut errors);
    }

    return instructions;
}

fn begin_command(token: Token, iterator: &mut IntoIter<Token>, modifiers: &mut Vec<Modifier>, errors: &mut Vec<ParseError>) {
    match token.data {
        TokenData::Segment(str) => {
            match str {
                "at" => at_modifier(token, iterator, modifiers, errors),
                "copy" => copy_command(token, iterator, modifiers, errors),
                _ => errors.push(ParseError { message: format!("Invalid token, '{str}' is not recognized as a valid modifier or command"), position: token.position })
            }
        }
        _ => errors.push(ParseError { message: String::from("Unexpected token, expected a command or modifier"), position: token.position })
    }
}

fn at_modifier(token: Token, iterator: &mut IntoIter<Token>, modifiers: &mut Vec<Modifier>, errors: &mut Vec<ParseError>) {
    match iterator.next() {
        Some(token) => {
            if let TokenData::Symbol(ch) = token.data {
                if ch != '{' {
                    errors.push(ParseError { message: format!("Unexpected token, expected '{{', found {}", ch), position: token.position });
                }

                return;
            }

            errors.push(ParseError { message: format!("Unexpected token, expected '{{', found {}", token.stringify()), position: token.position });
        }

        None => errors.push(ParseError { message: String::from("Unexpected end of input, expected '{'"), position: token.position })
    }

    while let Some(token) = iterator.next() {
        match token.data {
            TokenData::Symbol(ch) if ch == '}' => break,
            _ => begin_command(token, iterator, &mut modifiers.clone(), errors)
        }
    }
}

fn copy_command(token: Token, iterator: &mut IntoIter<Token>, modifiers: &mut Vec<Modifier>, errors: &mut Vec<ParseError>) {
    let mut origin: Vec<&str> = Vec::new();
    let mut target: Vec<&str> = Vec::new();

    for modifier in modifiers {
        match modifier {
            Modifier::At(str) => origin.push(str),
            Modifier::To(str) => target.push(str),
            _=> return errors.push(ParseError { message: format!("Invalid command, cannot use 'copy' under this modifier chain, '{}' is not a valid modifier, expected ''at' and 'to' modifiers", modifier.stringify()), position: token.position })
        }
    }

    while let Some(token) = iterator.next() {
        match token.data {
            TokenData::Symbol(str) if str == ';' => break,
            TokenData::Segment(str) => match str {
                "at" => at_attribute(token, iterator, errors, &mut origin),
                _ => return errors.push(ParseError { message: format!("Unexpected token, expected 'at' or 'to' attributes, found {str}"), position: token.position })
            },
            _ => return errors.push(ParseError { message: format!("Unexpected token, expected 'at' or 'to' attributes, found {}", token.stringify()), position: token.position })
        }
    }

    if origin.is_empty() {
        errors.push(ParseError { message: String::from("Missing 'at' attribute"), position: token.position });
        return;
    }

    if target.is_empty() {
        errors.push(ParseError { message: String::from("Missing 'to' attribute"), position: token.position });
        return;
    }

    println!("Origin: {:?}", origin);
}

fn at_attribute(token: Token, iterator: &mut IntoIter<Token>, errors: &mut Vec<ParseError>, origin: &mut Vec<&str>) {
    match iterator.next() {
        Some(token) => {
            if let TokenData::String(str) = iterator.next() {
                origin.push(str);
            } else {
                errors.push(ParseError { message: format!("Unexpected token, expected string literal, found {}", token.stringify()), position: token.position });
            }
        }

        None => errors.push(ParseError { message: String::from("Unexpected end of input, expected string literal"), position: token.position })
    }
}