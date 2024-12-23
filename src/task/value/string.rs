use std::fmt::{write, Display, Formatter};
use std::str::{from_utf8, Utf8Error};
use std::sync::Arc;

#[derive(Debug)]
pub struct StringExpression {
    parts: Vec<StringExpressionPart>
}

#[derive(Debug, Clone)]
pub enum StringExpressionPart {
    Literal(Box<str>),
    Variable(Box<str>),
}

impl Display for StringExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for part in &self.parts {
            match part {
                StringExpressionPart::Literal(str) => write!(f, "{}", str),
                StringExpressionPart::Variable(str) => write!(f, "${}", str),
            }?;
        }

        Ok(())
    }
}

impl StringExpression {
    pub fn new(parts: Vec<StringExpressionPart>) -> Self {
        Self { parts }
    }
}

pub fn ch_to_box_str(ch: char) -> Result<Box<str>, Utf8Error> {
    let mut buffer = [0; 4];

    match from_utf8(&buffer) {
        Ok(str) => Ok(Box::from(str)),
        Err(err) => Err(err)
    }
}
