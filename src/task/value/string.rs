use std::fmt::{write, Display, Formatter};
use std::str::{from_utf8, Utf8Error};
use std::sync::Arc;
use crate::task::layers::parsers::commands::StringSource;

#[derive(Debug, Clone)]
pub struct StringExpression {
    parts: Vec<StringExpressionPart>
}

#[derive(Debug, Clone)]
pub enum StringExpressionPartKind {
    Literal,
    Variable,
}

#[derive(Debug, Clone)]
pub struct StringExpressionPart {
    pub kind: StringExpressionPartKind,
    pub value: String,
}

impl Display for StringExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"")?;

        for part in &self.parts {
            match &part.kind {
                StringExpressionPartKind::Literal => write!(f, "{}", part.value)?,
                StringExpressionPartKind::Variable => write!(f, "${}", part.value)?,
            }
        }

        write!(f, "\"")
    }
}

impl StringExpression {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }
    
    pub fn push(&mut self, part: StringExpressionPart) {
        self.parts.push(part);
    }
}

