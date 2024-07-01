use crate::types::data::{DataToken, EquToken};
use crate::types::instruction::InstructionToken;
use serde::Serialize;

pub const REF_TOKEN_OFFSET_SUFFIX: &'static str = ".r";
pub const REF_TOKEN_LOWER_WORD_SUFFIX: &'static str = ".l";
pub const REF_TOKEN_UPPER_WORD_SUFFIX: &'static str = ".u";

#[derive(Debug, Serialize)]
pub enum NumberType {
    Hex,
    Decimal,
}

#[derive(Debug, Serialize)]
pub struct NumberToken {
    pub value: u32,
    pub number_type: NumberType,
}

#[derive(Debug, Serialize)]
pub struct LabelToken {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct OriginToken {
    pub offset: u32,
}

#[derive(Debug)]
pub enum Token {
    Comment(String),
    Label(LabelToken),
    Instruction(InstructionToken),
    Origin(OriginToken),
    Data(DataToken),
    Equ(EquToken),
}
