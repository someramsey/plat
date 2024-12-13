use crate::task::nodes::node::Node;
use crate::task::position::Position;
use peekmore::{PeekMore, PeekMoreIterator};
use std::vec::IntoIter;

pub struct NodeIter<T> {
    pub iter: PeekMoreIterator<IntoIter<Node<T>>>,
    pub position: Position
}

impl<T> NodeIter<T> {
    pub fn new(vec: Vec<Node<T>>) -> NodeIter<T> {
        NodeIter { iter: vec.into_iter().peekmore(), position: Position::new() }
    }

    pub fn next(&mut self) -> Option<Node<T>> {
        let val = self.iter.next();

        if let Some(node) = val {
            self.position = node.position.clone();
            return Some(node);
        }

        return None;
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