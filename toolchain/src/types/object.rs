use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SymbolDefinition {
    pub name: String,
    pub offset: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectDefinition {
    pub symbols: Vec<SymbolDefinition>,
    pub symbol_refs: Vec<SymbolDefinition>,
    pub program: Vec<u8>,
}
