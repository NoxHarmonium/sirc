use super::support::assemble_object;
use peripheral_cpu::coprocessors::processing_unit::{
    definitions::InstructionData, encoding::decode_instruction,
};
use toolchain::{data::linker::link_object, types::object::try_merge_object_definitions};

#[test]
fn auto_update_symbol_offsets_link_like_literals() {
    for (suffix, value) in [(".r", 4_u16), (".u", 0x12), (".l", 0x3404)] {
        let objects = [
            assemble_object(&format!(
                "LOAD|!= r3, (@target{suffix}, a)+\nSTOR|== -(@target{suffix}, a), r4\n"
            )),
            assemble_object(":target\nNOOP\n"),
        ];
        let (merged, _) = try_merge_object_definitions(&objects).unwrap();
        let bytes = link_object(&merged, 0x0012_3400).unwrap();
        let second = if suffix == ".r" { 2 } else { value };
        let literal = assemble_object(&format!(
            "LOAD|!= r3, (#{value}, a)+\nSTOR|== -(#{second}, a), r4\n"
        ));
        assert_eq!(&bytes[..8], &literal.program[..8]);
    }
    let object = assemble_object(":target\nNOOP\nLOAD r3, (@target.r, a)+\nSTOR -(@target.r, a), r4\nLOAD r5, (@target.r, a)\n");
    let bytes = link_object(&object, 100).unwrap();
    for (offset, expected) in [(4, 0xfffe), (8, 0xfffc), (12, 0xfffa)] {
        let InstructionData::Immediate(data) =
            decode_instruction(bytes[offset..offset + 4].try_into().unwrap())
        else {
            panic!("expected immediate");
        };
        assert_eq!(data.value, expected);
    }
}

#[test]
fn duplicate_labels_are_rejected_even_if_unused() {
    for usage in ["", "MOVI r1, @same.l\n"] {
        let source = format!(":same\nNOOP\n:same\n{usage}");
        let (_, tokens) = toolchain::parsers::shared::parse_tokens(&source).unwrap();
        let error =
            toolchain::data::object::try_build_object(tokens, "test.sasm".into(), source.clone())
                .unwrap_err();
        assert!(error.message.contains("Duplicate label [same]"));
        assert_eq!(error.input_offset, 11);
        let first = assemble_object(&format!(":same\nNOOP\n{usage}"));
        for objects in [[first.clone(), first.clone()], [first.clone(), first]] {
            assert!(try_merge_object_definitions(&objects)
                .unwrap_err()
                .contains("Duplicate label [same]"));
        }
    }
    let objects = [
        assemble_object(":Name\nNOOP\n"),
        assemble_object(":name\nNOOP\n"),
    ];
    let (merged, debug) = try_merge_object_definitions(&objects).unwrap();
    assert_eq!(merged.symbols[1].offset, 4);
    assert_eq!(debug.debug_info_map.len(), 2);
    assert!(link_object(&merged, 100).is_ok());
}
