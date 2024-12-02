use crate::task::data::str::Str;
use crate::task::position::Position;
use std::sync::Arc;

#[derive(Debug)]
pub struct Error {
    pub message: Str,
    pub context: ErrorContext
}

#[derive(Debug)]
pub struct ErrorContext {
    pub position: Position,
    pub cause: ErrorCause,
}

#[derive(Debug)]
pub enum ErrorCause {
    UnexpectedNode,
    EndOfFile,
}

impl ErrorCause {
    pub fn stringify(&self) -> &str {
        match self {
            ErrorCause::UnexpectedNode => "Unexpected node",
            ErrorCause::EndOfFile => "End of file",
        }
    }
}

impl ErrorContext {
    pub fn new(position: Position, cause: ErrorCause) -> Self {
        Self { position, cause }
    }
}

impl Error {
    pub fn new(message: &str, position: Position, cause: ErrorCause) -> Self {
        Self {
            message: Arc::from(message),
            context: ErrorContext::new(position, cause),
        }
    }

    pub fn with_context(message: &str, context: ErrorContext) -> Self {
        Self {
            message: Arc::from(message),
            context,
        }
    }

    pub fn stringify(&self) -> String {
        format!(
            "[{}] {} at {}:{}",
            self.context.cause.stringify(),
            self.message, self.context.position.line, self.context.position.column
        )
    }
}