use std::slice::Iter;
use std::vec::IntoIter;
use crate::task::position::Position;

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
    data: Iter<'a, Node<T>>,
    pub position: Position,
    pub done: bool,
    pub current: Option<&'a Node<T>>,
}

impl<'a, T> NodeIter<'a, T> {
    pub fn new(data: &'a Vec<Node<T>>) -> NodeIter<'a, T> {
        NodeIter { data: data.into_iter(), position: Position::new(), current: None, done: false }
    }

    pub fn next(&mut self) -> Option<&Node<T>> {
        if self.done {
            return None;
        }

        return match self.data.next() {
            Some(node) => {
                self.position = node.position.clone();
                self.current = Some(node);

                Some(node)
            }

            None => {
                self.done = true;
                self.current = None;

                None
            }
        }
    }
}