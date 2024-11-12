use std::ops::Add;
use std::path::Path;
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
pub enum Binding<'a> {
    At(&'a str),
    To(&'a str),
}

pub struct ParseError<'a> {
    pub message: &'a str,
    pub position: TokenPosition,
}

pub fn parse(data: Vec<Token>) -> Vec<Instruction> {
    let mut iterator = data.into_iter();
    let mut instructions: Vec<Instruction> = Vec::new();
    let mut errors: Vec<ParseError> = Vec::new();
    let mut chain: Vec<Binding> = Vec::new();

    parse_scope(&mut iterator, &mut chain, &mut errors);

    return instructions;
}

fn parse_base(iterator: &mut IntoIter<Token>, chain: &mut Vec<Binding>, errors: &mut Vec<ParseError>, token: Token) {
    match &token.data {
        TokenData::Segment(str) => {
            match str {
                &"at" => {
                    parse_at(iterator, chain, errors, token);
                }
                _ => errors.push(ParseError { message: "Unknown segment", position: token.position })
            }
        }
        _ => errors.push(ParseError { message: "Unexpected token", position: token.position })
    }
}

fn parse_scope(iterator: &mut IntoIter<Token>, chain: &mut Vec<Binding>, errors: &mut Vec<ParseError>) {
    while let Some(token) = iterator.next() {
        match &token.data {
            TokenData::Symbol(ch) if *ch == '}' => break,
            _ => parse_base(iterator, chain, errors, token),
        }
    }
}

fn parse_at(iterator: &mut IntoIter<Token>, chain: &mut Vec<Binding>, errors: &mut Vec<ParseError>, this: Token) {
    let target_arg = iterator.next();

    if let Some(target_token) = target_arg {
        let Token { data, position } = target_token;

        match data {
            TokenData::Symbol(ch) => {
                // parse_scope(iterator, chain, errors);
            }

            TokenData::String(str) => {
                chain.push(Binding::At(str));
            }

            _ => errors.push(ParseError { message: "Expected string or symbol", position }),
        }

        return;
    }

    errors.push(ParseError { message: "Expected argument", position: this.position });
}