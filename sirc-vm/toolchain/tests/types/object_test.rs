use std::collections::BTreeMap;

use sbrc_vm::debug_adapter::types::ObjectDebugInfo;
use toolchain::types::object::{
    merge_object_definitions, ObjectDefinition, RefType, SymbolDefinition, SymbolRef,
};

#[test]
fn test_merge_object_definition_single_definition_untouched() {
    let first_def = ObjectDefinition {
        symbols: vec![
            SymbolDefinition {
                name: "first_first".to_string(),
                offset: 3,
            },
            SymbolDefinition {
                name: "first_second".to_string(),
                offset: 15,
            },
        ],
        symbol_refs: vec![
            SymbolRef {
                name: "second_second".to_string(),
                offset: 7,
                ref_type: RefType::LowerWord,
                data_only: false,
            },
            SymbolRef {
                name: "second_first".to_string(),
                offset: 0,
                ref_type: RefType::LowerWord,
                data_only: true,
            },
        ],
        program: vec![
            0xF, 0xF, 0xE, 0xE, 0xD, 0xD, 0xC, 0xC, 0xB, 0xB, 0xA, 0xA, 0x9, 0x9, 0x8, 0x8,
        ],
        debug_info: None,
    };

    let (merged, merged_debug_info) = merge_object_definitions(&[first_def.clone()]);

    assert_eq!(first_def, merged);
    assert_eq!(0, merged_debug_info.debug_info_map.len());
}

#[test]
fn test_merge_object_definitions_counts() {
    let first_def = ObjectDefinition {
        symbols: vec![
            SymbolDefinition {
                name: "first_first".to_string(),
                offset: 3,
            },
            SymbolDefinition {
                name: "first_second".to_string(),
                offset: 15,
            },
        ],
        symbol_refs: vec![
            SymbolRef {
                name: "second_second".to_string(),
                offset: 7,
                ref_type: RefType::LowerWord,
                data_only: false,
            },
            SymbolRef {
                name: "second_first".to_string(),
                offset: 0,
                ref_type: RefType::LowerWord,
                data_only: true,
            },
        ],
        program: vec![
            0xF, 0xF, 0xE, 0xE, 0xD, 0xD, 0xC, 0xC, 0xB, 0xB, 0xA, 0xA, 0x9, 0x9, 0x8, 0x8,
        ],
        debug_info: Some(ObjectDebugInfo {
            original_filename: "UNIT_TEST_FIRST_DEF".to_string(),
            original_input: "some original input".to_string(),
            program_to_input_offset_mapping: BTreeMap::from([(1, 2), (3, 4)]),
            checksum: "ABCD".to_string(),
        }),
    };
    let second_def = ObjectDefinition {
        symbols: vec![
            SymbolDefinition {
                name: "second_first".to_string(),
                offset: 3,
            },
            SymbolDefinition {
                name: "second_second".to_string(),
                offset: 15,
            },
        ],
        symbol_refs: vec![
            SymbolRef {
                name: "first_second".to_string(),
                offset: 7,
                ref_type: RefType::LowerWord,
                data_only: false,
            },
            SymbolRef {
                name: "first_first".to_string(),
                offset: 0,
                ref_type: RefType::LowerWord,
                data_only: true,
            },
        ],
        program: vec![
            0x0, 0x0, 0x1, 0x1, 0x2, 0x2, 0x3, 0x3, 0x4, 0x4, 0x5, 0x5, 0x6, 0x6, 0x7, 0x7,
        ],
        debug_info: Some(ObjectDebugInfo {
            original_filename: "UNIT_TEST_SECOND_DEF".to_string(),
            original_input: "some original input".to_string(),
            program_to_input_offset_mapping: BTreeMap::from([(6, 7), (8, 9)]),
            checksum: "ABCD".to_string(),
        }),
    };
    let (merged, merged_debug_info) = merge_object_definitions(&[first_def, second_def]);

    assert_eq!(4, merged.symbols.len());
    assert_eq!(4, merged.symbol_refs.len());
    assert_eq!(32, merged.program.len());
    assert_eq!(2, merged_debug_info.debug_info_map.keys().len());
}
