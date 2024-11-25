use std::sync::Arc;
use crate::task::tokenizer::str::Str;

#[derive(Debug)]
pub enum Num {
    Integer(i32),
    Decimal(f32),
}

impl Num {
    pub fn stringify(&self) -> Str {
        match self {
            Num::Integer(n) => Arc::from(n.to_string()),
            Num::Decimal(n) => Arc::from(n.to_string()),
        }
    }
}