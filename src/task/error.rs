use crate::task::position::Position;
use std::fmt::{Display, Formatter};

pub enum Error {
    Unexpected {
        expected: Box<str>,
        received: Box<str>,
        position: Position,
    },
    Invalid {
        received: Box<str>,
        position: Position,
    },
    EndOfFile {
        expected: Box<str>,
    },
    Other {
        message: Box<str>,
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