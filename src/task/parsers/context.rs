use crate::task::error::Error;
use crate::task::position::Position;
use crate::task::tokenize::{Str, StrExpression, Token, TokenData};
use std::sync::Arc;
use std::vec::IntoIter;

pub struct ParseContext<T> {
    pub done: bool,
    pub failed: bool,
    pub pos: Position,
    pub iterator: IntoIter<Token>,
    pub nodes: Vec<Node<T>>,
    pub errors: Vec<Error>,
}

pub struct Node<T> {
    pub data: T,
    pub position: Position,
}

pub fn infer_result<T>(context: ParseContext<T>) -> Result<Vec<Node<T>>, Vec<Error>> {
    if context.failed {
        Err(context.errors)
    } else {
        Ok(context.nodes)
    }
}

pub type ParseResult<T> = Result<Vec<Node<T>>, Vec<Error>>;

impl<T> ParseContext<T> {
    pub fn new(iterator: IntoIter<Token>) -> ParseContext<T> {
        ParseContext {
            iterator,
            done: false,
            failed: false,
            pos: Position { line: 0, column: 0 },
            nodes: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        match self.iterator.next() {
            Some(token) => {
                self.pos = token.position.clone();
                Some(token)
            }
            None => {
                self.done = true;
                None
            }
        }
    }

    pub fn push(&mut self, data: T) {
        self.nodes.push(Node { data, position: self.pos.clone() });
    }

    pub fn throw(&mut self, message: Str) {
        self.throw_at(message, self.pos.clone());
    }

    pub fn throw_at(&mut self, message: Str, position: Position) {
        self.errors.push(Error { message, position });
        self.failed = true;
    }

    pub fn expect_symbol(&mut self, symbol: char) -> bool {
        if let Some(Token { data, position }) = self.next() {
            match data {
                TokenData::Symbol(ch) if ch == symbol => return true,

                _ => {
                    let kind = data.stringify();
                    self.throw_at(
                        Arc::from(format!("Expected '{symbol}', found {kind}")),
                        position,
                    )
                }
            }
        } else {
            self.throw(Arc::from(format!("Expected '{symbol}'")));
        }

        return false;
    }

    pub fn expect_segment(&mut self, segment: &str) -> bool {
        if let Some(Token { data, position }) = self.next() {
            match data {
                TokenData::Segment(arc) if arc.as_ref() == segment => return true,

                _ => {
                    let kind = data.stringify();
                    self.throw_at(
                        Arc::from(format!("Expected '{segment}', found {kind}")),
                        position,
                    )
                }
            }
        } else {
            self.throw(Arc::from(format!("Expected '{segment}'")));
        }

        return false;
    }

    pub fn read_segment(&mut self) -> Option<Str> {
        if let Some(Token { data, position }) = self.next() {
            match data {
                TokenData::Segment(str) => return Some(str),

                _ => {
                    let kind = data.stringify();
                    self.throw_at(
                        Arc::from(format!("Expected segment, found {kind}")),
                        position,
                    );
                }
            }
        } else {
            self.throw(Arc::from("Expected segment"));
        }

        return None;
    }

    pub fn read_string(&mut self) -> Option<StrExpression> {
        if let Some(Token { data, position }) = self.next() {
            match data {
                TokenData::String(str) => return Some(str),

                _ => {
                    let kind = data.stringify();
                    self.throw_at(
                        Arc::from(format!("Expected string, found {kind}")),
                        position,
                    );
                }
            }
        } else {
            self.throw(Arc::from("Expected string literal"));
        }

        return None;
    }

    pub fn result(self) -> ParseResult<T> {
        if self.failed {
            Err(self.errors)
        } else {
            Ok(self.nodes)
        }
    }
}