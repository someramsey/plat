use std::sync::Arc;
use glob::glob;
use crate::task::error::Error;
use crate::task::parse::{parse, Instruction};
use crate::task::tokenize::tokenize;

pub fn evaluate(instructions: Vec<Instruction>) -> Option<Vec<Error>> {
    let mut errors: Vec<Error> = Vec::new();

    if let Ok(commands) = instructions {
        for command in commands {
            match command {
                Instruction::Copy { origin, target } => copy(origin, target, &mut errors),
                _ => {}
            }
        }
    }

    if errors.is_empty() {
        None
    } else {
        Some(errors)
    }
}

fn copy(origin: Arc<str>, target: Arc<str>, errors: &mut Vec<Error>) {

}