use crate::task::error::Error;
use crate::task::nodes::node::Node;

pub enum NodeCollection<T> {
    Ok(Vec<Node<T>>),
    Failed(Vec<Error>),
}

pub type CollectionResult<T> = Result<Box<[Node<T>]>, Box<[Error]>>;

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

    pub fn throw_all(&mut self, errors: Vec<Error>) {
        match self {
            NodeCollection::Ok(_) => *self = NodeCollection::Failed(errors),
            NodeCollection::Failed(vec) => vec.extend(errors)
        }
    }

    pub fn into_boxed_result(self) -> CollectionResult<T> {
        match self {
            NodeCollection::Ok(tokens) => Ok(tokens.into_boxed_slice()),
            NodeCollection::Failed(errors) => Err(errors.into_boxed_slice())
        }
    }

    pub fn try_push(&mut self, getter: impl FnOnce() -> Node<T>) {
        if let NodeCollection::Ok(vec) = self {
            vec.push(getter())
        }
    }

    pub fn try_throw(&mut self, res: Result<(), Error>) {
        if let Err(err) = res {
            self.throw(err)
        }
    }
}
