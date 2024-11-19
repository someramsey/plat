use crate::task::parse::{parse, Instruction};
use crate::task::tokenize::tokenize;

pub fn run_task(data: &str) {
    let tokens = tokenize(data);
    let instructions = parse(tokens);


    // for command in instructions {
    //     match command {
    //         Instruction::Copy(arg) =>
    //             println!("saying: {}", arg),
    //     }
    // }


    println!("{:?}", instructions);
    
    
}