use crate::task::nodes::node::Node;
use crate::task::position::Position;
use peekmore::{PeekMore, PeekMoreIterator};
use std::vec::IntoIter;

pub struct NodeIter<T> {
    pub iter: PeekMoreIterator<IntoIter<Node<T>>>,
    pub position: Position,
    pub done: bool,
}

impl<T> NodeIter<T> {
    pub fn new(vec: Vec<Node<T>>) -> NodeIter<T> {
        NodeIter {
            iter: vec.into_iter().peekmore(),
            position: Position::new(),
            done: false,
        }
    }

    pub fn next_if(&mut self, condition: fn(&Node<T>) -> bool) -> Option<Node<T>> {
        let val = self.iter.next_if(condition);
        self.next_internal(val)
    }

    pub fn next(&mut self) -> Option<Node<T>> {
        let val = self.iter.next();
        self.next_internal(val)
    }

    fn next_internal(&mut self, val: Option<Node<T>>) -> Option<Node<T>> {
        match val {
            Some(node) => {
                self.position = node.position.clone();
                Some(node)
            }
            None => {
                self.done = true;
                None
            }
        }
    }

    pub fn peek(&mut self) -> Option<&Node<T>> {
        self.iter.peek()
    }

    pub fn peek_slice(&mut self, count: usize) -> &[Option<Node<T>>] {
        self.iter.peek_amount(count)
    }

    pub fn skip(&mut self) {
        if let Some(node) = self.iter.next() {
            self.position = node.position.clone();
        }
    }

    pub fn skip_by(&mut self, count: usize) {
        let mut i = 0;

        while i < count - 1 {
            if self.iter.next().is_none() {
                return;
            }

            i += 1;
        }

        if let Some(node) = self.iter.next() {
            self.position = node.position.clone();
        }
    }
}