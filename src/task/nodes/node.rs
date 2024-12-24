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
    ($data:pat) => { Node { data: $data, .. } };
    ($data:pat, $position: pat) => { Node { data: $data, position: $position }};
}

#[macro_export]
macro_rules! some_node {
    ($data:pat) => { Some(node!($data)) };
    ($data:pat, $position: pat) => { Some(node!($data, $position)) };
}

#[macro_export]
macro_rules! expect_node {
    ($node:expr, $expected:pat) => {
        match $node {
            v@$expected => Ok(v),
            some_node!(other, position) => Err(Error::Unexpected { expected: String::from(stringify!($expected)), received: format!("{}", other), position: position.clone() }),
            None => Err(Error::EndOfFile { expected: String::from(stringify!($expected)) })
        }
    };

    ($node:expr, $expected:pat => $result:expr) => {
        match $node {
            $expected => Ok($result),
            some_node!(other, position) => Err(Error::Unexpected { expected: String::from(stringify!($expected)), received: format!("{}", other), position: position.clone() }),
            None => Err(Error::EndOfFile { expected: String::from(stringify!($expected)) })
        }
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

impl<T> Debug for Node<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ {:?} {:?} }}", self.data, self.position)
    }
}