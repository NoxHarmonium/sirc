use std::collections::BTreeMap;

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

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Default)]
pub struct ObjectDebugInfo {
    pub original_filename: String,
    pub original_input: String,
    pub program_to_input_offset_mapping: BTreeMap<usize, usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Default)]
pub struct ObjectDefinition {
    // The offset in these definitions is the location of the symbol
    pub symbols: Vec<SymbolDefinition>,
    // The offset in these definitions is the location of the ref in the program
    pub symbol_refs: Vec<SymbolRef>,
    pub program: Vec<u8>,
    pub debug_info: Option<ObjectDebugInfo>,
}

pub fn merge_object_definitions(
    object_definitions: &[ObjectDefinition],
) -> (ObjectDefinition, BTreeMap<u32, ObjectDebugInfo>) {
    let mut output: ObjectDefinition = ObjectDefinition::default();

    let mut debug_info_map: BTreeMap<u32, ObjectDebugInfo> = BTreeMap::new();

    for object_definition in object_definitions {
        let prev_offset: u32 = output
            .program
            .len()
            .try_into()
            .expect("Program length cannot not be larger than 32 bits");

        let offset_symbols: Vec<SymbolDefinition> = object_definition
            .symbols
            .iter()
            .map(|s| SymbolDefinition {
                name: s.name.clone(),
                offset: prev_offset + s.offset,
            })
            .collect();

        let offset_symbol_refs: Vec<SymbolRef> = object_definition
            .symbol_refs
            .iter()
            .map(|s| SymbolRef {
                name: s.name.clone(),
                offset: prev_offset + s.offset,
                ..*s
            })
            .collect();

        output.symbols.extend(offset_symbols);
        output.symbol_refs.extend(offset_symbol_refs);
        output.program.extend(object_definition.program.clone());
        if let Some(debug_info) = &object_definition.debug_info {
            let mut offset_mapping = debug_info.program_to_input_offset_mapping.clone();
            for value in offset_mapping.values_mut() {
                *value += prev_offset as usize;
            }
            let offset_debug_info = ObjectDebugInfo {
                original_filename: debug_info.original_filename.clone(),
                original_input: debug_info.original_input.clone(),
                program_to_input_offset_mapping: offset_mapping,
            };
            debug_info_map.insert(prev_offset, offset_debug_info);
        }
    }
    (output, debug_info_map)
}
