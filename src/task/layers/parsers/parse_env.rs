use crate::task::data::number::Number;
use crate::task::data::string::StringExpression;
use crate::task::layers::tokenize::Token;
use crate::task::nodes::collection::NodeCollection;
use crate::task::nodes::iterator::NodeIter;
use crate::task::nodes::node::Node;
use crate::{node, symbol};

#[derive(Debug)]
pub enum Validator {
    Text,
    Number,
    Decimal,
    Integer,
    Regex(Box<str>),
    Range(i32, i32),
}

#[derive(Debug)]
pub enum FieldData {
    Input(Validator),
    Switch(StringExpression),
}

#[derive(Debug)]
pub struct Field {
    pub identifier: Box<str>,
    pub prompt: StringExpression,
    pub data: FieldData,
}

pub enum MatchPattern {
    String(StringExpression),
    Regex(Box<str>),
    Number(Number),
    Range(i32, i32),
    Any
}

pub struct Match {
    pattern: Vec<MatchPattern>,
    body: Vec<Statement>
}

pub enum Statement {
    Field(Field),
    Match(Match)
}


#[macro_export]
macro_rules! symbol {
    ($data:pat) => {
        Some(Node { data: Token::Symbol($data), .. })
    };
}

enum ParseToken {
    Scopes,
    Field,
    Prompt,
    Validator
}



pub fn parse_first(tokens: Vec<Node<Token>>) -> NodeCollection<ParseToken> {
    let mut iter = NodeIter::new(tokens);
    let mut collection = NodeCollection::new();
    
    return collection;
}

pub fn parse_last(tokens: Vec<Node<ParseToken>>) -> NodeCollection<Statement> {
    let mut iter = NodeIter::new(tokens);
    let mut collection = NodeCollection::new();
    
    return collection;
}

pub fn parse_env(tokens: Vec<Node<Token>>) -> NodeCollection<Statement> {
    let mut iter = NodeIter::new(tokens);
    let mut collection = NodeCollection::new();
    


    while let node!(head, position) = iter.next() {
        match head {
            Token::Variable(identifier) => {
                //expect ':'
                //expect type
                //expect '>'
                //expect string
                //expect semi

            }

            Token::Segment(segment) => {

            }

            _ => {
                println!("Unexpected Token");
            }
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