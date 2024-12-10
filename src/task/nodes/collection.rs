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

    pub fn push(&mut self, data: Node<T>) {
        if let NodeCollection::Ok(vec) = self {
            vec.push(data);
        }
    }

    pub fn throw(&mut self, err: Error) {
        match self {
            NodeCollection::Ok(_) => *self = NodeCollection::Failed(vec![err]),
            NodeCollection::Failed(vec) => vec.push(err)
        }
    }
}
