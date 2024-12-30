use crate::task::position::Position;
use std::fmt::{Display, Formatter};

pub enum Error {
    Unexpected {
        expected: String,
        received: String,
        position: Position,
    },
    Invalid {
        message: String,
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
                write!(f, "Expected '{}', found '{}' at {}",
                       expected,
                       received,
                       position
                )
            }

            Error::Other { message, position } => {
                write!(f, "Internal error: '{}' at {}",
                       message,
                       position
                )
            },

            Error::Invalid { message, received, position } => {
                write!(f, "Invalid ({}): Found '{}' at {}",
                       message,
                       received,
                       position
                )
            }
        }
    }
}