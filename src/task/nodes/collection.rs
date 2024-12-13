use crate::task::error::Error;
use crate::task::nodes::node::Node;

pub enum NodeCollection<T> {
    Ok(Vec<Node<T>>),
    Failed(Vec<Error>),
}

impl<T> NodeCollection<T> {
    pub fn new() -> Self {
        NodeCollection::Ok(Vec::new())
    }

    pub fn throw(&mut self, err: Error) {
        match self {
            NodeCollection::Ok(_) => *self = NodeCollection::Failed(vec![err]),
            NodeCollection::Failed(vec) => vec.push(err)
        }
    }
}
