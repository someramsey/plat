use std::sync::Arc;
use crate::task::data::str::Str;

#[derive(Debug)]
pub enum Number {
    Integer(i32),
    Decimal(f32),
}

impl Number {
    pub fn stringify(&self) -> Str {
        match self {
            Number::Integer(n) => Arc::from(n.to_string()),
            Number::Decimal(n) => Arc::from(n.to_string()),
        }
    }
}