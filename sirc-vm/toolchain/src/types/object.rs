use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct SymbolDefinition {
    pub name: String,
    pub offset: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct SymbolRef {
    pub name: String,
    pub offset: u32,
    pub ref_type: RefType,
    pub data_only: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ObjectDefinition {
    // The offset in these definitions is the location of the symbol
    pub symbols: Vec<SymbolDefinition>,
    // The offset in these definitions is the location of the ref in the program
    pub symbol_refs: Vec<SymbolRef>,
    pub program: Vec<u8>,
}

pub fn merge_object_definitions(object_definitions: &[ObjectDefinition]) -> ObjectDefinition {
    let empty_definition: ObjectDefinition = ObjectDefinition {
        symbols: vec![],
        symbol_refs: vec![],
        program: vec![],
    };

    object_definitions
        .iter()
        .fold(empty_definition, |prev, curr| -> ObjectDefinition {
            let prev_offset: u32 = prev
                .program
                .len()
                .try_into()
                .expect("Program length cannot not be larger than 32 bits");

            let offset_symbols: Vec<SymbolDefinition> = curr
                .symbols
                .iter()
                .map(|s| SymbolDefinition {
                    name: s.name.clone(),
                    offset: prev_offset + s.offset,
                })
                .collect();

            let offset_symbol_refs: Vec<SymbolRef> = curr
                .symbol_refs
                .iter()
                .map(|s| SymbolRef {
                    name: s.name.clone(),
                    offset: prev_offset + s.offset,
                    ..*s
                })
                .collect();

            ObjectDefinition {
                symbols: [prev.symbols.as_slice(), offset_symbols.as_slice()].concat(),
                symbol_refs: [prev.symbol_refs.as_slice(), offset_symbol_refs.as_slice()].concat(),
                program: [prev.program.as_slice(), curr.program.as_slice()].concat(),
            }
        })
}
