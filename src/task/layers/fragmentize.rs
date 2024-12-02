use crate::task::node::Node;
use crate::task::position::Position;
use std::str::Chars;

#[derive(Debug,)]
pub enum Fragment<'a> {
    AlphaNumeric(&'a str),
    Numeric(&'a str),
    Symbol(char),
}

struct Cursor<'a> {
    data: &'a str,
    head: usize,
    tail: usize,
}

struct Iteration<'a> {
    iterator: Chars<'a>,
    position: Position,
    current: Option<char>
}

impl<'a> Cursor<'a> {
    fn new(data: &'a str) -> Cursor {
        Cursor {
            data,
            head: 0,
            tail: 0,
        }
    }

    fn skip(&mut self) {
        self.head += 1;
        self.tail += 1;
    }

    fn take(&mut self) {
        self.head += 1;
    }

    fn collect(&mut self) -> &'a str {
        let slice = &self.data[self.tail..self.head];
        self.tail = self.head;

        return slice;
    }
}

impl Iteration<'_> {
    fn new(data: &str) -> Iteration {
        let mut iterator = data.chars();
        let current = iterator.next();

        Iteration {
            iterator, current,
            position: Position::new(),
        }
    }

    fn next(&mut self) {
        let next = self.iterator.next();
        self.current = next;
    }

    fn advance(&mut self, ch: char) {
        self.next();

        if ch == '\n' {
            self.position.newline();
        } else {
            self.position.shift();
        }
    }
}

pub fn fragmentize(data: &str) -> Vec<Node<Fragment>> {
    let mut fragments: Vec<Node<Fragment>> = Vec::new();

    let mut iteration = Iteration::new(data);
    let mut cursor = Cursor::new(data);

    while let Some(ch) = iteration.current {
        if ch.is_numeric() {
            while let Some(ch) = iteration.current {
                if !ch.is_numeric() {
                    fragments.push(Node::new(
                        Fragment::Numeric(cursor.collect()),
                        iteration.position.clone(),
                    ));
                    
                    break;
                }

                cursor.take();
                iteration.advance(ch);
            }
        } else if ch.is_alphanumeric() {
            while let Some(ch) = iteration.current {
                if !ch.is_alphanumeric() {
                    fragments.push(Node::new(
                        Fragment::AlphaNumeric(cursor.collect()),
                        iteration.position.clone(),
                    ));

                    break;
                }

                cursor.take();
                iteration.advance(ch);
            }
        } else {
            if !ch.is_whitespace() {
                fragments.push(Node::new(
                    Fragment::Symbol(ch),
                    iteration.position.clone(),
                ));
            }

            cursor.skip();
            iteration.advance(ch);
        }
    }

    return fragments;
}