use std::str::{from_utf8, Utf8Error};

pub type StringExpression = Box<[StringPart]>;

#[derive(Debug, Clone)]
pub enum StringPart {
    Literal(Box<str>),
    Variable(Box<str>),
}

pub fn ch_to_box(ch: char) -> Result<Box<str>, Utf8Error> {
    let mut buffer = [0; 4];

    match from_utf8(&buffer) {
        Ok(str) => Ok(Box::from(str)),
        Err(err) => Err(err)
    }
}