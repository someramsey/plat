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

impl Clone for Binding<'_> {
    fn clone(&self) -> Self {
        match self {
            Binding::At(str) => Binding::At(str),
            Binding::To(str) => Binding::To(str),
        }
    }
}

pub struct ParseError<'a> {
    pub message: &'a str,
    pub position: TokenPosition,
}

pub fn parse(data: Vec<Token>) -> Vec<Instruction> {
    let mut iterator = data.into_iter();
    let mut instructions: Vec<Instruction> = Vec::new();
    let mut errors: Vec<ParseError> = Vec::new();

    while let Some(token) = iterator.next() {
        begin_chain(&mut iterator, &mut errors, &mut Vec::new(), token)
    }

    return instructions;
}
fn parse_scope(iterator: &mut IntoIter<Token>, chain: &mut Vec<Binding>, errors: &mut Vec<ParseError>) {
    while let Some(token) = iterator.next() {
        match &token.data {
            TokenData::Symbol(ch) if *ch == '}' => break,

            //find a way to clone chain
            _ => begin_chain(iterator, errors, &mut chain.clone(), token),
        }
    }
}

fn begin_chain(iterator: &mut IntoIter<Token>, errors: &mut Vec<ParseError>, chain: &mut Vec<Binding>, token: Token) {
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


fn parse_at<'a>(iterator: &mut IntoIter<Token<'a>>, chain: &mut Vec<Binding<'a>>, errors: &mut Vec<ParseError>, this: Token) {
    let target_arg = iterator.next();

    if let Some(target_token) = target_arg {
        match target_token.data {
            TokenData::Symbol(ch) => {
                if (ch == '{') {
                    parse_scope(iterator, chain, errors);
                } else {
                    errors.push(ParseError { message: "Unexpected symbol, expected binding or '{'", position: target_token.position });
                }
            }

            TokenData::String(str) => {
                chain.push(Binding::At(str));
            }

            _ => errors.push(ParseError { message: "Expected string or symbol", position: target_token.position }),
        }
    } else {
        errors.push(ParseError { message: "Expected argument", position: this.position });
    }
}