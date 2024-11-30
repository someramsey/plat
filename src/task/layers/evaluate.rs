//TODO: Reimplement after handling str_expressions
use crate::task::error::Error;
use crate::task::layers::parsers::parse_script::Instruction;
use crate::task::position::Position;
use glob::glob;
use std::{fs, net};
use crate::task::node::Node;

pub fn evaluate(nodes: Vec<Node<Instruction>>) -> Option<Vec<Error>> {
    let mut errors: Vec<Error> = Vec::new();

    for node in nodes {
        let Node { data, position } = node;

        match data {
            // Instruction::Copy { origin, target } => copy(&origin, &target, position, &mut errors),
            _ => {}
        }
    }

    if errors.is_empty() {
        None
    } else {
        Some(errors)
    }
}

// fn copy(origin: &str, target: &str, position: Position, errors: &mut Vec<Error>) {
//     let origins = match glob(origin) {
//         Ok(paths) => paths.filter_map(Result::ok).collect::<Vec<_>>(),
//         Err(err) => {
//             errors.push(Error::new(format!("Failed to resolve given origin path: {}", err), position));
//             return;
//         }
//     };
//
//     let targets = match glob(target) {
//         Ok(paths) => paths.filter_map(Result::ok).collect::<Vec<_>>(),
//         Err(err) => {
//             errors.push(Error::new(format!("Failed to resolve given target path: {}", err), position));
//             return;
//         }
//     };
//
//     for origin in origins {
//         for target in &targets {
//             if let Err(err) = fs::copy(&origin, target) {
//                 errors.push(Error::new(format!("Failed to copy file: {}", err), position.clone()));
//             }
//         }
//     }
// }
