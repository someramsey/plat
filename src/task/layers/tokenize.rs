use crate::task::error::Error;
use crate::task::layers::fragmentize::Fragment;
use crate::task::nodes::collection::NodeCollection;
use crate::task::nodes::iterator::NodeIter;
use crate::task::nodes::node::Node;
use crate::task::value::number::NumberValue;
use crate::task::value::string::{ch_to_box_str, StringExpression, StringExpressionPart};
use crate::task::value::Value;
use crate::{some_node, nodes, node, expect_node};
use std::fmt::{format, Debug, Display, Formatter};
use crate::task::position::Position;
use crate::task::value::range::RangeValue;

#[derive(Debug)]
pub enum Token<'a> {
    Segment(&'a str),
    Symbol(char),
    Identifier(&'a str),
    Value(Value),
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Segment(str) => write!(f, "{}", *str),
            Token::Symbol(ch) => write!(f, "symbol '{}'", ch),
            Token::Identifier(str) => write!(f, "identifier '{}'", str),
            Token::Value(value) => write!(f, "{}", value),
        }
    }
}

pub fn tokenize(fragments: Vec<Node<Fragment>>) -> NodeCollection<Token> {
    let mut iter: NodeIter<Fragment> = NodeIter::new(fragments);
    let mut collection: NodeCollection<Token> = NodeCollection::new();

    while let some_node!(data, position) = iter.next() {
        let capture = match data {
            Fragment::AlphaNumeric(str) => Ok(Token::Segment(str)),
            Fragment::Numeric(base) => tokenize_numeric(&mut iter, base),

            Fragment::Symbol(ch) => match ch {
                '"' => capture_string(&mut iter),
                '/' => capture_regex(&mut iter),
                '$' => capture_variable(&mut iter),

                _ => Ok(Token::Symbol(ch))
            }
        };

        match capture {
            Ok(token) => {
                if let NodeCollection::Ok(ref mut vec) = collection {
                    vec.push(Node::new(token, position.clone()));
                }
            }

            Err(err) => collection.throw(err)
        }
    }

    return collection;
}

fn tokenize_numeric<'a>(iter: &mut NodeIter<Fragment<'a>>, base: &str) -> Result<Token<'a>, Error> {
    let base_position = iter.position.clone();

    return match iter.peek_slice(3) {
        nodes!(Fragment::Symbol('.'), Fragment::Symbol('.'), rest) => {
            return match rest {
                Fragment::Numeric(end) => {
                    let parsed_base = base.parse::<i32>()
                        .map_err(|_| Error::Other { message: Box::from("Failed to parse range begin"), position: base_position.clone() })?;

                    let parsed_end = end.parse::<i32>()
                        .map_err(|_| Error::Other { message: Box::from("Failed to parse range end"), position: iter.position.clone() })?;

                    iter.skip_by(3);

                    Ok(Token::Value(Value::Range(RangeValue(parsed_base, parsed_end))))
                }

                _ => Err(Error::Unexpected { expected: Box::from("Numeric"), received: Box::from(format!("{}", rest)), position: iter.position.clone() })
            };
        }

        nodes!(Fragment::Symbol('.'), rest) => {
            if let Fragment::Numeric(frac) = rest {
                let value = format!("{}.{}", base, frac).parse::<f32>()
                    .map_err(|_| Error::Other { message: Box::from("Failed to parse decimal"), position: base_position.clone() })?;

                iter.skip_by(2);

                return Ok(Token::Value(Value::Number(NumberValue::Decimal(value))));
            }

            iter.skip();

            Err(Error::Unexpected { expected: Box::from("Numeric"), received: Box::from(""), position: iter.position.clone() })
        }

        _ => {
            let value = base.parse::<i32>()
                .map_err(|err| Error::Other { message: Box::from(format!("Failed to parse integer ({err})")), position: base_position.clone() })?;

            Ok(Token::Value(Value::Number(NumberValue::Integer(value))))
        }
    };
}

fn capture_variable<'a>(iter: &mut NodeIter<Fragment<'a>>) -> Result<Token<'a>, Error> {
    match expect_node!(iter.peek(), some_node!(Fragment::AlphaNumeric(slice)) => slice) {
        Ok(slice) => {
            let slice = *slice;
            
            iter.next();
            Ok(Token::Identifier(slice))
        },
        Err(err) => Err(err)
    }
}

fn capture_regex<'a>(iter: &mut NodeIter<Fragment<'a>>) -> Result<Token<'a>, Error> {
    let mut data: String = String::new();

    while let Some(fragment) = iter.next() {
        match fragment.data {
            Fragment::Symbol('\\') => {
                iter.next();
            }

            Fragment::Symbol('/') => {
                iter.next();
                return Ok(Token::Value(Value::Regex(data.into_boxed_str())));
            }

            Fragment::Numeric(slice) |
            Fragment::AlphaNumeric(slice) => data.push_str(slice),

            Fragment::Symbol(ch) => data.push(ch)
        }
    }

    Err(Error::EndOfFile { expected: Box::from("'/") })
}

fn capture_string<'a>(iter: &mut NodeIter<Fragment<'a>>) -> Result<Token<'a>, Error> {
    let mut expr: Vec<StringExpressionPart> = Vec::new();

    while let Some(fragment) = iter.peek() {
        match fragment.data {
            Fragment::Numeric(slice) |
            Fragment::AlphaNumeric(slice) =>
                expr.push(StringExpressionPart::Literal(Box::from(slice))),

            Fragment::Symbol(ch) => {
                match ch {
                    '\\' => { iter.next(); }

                    '"' => {
                        iter.next();

                        return Ok(Token::Value(Value::String(StringExpression::new(expr))));
                    }

                    '$' => {
                        match expect_node!(iter.next(), some_node!(Fragment::AlphaNumeric(slice)) => slice) {
                            Ok(slice) => expr.push(StringExpressionPart::Variable(Box::from(slice))),
                            Err(err) => return Err(err)
                        }
                    }

                    _ => match ch_to_box_str(ch) {
                        Ok(slice) => expr.push(StringExpressionPart::Literal(slice)),
                        Err(err) => return Err(Error::Other { message: Box::from(format!("Internal error: {}", err)), position: iter.position.clone() })
                    },
                }
            }
        }

        iter.next();
    }

    Err(Error::EndOfFile { expected: Box::from("\"") })
}


