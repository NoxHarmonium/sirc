use crate::types::data::{DataToken, EquToken};
use crate::types::instruction::InstructionToken;
use serde::Serialize;

pub const REF_TOKEN_OFFSET_SUFFIX: &str = ".r";
pub const REF_TOKEN_LOWER_WORD_SUFFIX: &str = ".l";
pub const REF_TOKEN_UPPER_WORD_SUFFIX: &str = ".u";

#[derive(Debug, Clone, Serialize)]
pub enum NumberType {
    Hex,
    Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct NumberToken {
    pub value: u32,
    pub number_type: NumberType,
}

#[derive(Debug, Clone, Serialize)]
pub struct LabelToken {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OriginToken {
    pub offset: u32,
}

#[derive(Debug, Clone)]
pub enum Token {
    Comment(String),
    Label(LabelToken),
    Instruction(InstructionToken),
    Origin(OriginToken),
    Data(DataToken),
    Equ(EquToken),
}
