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
pub enum Modifier<'a> {
    At(Vec<StringSource<'a>>),
    To(Vec<StringSource<'a>>),
    For(String),
}

#[derive(Clone)]
pub enum Command<'a> {
    Copy(Position),
    Write(StringSource<'a>, Position),
}


#[derive(Debug)]
pub enum Instruction<'a> {
    Copy {
        source: Box<[StringSource<'a>]>,
        target: Box<[StringSource<'a>]>,
    },
    Write {
        value: StringSource<'a>,
        selector: String,
        target: Box<[StringSource<'a>]>,
    },
}

pub fn parse_commands(tokens: Vec<Node<Token>>) -> NodeCollection<Instruction> {
    let mut collection = NodeCollection::new();
    let mut iter = NodeIter::new(tokens);

    let mut stack: Vec<Modifier> = Vec::new();
    let mut command: Option<Command> = None;

    while let some_node!(data, position) = iter.peek() {
        match data {
            Token::Symbol(';') => {
                submit_stack(&mut collection, stack, command, position);

                stack = Vec::new();
                command = None;

                iter.skip();
            }

            Token::Symbol('{') => {
                iter.skip();
                scope(&mut iter, &mut collection, stack, command);

                stack = Vec::new();
                command = None;
            }

            _ => keyword(&mut iter, &mut collection, &mut stack, &mut command),
        }
    }

    return collection;
}


fn scope<'a>(iter: &mut NodeIter<Token<'a>>, collection: &mut NodeCollection<Instruction<'a>>, stack: Vec<Modifier<'a>>, command: Option<Command<'a>>) {
    let mut scope_stack = stack.clone();
    let mut scope_command = command.clone();

    while let some_node!(data, position) = iter.peek() {
        match data {
            Token::Symbol('}') => {
                iter.skip();
                return;
            }

            Token::Symbol('{') => {
                iter.skip();
                scope(iter, collection, scope_stack, scope_command);

                scope_stack = stack.clone();
                scope_command = command.clone();
            }

            Token::Symbol(';') => {
                submit_stack(collection, scope_stack, scope_command, position);

                scope_stack = stack.clone();
                scope_command = command.clone();

                iter.skip();
            }

            _ => keyword(iter, collection, &mut scope_stack, &mut scope_command),
        }
    }

    collection.throw(Error::EndOfFile { expected: String::from("'}'") });
}

fn keyword<'a>(iter: &mut NodeIter<Token<'a>>, collection: &mut NodeCollection<Instruction<'a>>, stack: &mut Vec<Modifier<'a>>, command: &mut Option<Command<'a>>) {
    expect_node!(iter.next(), "Keyword", some_node!(Token::Segment(keyword), position) => {
        match keyword {
            "at" => at_modifier(iter, collection, stack),
            "to" => to_modifier(iter, collection, stack),
            "for" => for_modifier(iter, collection, stack, position),
            "copy" => copy(collection, command, position),
            "write" => write(iter, collection, command, position),

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

fn string_param<'a>(iter: &mut NodeIter<Token<'a>>, collection: &mut NodeCollection<Instruction<'a>>) -> Result<StringSource<'a>, Error> {
    expect_node!(iter.next(), "String, Identifier or Scope",
        some_node!(Token::Value(Value::String(expr))) => StringSource::Expression(expr),
        some_node!(Token::Identifier(identifier)) => StringSource::Variable(identifier)
    )
}

macro_rules! push_or_merge_modifier {
    ($stack:expr, $variant:ident, $param:expr) => {
        if let Some(Modifier::$variant(vec)) = $stack.iter_mut().find(|modifier| matches!(modifier, Modifier::$variant(_))) {
            vec.push($param);
        } else {
            $stack.push(Modifier::$variant(vec![$param]));
        }
    };
}

fn at_modifier<'a>(iter: &mut NodeIter<Token<'a>>, collection: &mut NodeCollection<Instruction<'a>>, stack: &mut Vec<Modifier<'a>>) {
    let param = guard!(string_param(iter, collection)
        .map_err(|err| collection.throw(err)));

    push_or_merge_modifier!(stack, At, param);
}

fn to_modifier<'a>(iter: &mut NodeIter<Token<'a>>, collection: &mut NodeCollection<Instruction<'a>>, stack: &mut Vec<Modifier<'a>>) {
    let param = guard!(string_param(iter, collection)
        .map_err(|err| collection.throw(err)));

    push_or_merge_modifier!(stack, To, param);
}

fn for_modifier<'a>(iter: &mut NodeIter<Token<'a>>, collection: &mut NodeCollection<Instruction<'a>>, stack: &mut Vec<Modifier<'a>>, position: Position) {
    let param = guard!(expect_node!(iter.next(), "Regex", some_node!(Token::Value(Value::Regex(str))) => str)
        .map_err(|err| collection.throw(err)));

    if stack.iter().any(|modifier| matches!(modifier, Modifier::For(_))) {
        collection.throw(Error::Invalid {
            message: String::from("For command can not be chained multiple times."),
            received: String::from("for"),
            position,
        });
    } else {
        stack.push(Modifier::For(param));
    }
}

fn copy<'a>(collection: &mut NodeCollection<Instruction<'a>>, command: &mut Option<Command<'a>>, position: Position) {
    match command {
        Some(_) => collection.throw(Error::Other {
            message: String::from("Copy command can not be chained multiple times."),
            position,
        }),

        None => *command = Some(Command::Copy(position)),
    }
}

fn write<'a>(iter: &mut NodeIter<Token<'a>>, collection: &mut NodeCollection<Instruction<'a>>, command: &mut Option<Command<'a>>, position: Position) {
    match command {
        Some(_) => collection.throw(Error::Other {
            message: String::from("Write command can not be chained multiple times."),
            position,
        }),

        None => {
            let param = guard!(string_param(iter, collection)
                .map_err(|err| collection.throw(err)));

            *command = Some(Command::Write(param, position));
        }
    }
}

fn submit_stack<'a>(collection: &mut NodeCollection<Instruction<'a>>, stack: Vec<Modifier<'a>>, command: Option<Command<'a>>, position: &Position) {
    let Some(command) = command else {
        collection.throw(Error::Other {
            message: String::from("Chain does not contain an active command."),
            position: position.clone(),
        });
        return;
    };

    match command {
        Command::Copy(position) => {
            let mut source: Option<Box<[StringSource]>> = None;
            let mut target: Option<Box<[StringSource]>> = None;

            for modifier in stack {
                match modifier {
                    Modifier::At(vec) => source = Some(vec.into_boxed_slice()),
                    Modifier::To(vec) => target = Some(vec.into_boxed_slice()),

                    _ => collection.throw(Error::Other {
                        message: String::from("Copy command can only be used under 'at' and 'to' modifiers."),
                        position: position.clone(),
                    })
                }
            }

            let Some(source) = source else {
                collection.throw(Error::Other {
                    message: String::from("Copy command requires an 'at' modifier."),
                    position: position.clone(),
                });

                return;
            };

            let Some(target) = target else {
                collection.throw(Error::Other {
                    message: String::from("Copy command requires a 'to' modifier."),
                    position: position.clone(),
                });

                return;
            };

            collection.try_push(|| Node::new(
                Instruction::Copy { source, target },
                position.clone(),
            ));
        }

        Command::Write(value, position) => {
            let mut selector: Option<String> = None;
            let mut target: Option<Box<[StringSource]>> = None;

            for modifier in stack {
                match modifier {
                    Modifier::For(str) => selector = Some(str),
                    Modifier::To(vec) => target = Some(vec.into_boxed_slice()),

                    _ => collection.throw(Error::Other {
                        message: String::from("Write command can only be used under 'for' and 'to' modifiers."),
                        position: position.clone(),
                    })
                }
            }

            let Some(selector) = selector else {
                collection.throw(Error::Other {
                    message: String::from("Write command requires a 'for' modifier."),
                    position: position.clone(),
                });

                return;
            };

            let Some(target) = target else {
                collection.throw(Error::Other {
                    message: String::from("Write command requires a 'to' modifier."),
                    position: position.clone(),
                });

                return;
            };

            collection.try_push(|| Node::new(
                Instruction::Write { value, selector, target },
                position.clone(),
            ));
        }
    }
}

