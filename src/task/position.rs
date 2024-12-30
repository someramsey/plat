use std::fmt::{Debug, Display, Formatter};

pub struct Position {
    pub line: i32,
    pub column: i32,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl Clone for Position {
    fn clone(&self) -> Self {
        return Position { line: self.line, column: self.column };
    }
}

impl Position {
    pub fn new() -> Self {
        Position { line: 0, column: 0 }
    }
    pub fn newline(&mut self) {
        self.line += 1;
        self.column = 0;
    }
    
    pub fn shift(&mut self) {
        self.column += 1;
    }
}