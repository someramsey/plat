use crate::task::tokenizer::str::Str;

pub type StrExpression = Vec<StrExpressionItem>;

#[derive(Debug)]
pub enum StrExpressionItem {
    Literal(Str),
    Variable(Str),
}

impl Clone for StrExpressionItem {
    fn clone(&self) -> Self {
        match self {
            StrExpressionItem::Literal(str) => StrExpressionItem::Literal(str.clone()),
            StrExpressionItem::Variable(str) => StrExpressionItem::Variable(str.clone()),
        }
    }
}