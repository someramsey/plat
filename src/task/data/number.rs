use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(Debug)]
pub enum Number {
    Integer(i32),
    Decimal(f32),
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Integer(n) => write!(f, "Int {}", n),
            Number::Decimal(n) => write!(f, "Dec {}", n),
        }
    }
}