use crate::task::parser::{parse_command, TaskCommand};
use crate::task::tokenizer::Tokenizer;

pub fn run_task(data: &str) {
    let mut tokenizer = Tokenizer::new(data);

    while !tokenizer.ended {
        match parse_command(&mut tokenizer) {
            Ok(command) => {
                match command {
                    TaskCommand::Say(arg) => println!("saying {}", arg),
                }
            }

            Err(err) =>
                eprintln!("Error parsing command at line {} column {}: {}", tokenizer.line, tokenizer.col, err),
        }
    }
}