use crate::task::position::Position;
use std::sync::Arc;

#[derive(Debug)]
pub struct Error {
    pub message: Arc<str>,
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
