use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct Range {
    begin: i32,
    end: i32,
}

impl Range {
    pub fn new(begin: i32, end: i32) -> Self {
        return Range {
            begin,
            end,
        };
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.begin, self.end)
    }
}