use peripheral_cpu::coprocessors::processing_unit::encoding::decode_instruction;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
use tempfile::tempdir;

fn assemble(directory: &Path, name: &str, source: &str) -> (Output, std::path::PathBuf) {
    let input = directory.join(format!("{name}.sasm"));
    let object = directory.join(format!("{name}.o"));
    fs::write(&input, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_assembler"))
        .args(["-i"])
        .arg(input)
        .arg("-o")
        .arg(&object)
        .output()
        .unwrap();
    (output, object)
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn auto_update_symbol_offsets_link_like_literals() {
    use peripheral_cpu::coprocessors::processing_unit::definitions::InstructionData;
    for (suffix, value) in [(".r", 4_u16), (".u", 0x12), (".l", 0x3404)] {
        let directory = tempdir().unwrap();
        let source =
            format!("LOAD|!= r3, (@target{suffix}, a)+\nSTOR|== -(@target{suffix}, a), r4\n");
        let (output, first) = assemble(directory.path(), "first", &source);
        assert_success(&output);
        let (output, second) = assemble(directory.path(), "second", ":target\nNOOP\n");
        assert_success(&output);
        let binary = directory.path().join("program.bin");
        let output = Command::new(env!("CARGO_BIN_EXE_linker"))
            .arg(first)
            .arg(second)
            .arg("-o")
            .arg(&binary)
            .args(["-s", "1192960"])
            .output()
            .unwrap();
        assert_success(&output);
        let bytes = fs::read(binary).unwrap();
        let literal_second = if suffix == ".r" { 2 } else { value };
        let literal = format!("LOAD|!= r3, (#{value}, a)+\nSTOR|== -(#{literal_second}, a), r4\n");
        let (output, object) = assemble(directory.path(), "literal", &literal);
        assert_success(&output);
        let object: toolchain::types::object::ObjectDefinition =
            postcard::from_bytes(&fs::read(object).unwrap()).unwrap();
        assert_eq!(&bytes[..8], &object.program[..8]);
        assert!(matches!(
            decode_instruction(bytes[..4].try_into().unwrap()),
            InstructionData::Immediate(_)
        ));
    }
    // A reference to an earlier label must keep a negative relative displacement.
    let directory = tempdir().unwrap();
    let (output, object) = assemble(directory.path(), "backward", ":target\nNOOP\nLOAD r3, (@target.r, a)+\nSTOR -(@target.r, a), r4\nLOAD r5, (@target.r, a)\n");
    assert_success(&output);
    let binary = directory.path().join("backward.bin");
    let output = Command::new(env!("CARGO_BIN_EXE_linker"))
        .arg(object)
        .arg("-o")
        .arg(&binary)
        .args(["-s", "100"])
        .output()
        .unwrap();
    assert_success(&output);
    let bytes = fs::read(binary).unwrap();
    for (offset, expected) in [(4, 0xfffe), (8, 0xfffc), (12, 0xfffa)] {
        let InstructionData::Immediate(data) =
            decode_instruction(bytes[offset..offset + 4].try_into().unwrap())
        else {
            panic!("expected immediate");
        };
        assert_eq!(data.value, expected);
    }
}
