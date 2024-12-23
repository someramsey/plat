use crate::task::error::Error;
use crate::task::nodes::node::Node;

pub enum NodeCollection<T> {
    Ok(Vec<Node<T>>),
    Failed(Vec<Error>),
}

pub type CollectionResult<T> = Result<Vec<Node<T>>, Vec<Error>>;

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
    
    pub fn into_result(self) -> CollectionResult<T> {
        match self {
            NodeCollection::Ok(tokens) => Ok(tokens),
            NodeCollection::Failed(errors) => Err(errors)
        }
    }

    pub fn try_collect(&mut self, getter: impl FnOnce() -> Node<T>) {
        if let NodeCollection::Ok(vec) = self {
            vec.push(getter())
        }
    }
}
