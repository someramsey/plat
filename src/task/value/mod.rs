use crate::task::value::number::NumberValue;
use crate::task::value::string::StringExpression;
use std::fmt::{Display, Formatter};
use crate::task::value::range::RangeValue;

pub mod number;
pub mod string;
pub mod range;

#[derive(Debug)]
pub enum Value {
    Regex(Box<str>),
    Range(RangeValue),
    String(StringExpression),
    Number(NumberValue),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(num) => write!(f, "{}", num),
            Value::String(string) => write!(f, "{}", string),
            Value::Range(range) => write!(f, "{}", range),
            Value::Regex(regex) => write!(f, "regex /{}/", regex),
        }
    }
}