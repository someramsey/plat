use crate::task::position::Position;
use std::sync::Arc;
use crate::task::tokenize::Str;

#[derive(Debug)]
pub struct Error {
    pub message: Str,
    pub position: Position,
}

impl Error {
    pub fn new(message: String, position: Position) -> Self {
        Self {
            message: Arc::from(message),
            position,
        }
    }

    pub fn stringify(&self) -> String {
        format!(
            "{} at {}:{}",
            self.message, self.position.line, self.position.column
        )
    }
}
