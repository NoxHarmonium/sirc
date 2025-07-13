use crate::types::object::RefType;
use crate::types::shared::NumberToken;
use serde::Serialize;

pub const DB_TOKEN: &str = ".DB";
pub const DW_TOKEN: &str = ".DW";
pub const DQ_TOKEN: &str = ".DQ";

pub const EQU_TOKEN: &str = ".EQU";

pub const DB_VALUE: u8 = 1;
pub const DW_VALUE: u8 = 2;
pub const DQ_VALUE: u8 = 4;

#[derive(Debug, Clone, Serialize)]
pub struct RefToken {
    pub name: String,
    pub ref_type: RefType,
}

#[derive(Debug, Clone, Serialize)]
pub enum DataType {
    Value(NumberToken),
    SymbolRef(RefToken),
    PlaceHolder(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct DataToken {
    pub size_bytes: u8,
    pub value: DataType,
}

#[derive(Debug, Clone, Serialize)]
pub struct EquToken {
    pub placeholder_name: String,
    pub number_token: NumberToken,
}
