use std::fmt::{Display, Formatter};
use crate::task::nodes::node::Node;
use crate::task::position::Position;
use std::str::Chars;

#[derive(Debug)]
pub enum Fragment<'a> {
    AlphaNumeric(&'a str),
    Numeric(&'a str),
    Symbol(char)
}

impl Display for Fragment<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Fragment::AlphaNumeric(str) => write!(f, "AlphaNumeric ({})", *str),
            Fragment::Numeric(str) => write!(f, "Numeric ({})", *str),
            Fragment::Symbol(ch) => write!(f, "Symbol ({})", ch),
        }
    }
}

struct Cursor<'a> {
    data: &'a str,
    head: usize,
    tail: usize,
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

struct Iteration<'a> {
    iterator: Chars<'a>,
    position: Position,
    current: Option<char>
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
            let pos = iteration.position.clone();
            numeric(&mut fragments, &mut iteration, &mut cursor, pos);
        } else if ch.is_alphanumeric() {
            let pos = iteration.position.clone();
            alphanumeric(&mut fragments, &mut iteration, &mut cursor, pos);
        } else {
            if ch == '-' {
                let pos = iteration.position.clone();

                cursor.take();
                iteration.advance(ch);
                numeric(&mut fragments, &mut iteration, &mut cursor, pos);
            } else if !ch.is_whitespace() {
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

fn alphanumeric<'a>(fragments: &mut Vec<Node<Fragment<'a>>>, iteration: &mut Iteration, mut cursor: &mut Cursor<'a>, position: Position) {
    loop {
        match iteration.current {
            Some(ch) if ch.is_alphanumeric() => {
                cursor.take();
                iteration.advance(ch);
            }

            None | Some(_) => break
        }
    }

    fragments.push(Node::new(Fragment::AlphaNumeric(cursor.collect()), position));
}

fn numeric<'a>(fragments: &mut Vec<Node<Fragment<'a>>>, mut iteration: &mut Iteration, mut cursor: &mut Cursor<'a>, position: Position) {
    loop {
        match iteration.current {
            Some(ch) if ch.is_numeric() => {
                cursor.take();
                iteration.advance(ch);
            }

            None | Some(_) => break
        }
    }

    fragments.push(Node::new(Fragment::Numeric(cursor.collect()), position));
}