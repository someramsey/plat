use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(Debug)]
pub enum NumberValue {
    Integer(i32),
    Decimal(f32),
}

impl Display for NumberValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberValue::Integer(n) => write!(f, "Int {}", n),
            NumberValue::Decimal(n) => write!(f, "Dec {}", n),
        }
    }
}