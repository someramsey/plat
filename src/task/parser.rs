use crate::task::tokenizer::{Token, Tokenizer};

pub enum TaskCommand<'a> {
    Say(&'a str)
}

pub fn parse_command<'a>(tokenizer: &'a mut Tokenizer<'_>) -> Result<TaskCommand<'a>, String> {
    let base = tokenizer.next()
        .ok_or("Expected command")?;

    if let Token::Segment(base) = base {
        match base {
            "say" => {
                let arg_token = tokenizer.next()
                    .ok_or("Expected argument")?;

                return if let Token::String(arg) = arg_token {
                    Ok(TaskCommand::Say(arg))
                } else {
                    Err("Expected string argument".to_string())
                };
            }

            _ => Err(format!("Unknown command: {}", base)),
        }
    } else {
        Err("Expected command segment".to_string())
    }
}