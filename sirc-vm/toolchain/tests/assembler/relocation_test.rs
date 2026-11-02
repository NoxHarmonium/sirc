use super::support::assemble_object;
use peripheral_cpu::coprocessors::processing_unit::{
    definitions::InstructionData, encoding::decode_instruction,
};
use std::io::ErrorKind;
use toolchain::{data::linker::link_object, types::object::try_merge_object_definitions};
use toolchain::{data::object::try_build_object, parsers::shared::parse_tokens};

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
        let (_, tokens) = parse_tokens(&source).unwrap();
        let error = try_build_object(tokens, "test.sasm".into(), source.clone()).unwrap_err();
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

#[test]
fn data_relocations_include_segment_base_and_signed_displacement() {
    let object =
        assemble_object(":target\n.DQ @target\n.DQ @target.u\n.DQ @target.l\n.DQ @target.r\n");
    assert_eq!(
        link_object(&object, 0x0012_3456).unwrap(),
        [0, 0x12, 0x34, 0x56, 0, 0, 0, 0x12, 0, 0, 0x34, 0x56, 0, 0, 0xff, 0xfa,]
    );
}

#[test]
fn linker_rejects_relocations_on_unsupported_instruction_formats() {
    let reference = assemble_object("MOVI r1, @target.l\n:target\nNOOP\n");
    for source in ["ADDI r1, #0\nNOOP\n", "ADDR r1, r2\nNOOP\n"] {
        let mut object = assemble_object(source);
        object.symbols = reference.symbols.clone();
        object.symbol_refs = reference.symbol_refs.clone();
        let error = link_object(&object, 0).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert!(error.to_string().contains("Unsupported relocation"));
    }
}

#[test]
#[should_panic(expected = "Cannot find symbol [missing]")]
fn linker_reports_undefined_symbol() {
    link_object(&assemble_object("MOVI r1, @missing.l\n"), 0).unwrap();
}
