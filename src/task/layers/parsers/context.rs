use crate::str;
use crate::task::collection::Collection;
use crate::task::error::Error;
use crate::task::position::Position;
use crate::task::layers::tokenize::{Token, TokenData};
use std::sync::Arc;
use std::vec::IntoIter;
use crate::task::data::str::Str;
use crate::task::data::str_expr::StrExpression;

#[derive(Debug)]
pub struct Node<T> {
    pub data: T,
    pub position: Position,
}

pub enum ParseState {
    Waiting,
    Working(Position),
    Done,
}

pub struct ParseContext<T> {
    pub state: ParseState,
    pub collection: Collection<Node<T>>,
    pub iterator: IntoIter<Token>,
}

impl<T> ParseContext<T> {
    pub fn new(iterator: IntoIter<Token>) -> ParseContext<T> {
        ParseContext::<T> {
            iterator,
            state: ParseState::Waiting,
            collection: Collection::<Node<T>>::new(),
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        return match self.iterator.next() {
            Some(token) => {
                self.state = ParseState::Working(token.position.clone());
                Some(token)
            }
            None => {
                self.state = ParseState::Done;
                None
            }
        };
    }

    pub fn push(&mut self, data: T) {
        if let ParseState::Working(pos) = &self.state {
            self.collection.push(Node { data, position: pos.clone() });
        }
    }

    pub fn throw(&mut self, message: Str) {
        if let ParseState::Working(pos) = &self.state {
            self.throw_at(message, pos.clone());
        }
    }

    pub fn throw_at(&mut self, message: Str, position: Position) {
        self.collection.throw(Error { message, position });
    }

    pub fn expect_symbol(&mut self, symbol: char) -> bool {
        if let Some(Token { data, position }) = self.next() {
            match data {
                TokenData::Symbol(ch) if ch == symbol => return true,

                _ => {
                    let kind = data.stringify();
                    self.throw_at(
                        str!("Expected '{symbol}', found {kind}"),
                        position,
                    )
                }
            }
        } else {
            self.throw(str!("Expected '{symbol}'"));
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
                        str!("Expected '{segment}', found {kind}"),
                        position,
                    )
                }
            }
        } else {
            self.throw(str!("Expected '{segment}'"));
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
                        str!("Expected segment, found {kind}"),
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
                        str!("Expected string, found {kind}"),
                        position,
                    );
                }
            }
        } else {
            self.throw(Arc::from("Expected string literal"));
        }

        return None;
    }

    pub fn is_done(&self) -> bool {
        return matches!(self.state, ParseState::Done);
    }
}