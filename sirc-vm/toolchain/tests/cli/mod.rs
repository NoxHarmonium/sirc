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

fn assert_assembly_error(source: &str, expected: &str, location: &str) {
    let directory = tempdir().unwrap();
    let (output, object) = assemble(directory.path(), "invalid", source);
    assert!(!output.status.success());
    let message = String::from_utf8_lossy(&output.stderr);
    assert!(message.contains(expected), "{source}: {message}");
    assert!(
        message.contains(&format!("invalid.sasm:{location}")),
        "{message}"
    );
    assert!(!message.contains("panicked"), "{message}");
    assert!(!object.exists());
}

#[test]
fn assembler_reports_located_errors_without_panicking() {
    assert_assembly_error("; heading\n.ORG 0x0001\nNOOP\n", "even word address", "2:1");
    assert_assembly_error("; heading\nADDIr1, #1\n", "whitespace", "2:");
    assert_assembly_error(
        "; heading\nADDI r1, $MISSING\n",
        "Undefined constant",
        "2:1",
    );
    assert_assembly_error(
        "; heading\n.EQU $BIG #0x10000\nADDI r1, $BIG\n",
        "16-bit immediate",
        "3:1",
    );
    let directory = tempdir().unwrap();
    let (output, object) = assemble(
        directory.path(),
        "valid",
        ".ORG 0x0002\nADDI[N]|!=\tr1, #1\nNOOP; comment\nWAIT\nRETS\n",
    );
    assert_success(&output);
    assert!(object.exists());
}

#[test]
fn assembler_rejects_every_program_counter_auto_update_form() {
    for offset in ["#0, ", "r2, ", "@target, ", "$N, ", ""] {
        for source in [
            format!("LOAD|!= r1, ({offset}p)+"),
            format!("STOR|== -({offset}p), r1"),
            format!("LDEA a, -({offset}p)"),
            format!("LDEL a, ({offset}p)+"),
            format!("LJSR ({offset}p)+"),
        ] {
            assert_assembly_error(
                &format!(".EQU $N #0\n{source}\n:target\nNOOP\n"),
                "Aliased register writes",
                "2:",
            );
        }
    }
    let directory = tempdir().unwrap();
    for source in [
        "LOAD r1, (#0, p)",
        "STOR (#0, p), r1",
        "LDEA a, (#0, p)",
        "LDEL a, (#0, p)",
        "LOAD r1, (#0, s)+",
        "STOR -(#0, s), r1",
    ] {
        let (output, _) = assemble(directory.path(), "legal", source);
        assert_success(&output);
    }
}
