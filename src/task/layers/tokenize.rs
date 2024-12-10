use crate::nodes;
use crate::task::data::num::Num;
use crate::task::data::str::Str;
use crate::task::data::str_expr::StrExpression;
use crate::task::layers::fragmentize::Fragment;
use crate::task::nodes::iterator::NodeIter;
use crate::task::nodes::node::Node;
use std::sync::Arc;

#[derive(Debug)]
pub enum Token {
    Segment(Str),
    Variable(Str),
    Symbol(char),
    String(StrExpression),
    Number(Num),
    Regex(Str),
    Range(i32, i32),
}

impl Token {
    pub fn stringify(&self) -> Str {
        match self {
            Token::Segment(str) => str.clone(),
            Token::Symbol(ch) => Arc::from(format!("symbol '{}'", ch)),
            Token::Number(num) => num.stringify(),
            Token::String(str) => Arc::from("string"),
            Token::Regex(str) => Arc::from(format!("regex (\"{}\")", str)),
            Token::Variable(str) => Arc::from(format!("${}", str)),
            Token::Range(start, end) => Arc::from(format!("range {}..{}", start, end)),
        }
    }
}

pub fn tokenize(fragments: Vec<Node<Fragment>>) {
    let mut iter = NodeIter::new(fragments);

    while let Some(fragment) = iter.next() {
        match fragment.data {
            Fragment::Symbol(ch) => match ch {
                '"' => {
                    println!("String: ");
                }

                '/' => {
                    println!("Regex: ");
                }

                '$' => {
                    println!("Variable: ");
                }

                _ => {
                    println!("Symbol: '{}'", ch);
                }
            }

            Fragment::AlphaNumeric(str) => {
                println!("AlphaNumeric: {}", str);
            }

            Fragment::Numeric(base) => match iter.peek_slice(3) {
                Some(nodes!(Fragment::Symbol('.'), Fragment::Symbol('.'), rest)) => {
                    if let Fragment::Numeric(end) = rest {
                        println!("Integer-Integer Range: {}..{}", base, end);
                    } else {
                        println!("invalid");
                    }

                    iter.skip(3);
                }

                Some(nodes!(Fragment::Symbol('.'), rest)) => {
                    if let Fragment::Numeric(frac) = rest {
                        println!("Decimal: {}.{}", base, frac);
                    } else {
                        println!("invalid");
                    }

                    iter.skip(2);
                }

                Some(_) | None => {
                    println!("Integer: {}", base);
                }
            },
        }
    }
}

// pub fn tokenize(fragments: Vec<Node<Fragment>>) -> Collection<Node<Token>> {
//     let mut collection: Collection<Node<Token>> = Collection::new();
//     // let mut iter = NodeIter::new(&fragments);
//     let mut iter = fragments.into_iter().peekable();
//
//
//     while let Some(fragment) = iter.peek() {
//
//         match fragment.data {
//             Fragment::Numeric(slice) => {}
//
//             Fragment::Symbol(ch) => {
//                 let capture_result = match ch {
//                     '"' => capture_string(&mut iter),
//                     '/' => capture_regex(&mut iter),
//                     '$' => capture_variable(&mut iter),
//                     _ => Ok(Token::Symbol(ch))
//                 };
//
//                 match capture_result {
//                     Ok(token) => collection.push(Node::new(token, fragment.position.clone())),
//                     Err(err) => collection.throw(err)
//                 }
//             }
//
//             Fragment::AlphaNumeric(str) => {
//                 collection.push(Node::new(
//                     Token::Segment(Arc::from(str)),
//                     fragment.position.clone(),
//                 ));
//             }
//         }
//     }
//
//     return collection;
// }
//
// fn capture_variable(iter: &mut NodeIter<Fragment>) -> Result<Token, Error> {
//     expect!(iter, Fragment::AlphaNumeric(x) => x)
//         .map_err(|context| Error::with_context("Expected variable identifier after '$'", context))
//         .map(|identifier| Token::Variable(Arc::from(identifier)))
// }
//
// fn capture_regex(iter: &mut NodeIter<Fragment>) -> Result<Token, Error> {
//     let mut parts: Vec<Str> = Vec::new();
//
//     while let Some(fragment) = iter.current {
//         match fragment.data {
//             Fragment::Symbol('\\') => {
//                 iter.next();
//             }
//
//             Fragment::Symbol('/') => {
//                 return Ok(Token::Regex(concat_str(parts)));
//             }
//
//             Fragment::Numeric(slice) |
//             Fragment::AlphaNumeric(slice) => parts.push(Arc::from(slice)),
//
//             Fragment::Symbol(ch) => parts.push(ch_to_str(ch)),
//         }
//
//         iter.next();
//     }
//
//     Err(Error::new("Failed to capture regex", iter.position.clone(), ErrorCause::EndOfFile))
// }
//

// fn capture_string(iter: &mut Iter<Fragment>) -> Result<Token, Error> {
//     let mut expr = StrExpression::new();
//
//     while let Some(fragment) = iter.peek() {
//         match fragment.data {
//             Fragment::Symbol(ch) => {
//                 match ch {
//                     '"' => {
//                         iter.next();
//                         return Ok(Token::String(expr))
//                     },
//
//                     '\\' => {
//                         iter.next();
//                     }
//
//                     '$' => {
//                         let slice = expect!(iter, Fragment::AlphaNumeric(x) => x)
//                             .map_err(|context| Error::with_context("Expected variable identifier after '$'", context))?;
//
//                         expr.push(StrExpressionItem::Variable(Arc::from(slice)));
//                     }
//
//                     _ => expr.push(StrExpressionItem::Literal(ch_to_str(ch)))
//                 }
//             }
//
//             Fragment::Numeric(slice) |
//             Fragment::AlphaNumeric(slice) =>
//                 expr.push(StrExpressionItem::Literal(Arc::from(slice))),
//         }
//
//         iter.next();
//     }
//
//     Err(Error::new("Failed to capture string", iter.position.clone(), ErrorCause::EndOfFile))
// }
//

