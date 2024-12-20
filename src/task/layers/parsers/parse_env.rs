use crate::task::layers::tokenize::Token;
use crate::task::nodes::collection::{NodeCollection, CollectionResult};
use crate::task::nodes::iterator::NodeIter;
use crate::task::nodes::node::Node;
use crate::node;
use crate::task::data::range::Range;
use crate::task::data::string::StringExpression;
use crate::task::error::{Error, ErrorKind};

macro_rules! symbol {
    ($data:pat) => {
        Some(Node { data: Token::Symbol($data), .. })
    };
}

macro_rules! expect_symbol {
    ($data:expr, $symbol:pat) => {
        match $data {
            node!(data,position) => {
                if !matches!(data,Token::Symbol($symbol))  {
                    collection.throw(Error::new(format!("Expected '{symbol}'"), position, ErrorKind::Unexpected));
                    return;
                }
            }

            None => {
                collection.throw(Error::new(format!("Expected '{symbol}'"), position, ErrorKind::EndOfFile));
                return;
            }
        }
    };
}

pub enum MetaFieldType {
    Text,
    Integer,
    Decimal,
    Switch,
}

pub enum MetaValidator {
    Range(Range),
    Regex(Box<str>),
    Switch(Box<[StringExpression]>),
}

pub enum CompoundBase<'a> {
    Declaration(&'a str, MetaFieldType),
    Prompt(&'a str),
    Validator(MetaValidator),
}

pub enum FieldData {
    Text(Option<Box<str>>),
    Integer(Option<Range>),
    Decimal(Option<Range>),
    Switch(Box<[StringExpression]>)
}

pub enum Compound<'a> {
    Field {
        identifier: &'a str,
        prompt: &'a str,
        data: FieldData,
    }
}

pub fn parse_env(tokens: Vec<Node<Token>>) -> CollectionResult<Compound> {
    let mut first_pass = match parse_base(tokens) {
        NodeCollection::Ok(tokens) => tokens,
        NodeCollection::Failed(errors) => {
            return Err(errors);
        }
    };

    return second_pass(first_pass).into_result();
}

pub fn parse_base(tokens: Vec<Node<Token>>) -> NodeCollection<CompoundBase> {
    let mut iter = NodeIter::new(tokens);
    let mut collection = NodeCollection::new();

    while let node!(data, position) = iter.next() {
        match data {
            Token::Variable(identifier) => {
                expect_symbol!(iter.next(), ':');

            },

            _ => todo!()
        }
    }

    return collection;
}

pub fn second_pass(tokens: Vec<Node<CompoundBase>>) -> NodeCollection<Compound> {
    let mut iter = NodeIter::new(tokens);
    let mut collection = NodeCollection::new();

    while let node!(data, position) = iter.next() {
        match data {
            _ => {}
        }
    }

    return collection;
}


// fn parse_full(context: &mut ParseContext<Field>) {
//     let identifier = match context.read_segment() {
//         Some(identifier) => identifier,
//         None => return,
//     };
//
//     context.expect_symbol(':');
//
//     let prompt_type = match context.read_segment() {
//         Some(prompt_type) => prompt_type,
//         None => return,
//     };
//
//     context.expect_symbol('|');
//
//     let prompt = match context.read_string() {
//         Some(prompt) => prompt,
//         None => return,
//     };
//
//     context.expect_symbol('>');
//
//     let data = match parse_data(context) {
//         Some(value) => value,
//         None => return,
//     };
//
//     context.push(Field { identifier, prompt, data });
// }
//
// fn parse_data(context: &mut ParseContext<Field>) -> Option<FieldData> {
//     Some(match context.next() {
//         Some(next) => {
//             let Token { data, position } = next;
//
//             match data {
//                 TokenData::Symbol('{') => parse_options_list(context),
//                 TokenData::Regex(value) => FieldData::Input(Validator::Custom(value)),
//
//                 TokenData::Segment(value) => match value.as_ref() {
//                     "text" => FieldData::Input(Validator::Text),
//                     "number" => FieldData::Input(Validator::Number),
//                     _ => {
//                         context.throw_at(Arc::from(format!("Unknown data type '{value}'")), position);
//                         return None;
//                     }
//                 },
//
//                 _ => {
//                     let kind = data.stringify();
//                     context.throw_at(Arc::from(format!("Expected regex, segment or options list, found {kind}")), position);
//                     return None;
//                 }
//             }
//         }
//         None => {
//             context.throw(Arc::from("Expected data options"));
//             return None;
//         }
//     })
// }
//
// fn parse_options_list(context: &mut ParseContext<Field>) -> FieldData {
//     let mut options = Vec::new();
//     let mut coma = false;
//
//     while !context.done {
//         if let Some(Token { data, position }) = context.next() {
//             if let TokenData::Symbol('}') = data {
//                 break;
//             }
//
//             if coma {
//                 if let TokenData::Symbol(',') = data {
//                     coma = false;
//                 } else {
//                     context.throw_at(Arc::from("Expected ','"), position);
//                 }
//             } else {
//                 if let TokenData::String(value) = data {
//                     options.push(value);
//                     coma = true;
//                 } else {
//                     context.throw_at(Arc::from("Expected string literal"), position);
//                 }
//             }
//         } else {
//             context.throw(Arc::from("Unexpected end of file, expected '}'"));
//         }
//     }
//
//     FieldData::Switch(options)
// }