use crate::task::error::Error;
use crate::task::parsers::context::{get_result, Node, ParseContext};
use crate::task::position::Position;
use crate::task::tokenize::{Token, TokenData};
use std::sync::Arc;

#[derive(Debug)]
pub enum Instruction {
    Copy {
        origin: Arc<str>,
        target: Arc<str>,
    },
    Write {
        value: String,
        pattern: String,
        target: String,
    },
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

pub fn parse_instructions(data: Vec<Token>) -> Result<Vec<Node<Instruction>>, Vec<Error>> {
    let mut iterator = data.into_iter();
    let mut context = ParseContext::new(iterator);

    while !context.done {
        if let Some(Token { data: TokenData::Segment(arc), position, }) = context.next() {
            begin_chain(&mut context, Vec::new(), &arc, position);
        }
    }

    return get_result(context);
}

fn begin_chain(context: &mut ParseContext<Instruction>, chain: Vec<Modifier>, str: &str, position: Position, ) {
    match str {
        "at" => at_modifier(context, chain),
        "to" => to_modifier(context, chain),
        "copy" => copy_command(context, chain),
        _ => context.throw_at(Arc::from(format!("Unknown command '{str}'")), position),
    }
}

fn begin_scope(context: &mut ParseContext<Instruction>, chain: Vec<Modifier>) {
    if !context.expect_symbol('{') {
        return;
    }

    while !context.done {
        if let Some(Token { data, position }) = context.next() {
            match data {
                TokenData::Symbol('}') => break,
                TokenData::Segment(str) => {
                    begin_chain(context, chain.clone(), str.as_ref(), position)
                }
                _ => context.throw_at(Arc::from("Expected command or '}'"), position),
            }
        } else {
            context.throw(Arc::from("Unclosed scope"));
        }
    }
}

fn at_modifier(context: &mut ParseContext<Instruction>, mut chain: Vec<Modifier>) {
    if let Some(str) = context.read_string() {
        chain.push(Modifier::At(str));
    }

    begin_scope(context, chain);
}

fn to_modifier(context: &mut ParseContext<Instruction>, mut chain: Vec<Modifier>) {
    if let Some(str) = context.read_string() {
        chain.push(Modifier::To(str));
    }

    begin_scope(context, chain);
}

fn copy_command(context: &mut ParseContext<Instruction>, chain: Vec<Modifier>) {
    let mut origin: Vec<Arc<str>> = Vec::new();
    let mut target: Vec<Arc<str>> = Vec::new();

    for modifier in chain {
        match modifier {
            Modifier::At(str) => origin.push(str),
            Modifier::To(str) => target.push(str),

            _ => context.throw(Arc::from(
                "Invalid modifier for 'copy', expected 'at' or 'to'",
            )),
        }
    }

    while let Some(Token { data, position }) = context.next() {
        match data {
            TokenData::Symbol(';') => break,
            TokenData::Segment(str) => match str.as_ref() {
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

                _ => context.throw_at(Arc::from(format!("Unknown command '{str}'")), position),
            },
            _ => context.throw_at(Arc::from("Expected command attribute"), position),
        }
    }

    if origin.is_empty() {
        context.throw(Arc::from("Expected at least one origin path"));
    }

    if target.is_empty() {
        context.throw(Arc::from("Expected at least one target path"));
    }

    context.push(Instruction::Copy {
        origin: build_path(origin),
        target: build_path(target),
    });
}

fn build_path(paths: Vec<Arc<str>>) -> Arc<str> {
    Arc::from(
        paths
            .iter()
            .map(|s| s.trim_matches('/'))
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("/"),
    )
}

fn read_string(context: &mut ParseContext<Instruction>, vec: &mut Vec<Arc<str>>) -> bool {
    if let Some(str) = context.read_string() {
        vec.push(str);
        true
    } else {
        false
    }
}
