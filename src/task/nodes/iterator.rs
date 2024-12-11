use crate::task::layers::fragmentize::Fragment;
use crate::task::nodes::node::Node;
use peekmore::PeekMoreIterator;
use std::slice::Iter;

pub type NodeIter<'a, T> = PeekMoreIterator<Iter<'a, Node<T>>>;


trait SkipBy<T> {
    fn skip_by(&mut self, count: usize);
}

impl<T: Iterator> SkipBy<T> for PeekMoreIterator<T> {
    fn skip_by(&mut self, count: usize) {
        for _ in 0..count {
            self.next();
        }
    }
}