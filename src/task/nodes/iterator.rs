use crate::task::nodes::node::Node;
use std::cmp::min;

pub struct NodeIter<T> {
    collection: Vec<Node<T>>,
    index: usize,
}

impl<T> NodeIter<T> {
    pub fn new(collection: Vec<Node<T>>) -> NodeIter<T> {
        NodeIter { collection, index: 0 }
    }

    pub fn next(&mut self) -> Option<&Node<T>> {
        if self.index >= self.collection.len() {
            return None;
        }

        let val = self.collection.get(self.index);
        self.index += 1;

        return val;
    }

    pub fn skip(&mut self, count: usize) {
        self.index += count;
    }

    pub fn peek(&self) -> Option<&Node<T>> {
        self.collection.get(self.index)
    }

    pub fn peek_slice(&self, offset: usize) -> Option<&[Node<T>]> {
        if self.index >= self.collection.len() {
            return None;
        }

        let begin = self.index;
        let end = min(self.index + offset, self.collection.len());

        self.collection.get(begin..end)
    }
}