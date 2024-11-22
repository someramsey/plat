#[derive(Debug)]
pub struct Position {
    pub line: i32,
    pub column: i32,
}

impl Clone for Position {
    fn clone(&self) -> Self {
        return Position { line: self.line, column: self.column };
    }
}