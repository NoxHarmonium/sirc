use serde::Serialize;

use crate::parsers::data::RefToken;

#[derive(Debug, Serialize)]
pub enum DataType {
    Value(u32),
    SymbolRef(RefToken),
    PlaceHolder(String),
}

#[derive(Debug, Serialize)]
pub struct DataToken {
    pub size_bytes: u8,
    pub value: DataType,
}

#[derive(Debug, Serialize)]
pub struct EquToken {
    pub placeholder_name: String,
    pub value: u32,
}
