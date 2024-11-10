use crate::task::tokenize::Token;
use std::slice::Iter;

pub enum Instruction {
    Copy {
        origin: String,
        target: String,
    },
    Write {
        value: String,
        pattern: String,
        target: String,
    }
}

fn parse_command(iter: &mut Iter<Token>) -> Result<Instruction, String> {
    let base = iter.next()
        .ok_or("Expected command")?;

    if let Token::Segment(base) = base {
        match base {
            &"at" => {
                let arg_token = iter.next()
                    .ok_or("Expected argument")?;

                let arg = match arg_token {
                    Token::String(arg) => arg,
                    _ => return Err("Expected string argument".to_string()),
                };

                let next = iter.next()
                    .ok_or("Expected target")?;





            }

            _ => Err(format!("Unknown command: {}", base)),
        }
    } else {
        Err("Expected command segment".to_string())
    }
}
pub fn parse(data: Vec<Token>) -> Vec<Instruction> {
    let mut iter = data.iter();
    let mut commands: Vec<Instruction> = Vec::new();

    while let Some(token) = iter.next() {
        match parse_command(&mut iter) {
            Ok(command) =>
                commands.push(command),

            Err(err) =>
                eprintln!("Error parsing command: {}", err),
        }
    }

    return commands;
}