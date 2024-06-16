use std::collections::BTreeMap;

use sirc_vm::{
    debug_adapter::types::{ObjectDebugInfo, ProgramDebugInfo},
    utils::lines::{translate_line_column_to_pc, translate_pc_to_line_column},
};

static TEST_SOURCE_FILE_CONTENTS: &str = r"; Reserved space for 128x32 bit exception vectors
.ORG 0x0000
.DQ @init

.ORG 0x0200

:init
LOAD    r1, #5
LOAD    r2, #3
LOAD    r3, #64

:loop
ADDR    r2, r1
; Remember that COMP has the same argument order as SUBR
CMPR    r3, r2
BRAN|>= @loop

NOOP

; Halt CPU
COPI    r1, #0x14FF";

// TODO: Add tests for multiple source files for PC <-> Line/Column translation
// category=Testing

fn get_debug_info() -> ProgramDebugInfo {
    ProgramDebugInfo {
        debug_info_map: BTreeMap::from([(
            0,
            ObjectDebugInfo {
                checksum: "91ee2c63df623437ce2af12bba2f40fa2401783ea349e838914cc31b3fbb2f95"
                    .to_string(),
                original_filename: "UNIT_TEST.asm".to_string(),
                original_input: TEST_SOURCE_FILE_CONTENTS.to_string(),
                program_to_input_offset_mapping: BTreeMap::from([
                    (0x0200, 92),
                    (0x0202, 107),
                    (0x0204, 122),
                    (0x0206, 145),
                    (0x0208, 217),
                    (0x020A, 232),
                    (0x020C, 247),
                    (0x020E, 264),
                ]),
            },
        )]),
    }
}

#[test]
pub fn test_translate_pc_to_line_column() {
    let program_debug_info = get_debug_info();

    assert_eq!(None, translate_pc_to_line_column(&program_debug_info, 0));
    assert_eq!(
        Some((8, 1, "UNIT_TEST.asm".to_string())),
        translate_pc_to_line_column(&program_debug_info, 0x0200)
    );
    // Odd instructions are impossible
    assert_eq!(
        None,
        translate_pc_to_line_column(&program_debug_info, 0x0201)
    );
    assert_eq!(
        Some((9, 1, "UNIT_TEST.asm".to_string())),
        translate_pc_to_line_column(&program_debug_info, 0x0202)
    );
    assert_eq!(
        Some((15, 1, "UNIT_TEST.asm".to_string())),
        translate_pc_to_line_column(&program_debug_info, 0x0208)
    );
    assert_eq!(
        None,
        translate_pc_to_line_column(&program_debug_info, 0x2000)
    );
}

#[test]
pub fn test_translate_line_column_to_pc() {
    let program_debug_info = get_debug_info();

    assert_eq!(None, translate_pc_to_line_column(&program_debug_info, 0));
    assert_eq!(
        Some(0x0200),
        translate_line_column_to_pc(&program_debug_info, "UNIT_TEST.asm", (8, 1))
    );
    assert_eq!(
        Some(0x0202),
        translate_line_column_to_pc(&program_debug_info, "UNIT_TEST.asm", (9, 1))
    );
    assert_eq!(
        Some(0x0208),
        translate_line_column_to_pc(&program_debug_info, "UNIT_TEST.asm", (15, 1))
    );
    assert_eq!(
        Some(0x020E),
        translate_line_column_to_pc(&program_debug_info, "UNIT_TEST.asm", (21, 1))
    );
    // EOF
    assert_eq!(
        None,
        translate_line_column_to_pc(&program_debug_info, "UNIT_TEST.asm", (21, 20))
    );
    // Past EOF
    assert_eq!(
        None,
        translate_line_column_to_pc(&program_debug_info, "UNIT_TEST.asm", (21, 21))
    );
    assert_eq!(
        None,
        translate_line_column_to_pc(&program_debug_info, "UNIT_TEST.asm", (22, 1))
    );
}
