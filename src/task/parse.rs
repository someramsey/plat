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
    Copy()
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
        conduct_chain(&mut iterator, &mut Vec::new(), &mut errors, token)
    }

    return instructions;
}

fn parse_scope<'a>(iterator: &mut IntoIter<Token<'a>>, chain: &mut Vec<Binding<'a>>, errors: &mut Vec<ParseError>) {
    while let Some(token) = iterator.next() {
        match &token.data {
            TokenData::Symbol(ch) if *ch == '}' => break,
            _ => conduct_chain(iterator, &mut chain.clone(), errors, token),
        }
    }
}

fn conduct_chain<'a>(iterator: &mut IntoIter<Token<'a>>, chain: &mut Vec<Binding<'a>>, errors: &mut Vec<ParseError>, token: Token) {
    match token.data {
        TokenData::Segment(str) => {
            match str {
                "at" => parse_at(iterator, chain, errors, token),
                "to" => parse_to(iterator, chain, errors, token),
                "copy" => parse_copy(iterator, chain, errors, token),

                _ => errors.push(ParseError { message: "Unknown segment", position: token.position })
            }
        }
        _ => errors.push(ParseError { message: "Unexpected token", position: token.position })
    }
}

/*
TODO: only allow one way selections
at "a" {
    copy at "b" to "c";
    //at "b" copy to "c"; //error
}
 */

fn parse_copy<'a>(iterator: &mut IntoIter<Token<'a>>, chain: &mut Vec<Binding<'a>>, errors: &mut Vec<ParseError>, this: Token) {

}

fn parse_at<'a>(iterator: &mut IntoIter<Token<'a>>, chain: &mut Vec<Binding<'a>>, errors: &mut Vec<ParseError>, this: Token) {
    let target_arg = iterator.next();

    //TODO: solve err repetition
    if let Some(target_token) = target_arg {
        match target_token.data {
            TokenData::Symbol(ch) if ch == '{' => parse_scope(iterator, chain, errors),

            TokenData::String(str) => {
                chain.push(Binding::At(str));
            }

            _ => errors.push(ParseError { message: "Expected an extended command or a scope", position: target_token.position })
        }

        let next = iterator.next();

        if let Some(next_token) = next {
            match next_token.data {
                TokenData::Symbol(ch) if ch == ';' => return,
                _ => conduct_chain(iterator, chain, errors, next_token)
            }
        }
    } else {
        errors.push(ParseError { message: "Expected argument", position: this.position });
    }
}

fn parse_to<'a>(iterator: &mut IntoIter<Token<'a>>, chain: &mut Vec<Binding<'a>>, errors: &mut Vec<ParseError>, this: Token) {
    let target_arg = iterator.next();

    //TODO: solve err repetition
    if let Some(target_token) = target_arg {
        match target_token.data {
            TokenData::Symbol(ch) if ch == '{' => parse_scope(iterator, chain, errors),

            TokenData::String(str) => {
                chain.push(Binding::To(str));
            }

            _ => errors.push(ParseError { message: "Expected an extended command or a scope", position: target_token.position })
        }

        let next = iterator.next();

        if let Some(next_token) = next {
            match next_token.data {
                TokenData::Symbol(ch) if ch == ';' => return,
                _ => conduct_chain(iterator, chain, errors, next_token)
            }
        }
    } else {
        errors.push(ParseError { message: "Expected argument", position: this.position });
    }
}