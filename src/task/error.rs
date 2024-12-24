use crate::task::position::Position;
use std::fmt::{Display, Formatter};

pub enum Error {
    Unexpected {
        expected: String,
        received: String,
        position: Position,
    },
    Invalid {
        received: String,
        position: Position,
    },
    EndOfFile {
        expected: String,
    },
    Other {
        message: String,
        position: Position,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::EndOfFile { expected } => {
                write!(f, "Expected '{}', found end of file", expected)
            }

            Error::Unexpected { expected, received, position } => {
                write!(f, "Expected '{}', found '{}' at {}:{}",
                       expected,
                       received,
                       position.line,
                       position.column
                )
            }

            Error::Other { message, position } => {
                write!(f, "Internal error: '{}' at {}:{}",
                       message,
                       position.line,
                       position.column
                )
            },

            Error::Invalid { received, position } => {
                write!(f, "Invalid: '{}'", received)
            }
        }
    }
}