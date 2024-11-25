use crate::task::error::Error;
use crate::task::tokenize::Token;

pub enum Collection<T> {
    Ok(Vec<Token>),
    Failed(Vec<Error>),
}

impl<T> Collection<T> {
    pub fn new() -> Self {
        Collection::Ok(Vec::new())
    }

    pub fn push(&mut self, data: T) {
        if let Collection::Ok(vec) = self {
            vec.push(data);
        }
    }

    pub fn throw(&mut self, err: Error) {
        match self {
            Collection::Ok(_) => *self = Collection::Failed(vec![err]),
            Collection::Failed(vec) => vec.push(err)
        }
    }
}
