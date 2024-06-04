use sbrc_vm::debug_adapter::server::{format_instruction_ref, parse_instruction_ref};

#[test]
fn test_format_instruction_ref() {
    assert_eq!("pc:ABC", format_instruction_ref(0xABC));
}

#[test]
fn test_parse_instruction_ref() {
    assert_eq!(0xABC, parse_instruction_ref("pc:ABC"));
}
