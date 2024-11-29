use std::iter::Peekable;
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

struct Cursor<'a> {
    data: &'a str,
    head: usize,
    tail: usize
}

impl Cursor<'_> {
    fn new(data: &str) -> Cursor {
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

    fn collect(&mut self) -> &str {
        let slice =  &self.data[self.tail..self.head];
        self.tail = self.head;

        return slice;
    }
}

struct Iteration<'a> {
    iterator: Chars<'a>,
    position: Position,
    current: Option<char>,
    moved: bool,
}

impl Iteration<'_> {
    fn new(data: &str) -> Iteration {
        Iteration {
            iterator: data.chars(),
            position: Position::new(),
            current: None,
            moved: false,
        }
    }

    fn peek(&mut self) -> Option<char> {
        if !self.moved {
            self.current = self.iterator.next();
            self.moved = true;
        }

        self.current
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

pub fn fragmentize(data: &str) -> Vec<Fragment> {
    let mut fragments: Vec<Fragment> = Vec::new();

    let mut iteration = Iteration::new(data);
    let mut cursor = Cursor::new(data);

    while let Some(ch) = iteration.peek() {
        if ch.is_numeric() {
            while let Some(ch) = iteration.peek() {
                if !ch.is_numeric() {
                    println!("{:?}", cursor.collect());
                    break;
                }

                cursor.take();
                iteration.advance(ch);
            }
        } else if ch.is_alphanumeric() {
            while let Some(ch) = iteration.peek() {
                if !ch.is_alphanumeric() {
                    println!("Alphanumric: {}", cursor.collect());
                    break;
                }

                cursor.take();
                iteration.advance(ch);
            }
        } else {
            if !ch.is_whitespace() {
                println!("Symbol: {:?}", ch);
            }

            cursor.skip();
            iteration.advance(ch);
        }
    }

    return fragments;
}
