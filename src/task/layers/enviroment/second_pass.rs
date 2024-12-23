use crate::task::value::range::{RangeValue};
use crate::task::value::string::StringExpression;

pub enum FieldData {
    Text(Option<Box<str>>),
    Integer(Option<RangeValue>),
    Decimal(Option<RangeValue>),
    Switch(Box<[StringExpression]>),
}

pub enum Compound<'a> {
    Field {
        identifier: &'a str,
        prompt: &'a str,
        data: FieldData,
    }
}