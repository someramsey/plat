use crate::task::layers::tokenize::Token;
use crate::task::position::Position;
use std::str::Chars;
use std::vec::IntoIter;

#[derive(Debug)]
pub enum FragmentData<'a> {
    AlphaNumeric(&'a str),
    Numeric(&'a str),
    Symbol(char),
}

#[derive(Debug)]
pub struct Fragment<'a> {
    pub data: FragmentData<'a>,
    pub position: Position,
}

struct Cursor {
    head: usize,
    tail: usize,
    position: Position,
}

impl Cursor {
    fn new() -> Cursor {
        Cursor {
            head: 0,
            tail: 0,
            position: Position::new(),
        }
    }

    fn shift(&mut self) {
        self.head += 1;
        self.position.shift();
    }
}

struct Iteration<'a> {
    chars: Chars<'a>,
    done: bool,
}

impl Iteration<'_> {
    fn new(iter: Chars) -> Iteration {
        Iteration {
            chars: iter,
            done: false,
        }
    }

    fn next(&mut self) -> Option<char> {
        if self.done {
            return None;
        }

        match self.chars.next() {
            Some(value) => Some(value),
            None => {
                self.done = true;
                None
            }
        }
    }
}

enum State {
    AlphaNumeric,
    Numeric,
    None,
}


pub fn fragmentize(data: &str) -> Vec<Fragment> {
    let mut iteration = Iteration::new(data.chars());

    let mut fragments: Vec<Fragment> = Vec::new();
    let mut cursor = Cursor::new();

    while !iteration.done {

    }

    return fragments;
}
