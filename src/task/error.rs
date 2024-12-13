use crate::task::position::Position;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Error {
    pub cause: ErrorCause,
    pub position: Position,
}

#[derive(Debug)]
pub struct ErrorCause {
    pub message: Box<str>,
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    UnexpectedNode,
    InternalError,
    EndOfFile,
}

impl Display for ErrorKind {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::UnexpectedNode => write!(f, "Unexpected node"),
            ErrorKind::InternalError => write!(f, "Internal error"),
            ErrorKind::EndOfFile => write!(f, "End of file"),
        }
    }
}

impl Error {
    pub fn new(message: &str, position: Position, kind: ErrorKind) -> Self {
        Self {
            position,
            cause: ErrorCause {
                kind,
                message: Box::from(message),
            },
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {} at {}:{}",
               self.cause.kind,
               self.cause.message,
               self.position.line,
               self.position.column
        )
    }
}