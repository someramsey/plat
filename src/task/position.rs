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