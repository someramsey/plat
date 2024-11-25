use crate::task::collection::Collection;
use crate::task::parsers::context::{Node, ParseContext};
use crate::task::tokenizer::str::Str;
use crate::task::tokenizer::str_expr::StrExpression;
use crate::task::tokenizer::tokenize::Token;

pub enum Validator {
    Text,
    Number,
    Custom(Str),
}

pub enum FieldData {
    Input(Validator),
    Switch(Vec<StrExpression>),
}

pub struct Field {
    pub identifier: Str,
    pub prompt: Str,
    pub data: FieldData,
}

pub enum MatchType {
    Regex(Str),
    Segment(Str),
}

pub struct Match {
    pub target: Str,

}

pub enum Statement {
    Field(Field),
    Match(Match)
}

pub fn parse_env(data: Vec<Token>) -> Collection<Node<Field>> {
    let mut iterator = data.into_iter();
    let mut context = ParseContext::new(iterator);

    while !context.is_done() {
        // parse_full(&mut context);
    }

    return context.collection;
}
//
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