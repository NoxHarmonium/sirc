use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum RefType {
    /// A 16-bit signed offset from the current PC
    Offset,
    /// An 8 bit signed offset from the current PC (used for short immediate instructions)
    SmallOffset,
    /// The lower 16 bits of a full 24-bit address of the target
    LowerWord,
    /// The upper 8 bits of a full 24-bit address of the target (8 bits are ignored)
    UpperWord,
    /// Full 24 bit program address. Only supported with the DQ directive, can not fit in instructions
    FullAddress,
    /// Automatically determine the ref type based on the instruction
    Implied,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SymbolDefinition {
    pub name: String,
    pub offset: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SymbolRef {
    pub name: String,
    pub offset: u32,
    pub ref_type: RefType,
    pub data_only: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectDefinition {
    // The offset in these definitions is the location of the symbol
    pub symbols: Vec<SymbolDefinition>,
    // The offset in these definitions is the location of the ref in the program
    pub symbol_refs: Vec<SymbolRef>,
    pub program: Vec<u8>,
}
