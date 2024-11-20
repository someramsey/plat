use crate::task::parse::{parse, Instruction};
use crate::task::tokenize::tokenize;

pub fn run_task(data: &str) {
    let tokens = tokenize(data);
    let instructions = parse(tokens);

    if let Ok(commands) = instructions {
        for command in commands {
            match command {
                Instruction::Copy { origin, target } => {
                    println!("Copy {} to {}", origin, target);
                }
                _ => {}
            }
        }
    }


    // println!("{:?}", instructions);
}