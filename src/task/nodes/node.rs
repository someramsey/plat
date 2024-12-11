use crate::task::position::Position;
use std::fmt::{Debug, Display};

#[macro_export]
macro_rules! nodes {
    ($($item:pat),*) => {
        [$(
            Some(Node { data: $item, .. })
        ),*,..]
    };
}

#[macro_export] 
macro_rules! node {
    ($data:pat, $position: pat) => {
        Some(Node { data: $data, position: $position })
    };
}

pub struct Node<T> {
    pub data: T,
    pub position: Position,
}

impl<T> Node<T> {
    pub fn new(data: T, position: Position) -> Node<T> {
        Node { data, position }
    }
}

impl<T> Debug for Node<T> where T: Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ {:?} {:?} }}", self.data, self.position)
    }
}