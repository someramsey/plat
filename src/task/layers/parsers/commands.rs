use std::sync::Arc;
use std::thread::yield_now;
use crate::{expect_node, expect_node_optional, node, some_node};
use crate::task::error::Error;
use crate::task::layers::tokenize::Token;
use crate::task::nodes::collection::NodeCollection;
use crate::task::nodes::iterator::NodeIter;
use crate::task::nodes::node::Node;
use crate::task::position::Position;
use crate::task::value::string::StringExpression;
use crate::task::value::Value;
macro_rules! guard {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(err) => return
        }
    };
}

#[derive(Clone)]
#[derive(Debug)]
pub enum StringSource<'a> {
    Variable(&'a str),
    Expression(StringExpression),
}

#[derive(Clone, Debug)]
pub enum Block<'a> {
    Copy,
    Write(Vec<StringSource<'a>>),
    At(Vec<StringSource<'a>>),
    To(Vec<StringSource<'a>>),
    For(Arc<()>),
}


#[derive(Debug)]
pub enum Instruction<'a> {
    Copy {
        source: Box<[StringSource<'a>]>,
        target: Box<[StringSource<'a>]>,
    },
    Write {
        value: Box<[StringSource<'a>]>,
        selector: (),
        target: Box<[StringSource<'a>]>,
    },
}

fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

pub fn parse_commands(tokens: Vec<Node<Token>>) -> NodeCollection<Instruction> {
    let mut collection = NodeCollection::new();
    let mut iter = NodeIter::new(tokens);

    let mut stack = Vec::new();

    while let some_node!(data) = iter.peek() {
        match data {
            Token::Symbol(';') => {
                iter.skip();

                println!("Stack: {:?}", stack);
            }

            Token::Symbol('{') => {
                iter.skip();
                scope(&mut iter, &mut collection, &mut stack);
            }

            _ => block(&mut iter, &mut collection, &mut stack)
        }
    }

    return collection;
}

fn merge_duplicates(stack: &mut Vec<Block>) {
    let mut i = 0;
    while i < stack.len() {
        let mut vec = Vec::new();
        vec.push(stack[i].clone());

        let mut j = i + 1;
        while j < stack.len() {
            if i != j && variant_eq(&stack[i], &stack[j]) {
                vec.push(stack[j].clone());
                stack.remove(j);
            }

            j += 1;
        }

        stack.remove(i);
        i += 1;
    }
}

fn scope<'a>(iter: &mut NodeIter<Token<'a>>, collection: &mut NodeCollection<Instruction<'a>>, stack: &mut Vec<Block<'a>>) {
    while let some_node!(data) = iter.peek() {
        if let Token::Symbol('}') = data {
            iter.skip();
            return;
        }

        block(iter, collection, &mut stack.clone());
    }

    collection.throw(Error::EndOfFile { expected: String::from("'}'") });
}

fn block<'a>(iter: &mut NodeIter<Token<'a>>, collection: &mut NodeCollection<Instruction<'a>>, stack: &mut Vec<Block<'a>>) {
    expect_node!(iter.next(), "Keyword", some_node!(Token::Segment(keyword), position) => {
        match keyword {
            "at" => at(iter, collection, stack),
            "to" => to(iter, collection, stack),
            "copy" => copy(stack, collection, position),

            _ => {
                collection.throw(Error::Unexpected {
                    expected: String::from("Keyword"),
                    received: keyword.to_string(),
                    position: position.clone(),
                });
            }
        }
    }).map_err(|err| collection.throw(err));
}

fn single_param<'a>(iter: &mut NodeIter<Token<'a>>, collection: &mut NodeCollection<Instruction<'a>>) -> Result<StringSource<'a>, Error> {
    expect_node!(iter.next(), "String, Identifier or Scope",
        some_node!(Token::Value(Value::String(expr))) => StringSource::Expression(expr),
        some_node!(Token::Identifier(identifier)) => StringSource::Variable(identifier)
    )
}

macro_rules! push_block {
    ($stack:expr, $block_variant:ident, $param:expr) => {
        if let Some(Block::$block_variant(vec)) = $stack.iter_mut().find(|block| matches!(block, Block::$block_variant(_))) {
            vec.push($param);
        } else {
            $stack.push(Block::$block_variant(vec![$param]));
        }
    };
}

fn at<'a>(iter: &mut NodeIter<Token<'a>>, collection: &mut NodeCollection<Instruction<'a>>, stack: &mut Vec<Block<'a>>) {
    let param = guard!(single_param(iter, collection)
        .map_err(|err| collection.throw(err)));

    push_block!(stack, At, param);
}

fn to<'a>(iter: &mut NodeIter<Token<'a>>, collection: &mut NodeCollection<Instruction<'a>>, stack: &mut Vec<Block<'a>>) {
    let param = guard!(single_param(iter, collection)
        .map_err(|err| collection.throw(err)));

    push_block!(stack, To, param);
}

fn copy<'a>(stack: &mut Vec<Block<'a>>, collection: &mut NodeCollection<Instruction<'a>>, position: Position) {
    if stack.iter().any(|block| matches!(block, Block::Copy)) {
        collection.throw(Error::Unexpected {
            expected: String::from("Copy block"),
            received: String::from("Copy block"),
            position
        });
    } else {
        stack.push(Block::Copy);
    }
}