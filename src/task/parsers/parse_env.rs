use std::sync::Arc;
use crate::task::error::Error;
use crate::task::parsers::context::{get_result, Node, ParseContext};
use crate::task::parsers::parse_script::Instruction;
use crate::task::position::Position;
use crate::task::tokenize::{Token, TokenData};

pub enum Validator {
    Text,
    Number,
    Custom(Arc<str>),
}

type SwitchOptions = Arc<Arc<str>>;

pub enum FieldData {
    Input(Validator),
    Switch(SwitchOptions),
}

pub struct Field {
    pub identifier: Arc<str>,
    pub prompt: Arc<str>,
    pub data: FieldData,
}

pub fn parse_env(data: Vec<Token>) -> Result<Vec<Node<Field>>, Vec<Error>> {
    let mut iterator = data.into_iter();
    let mut context = ParseContext::new(iterator);

    while !context.done {
        once(&mut context);
    }

    return get_result(context);
}

fn once(context: &mut ParseContext<Field>) {
    let identifier = match context.read_segment() {
        Some(identifier) => identifier,
        None => return,
    };

    context.expect_symbol(':');

    let prompt_type = match context.read_segment() {
        Some(prompt_type) => prompt_type,
        None => return,
    };

    context.expect_symbol('|');

    let prompt = match context.read_string() {
        Some(prompt) => prompt,
        None => return,
    };

    context.expect_symbol('>');

    let data = match context.next() {
        Some(next) => {
            let Token { data, position } = next;

            match data {
                TokenData::String(value) => FieldData::Input(Validator::Custom(value)),

                // TokenData::Symbol('{') => {
                //
                //
                //
                //
                // }

                _=> {
                    //throw
                    return;
                }
            }
        }
        None => {
            //throw
            return;
        },
    };
}