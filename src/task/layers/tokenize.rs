use crate::task::data::number::Number;
use crate::task::data::range::Range;
use crate::task::data::string::{ch_to_box, StringExpression, StringPart};
use crate::task::error::{Error, ErrorKind};
use crate::task::layers::fragmentize::Fragment;
use crate::task::nodes::collection::NodeCollection;
use crate::task::nodes::iterator::NodeIter;
use crate::task::nodes::node::Node;
use crate::{node, nodes};
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum Token<'a> {
    Segment(&'a str),
    Symbol(char),
    String(StringExpression),
    Variable(&'a str),
    Regex(Box<str>),
    Numeric(Number),
    Range(Range),
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Segment(str) => write!(f, "{}", *str),
            Token::Numeric(num) => write!(f, "{}", num),
            Token::Symbol(ch) => write!(f, "symbol '{}'", ch),
            Token::String(_) => write!(f, "string"),
            Token::Regex(str) => write!(f, "regex (\"{}\")", str),
            Token::Variable(str) => write!(f, "${}", str),
            Token::Range(range) => write!(f, "({})", range),
        }
    }
}

pub fn tokenize(fragments: Vec<Node<Fragment>>) -> NodeCollection<Token> {
    let mut iter: NodeIter<Fragment> = NodeIter::new(fragments);
    let mut collection: NodeCollection<Token> = NodeCollection::new();

    while let node!(data, position) = iter.next() {
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
    let begin = iter.position.clone();

    return match iter.peek_slice(2) {
        nodes!(Fragment::Symbol('.'), Fragment::Symbol('.')) => {
            iter.skip_by(2);

            if let node!(Fragment::Numeric(end), position) = iter.peek() {
                let begin = base.parse::<i32>()
                    .map_err(|_| Error::new("Failed to parse range start", begin, ErrorKind::InternalError))?;

                let end = end.parse::<i32>()
                    .map_err(|_| Error::new("Failed to parse range end", position.clone(), ErrorKind::InternalError))?;

                iter.skip();

                return Ok(Token::Range(Range::new(begin, end)));
            }

            Err(Error::new("Expected number after '..'", begin, ErrorKind::Unexpected))
        }

        nodes!(Fragment::Symbol('.'), rest) => {
            if let Fragment::Numeric(frac) = rest {
                let value = format!("{}.{}", base, frac).parse::<f32>()
                    .map_err(|_| Error::new("Failed to parse decimal", begin, ErrorKind::InternalError))?;

                iter.skip_by(2);

                return Ok(Token::Numeric(Number::Decimal(value)));
            }

            iter.skip();

            Err(Error::new("Expected number after '.'", iter.position.clone(), ErrorKind::Unexpected))
        }

        _ => {
            let value = base.parse::<i32>()
                .map_err(|_| Error::new("Failed to parse integer", begin, ErrorKind::InternalError))?;

            Ok(Token::Numeric(Number::Integer(value)))
        }
    };
}

//TODO: concat the internal error messages when throwing

fn capture_variable<'a>(iter: &mut NodeIter<Fragment<'a>>) -> Result<Token<'a>, Error> {
    if let node!(Fragment::AlphaNumeric(slice), position) = iter.peek() {
        let dw = *slice;
        iter.skip();

        return Ok(Token::Variable(dw));
    }

    Err(Error::new("Expected variable identifier", iter.position.clone(), ErrorKind::Unexpected))
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
                return Ok(Token::Regex(data.into_boxed_str()));
            }

            Fragment::Numeric(slice) |
            Fragment::AlphaNumeric(slice) => data.push_str(slice),

            Fragment::Symbol(ch) => data.push(ch)
        }
    }

    Err(Error::new("Unterminated Regex, Expected '/'", iter.position.clone(), ErrorKind::EndOfFile))
}

fn capture_string<'a>(iter: &mut NodeIter<Fragment<'a>>) -> Result<Token<'a>, Error> {
    let mut expr: Vec<StringPart> = Vec::new();

    while let Some(fragment) = iter.peek() {
        match fragment.data {
            Fragment::Numeric(slice) |
            Fragment::AlphaNumeric(slice) =>
                expr.push(StringPart::Literal(Box::from(slice))),

            Fragment::Symbol(ch) => {
                match ch {
                    '\\' => { iter.next(); }

                    '"' => {
                        iter.next();

                        let slice = expr.into_boxed_slice();
                        return Ok(Token::String(slice));
                    },

                    '$' => match iter.next() {
                        node!(Fragment::AlphaNumeric(slice), position) => {
                            expr.push(StringPart::Variable(Box::from(slice)));
                        }

                        _ => return Err(Error::new("Expected variable identifier after '$'", iter.position.clone(), ErrorKind::Unexpected))
                    },

                    _ => match ch_to_box(ch) {
                        Ok(slice) => expr.push(StringPart::Literal(slice)),
                        Err(_) => return Err(Error::new("Failed to encode utf8", iter.position.clone(), ErrorKind::InternalError))
                    },
                }
            }
        }

        iter.next();
    }

    Err(Error::new("Unterminated String, Expectd '\"'", iter.position.clone(), ErrorKind::EndOfFile))
}


