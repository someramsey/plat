use std::iter::Peekable;
use crate::task::position::Position;
use std::slice::Iter;

#[macro_export]
macro_rules! expect {
    ($iter:expr, $pat:pat => $binding:ident) => {{
        let iter:&mut NodeIter<_> = $iter;

        if let Some(next) = iter.next() {
            if let $pat = next.data {
                Ok($binding)
            } else {
                Err(ErrorContext::new(next.position.clone(), ErrorCause::UnexpectedNode))
            }
        } else {
            Err(ErrorContext::new(iter.position.clone(), ErrorCause::EndOfFile))
        }
    }};
}

#[macro_export]
macro_rules! check {
    ($iter:expr, $pat:pat => $binding:ident) => {{
        let iter:&mut NodeIter<_> = $iter;

        if let Some(Node { data: $pat, .. }) = iter.current {
            Some($binding)
        } else {
            None
        }
    }};
}

#[derive(Debug)]
pub struct Node<T> {
    pub data: T,
    pub position: Position,
}

impl<T> Node<T> {
    pub fn new(data: T, position: Position) -> Node<T> {
        Node { data, position }
    }
}

pub struct NodeIter<'a, T> {
    pub position: Position,
    pub current: Option<&'a Node<T>>,
    pub done: bool,
    iter: Iter<'a, Node<T>>
}

impl<'a, T> NodeIter<'a, T> {
    pub fn new(data: &'a Vec<Node<T>>) -> NodeIter<'a, T> {
        let mut iter = data.into_iter();
        let current = iter.next();

        NodeIter { iter, current, position: Position::new(), done: false }
    }

    pub fn next(&mut self) -> Option<&Node<T>> {
        if self.done {
            return None;
        }

        return match self.iter.next() {
            Some(node) => {
                let last = self.current.take();

                self.position = node.position.clone();
                self.current = Some(node);

                last
            }

            None => {
                self.done = true;
                None
            }
        }
    }
}