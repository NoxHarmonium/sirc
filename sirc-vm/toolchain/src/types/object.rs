use std::collections::BTreeMap;

use peripheral_cpu::coprocessors::processing_unit::definitions::INSTRUCTION_SIZE_WORDS;
use serde::{Deserialize, Serialize};

use sirc_vm::debug_adapter::types::{ObjectDebugInfo, ObjectDebugInfoMap, ProgramDebugInfo};

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
) -> (ObjectDefinition, ProgramDebugInfo) {
    let mut output: ObjectDefinition = ObjectDefinition::default();

    let mut debug_info_map: ObjectDebugInfoMap = BTreeMap::new();

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
            let prev_offset_words = prev_offset / INSTRUCTION_SIZE_WORDS;
            let offset_mapping = debug_info
                .program_to_input_offset_mapping
                .iter()
                .map(|(k, v)| (k + prev_offset_words, *v))
                .collect();
            println!(
                "object: {} prev_offset: {prev_offset}",
                debug_info.original_filename
            );
            let offset_debug_info = ObjectDebugInfo {
                original_filename: debug_info.original_filename.clone(),
                original_input: debug_info.original_input.clone(),
                program_to_input_offset_mapping: offset_mapping,
                checksum: debug_info.checksum.clone(),
            };
            debug_info_map.insert(prev_offset_words, offset_debug_info);
        }
    }
    (output, ProgramDebugInfo { debug_info_map })
}
