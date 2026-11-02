use super::assembler::support::assemble_object;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use tempfile::tempdir;
use toolchain::parsers::opcodes::ldel::ldel;

fn assemble(directory: &Path, name: &str, source: &str) -> (Output, PathBuf) {
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

#[test]
fn duplicate_label_cli_errors_do_not_write_output() {
    assert_assembly_error(":same\nNOOP\n:same\n", "Duplicate label [same]", "3:1");
    let directory = tempdir().unwrap();
    let (output, first) = assemble(directory.path(), "first", ":same\nNOOP\n");
    assert_success(&output);
    let (output, second) = assemble(directory.path(), "second", ":same\nNOOP\n");
    assert_success(&output);
    let binary = directory.path().join("duplicate.bin");
    let output = Command::new(env!("CARGO_BIN_EXE_linker"))
        .arg(first)
        .arg(second)
        .arg("-o")
        .arg(&binary)
        .args(["-s", "100"])
        .output()
        .unwrap();
    let message = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(message.contains("Duplicate label [same]"));
    assert!(message.contains("first.sasm") && message.contains("second.sasm"));
    assert!(!message.contains("panicked"));
    assert!(!binary.exists());
}

#[test]
fn bad_constants_report_original_source_locations() {
    assert_assembly_error(".EQU $N #16\nSHFT r1, LSL $N\n", "range of 0-15", "2:");
    assert_assembly_error(".EQU $N #-1\nADDI r1, $N, LSL #1\n", "8 bits", "2:");
    assert_assembly_error(".EQU $N #256\nEXCP $N\n", "EXCP vector", "2:");
    assert_assembly_error(".EQU $N #65536\nADDI r1, $N\n", "16-bit immediate", "2:");
    assert_assembly_error(
        "ADDI r1, $LATER\n.EQU $LATER #1\n",
        "Undefined constant $LATER",
        "1:10",
    );
    assert_assembly_error(".EQU $N #2\n.ORG $N\n", "parsing", "2:");
}

#[test]
fn ldel_link_source_is_an_intentional_aliased_write_error() {
    for offset in ["#1, ", "@target, ", "$N, ", "r2, ", ""] {
        let instruction = format!("LDEL a, ({offset}l)+\n");
        let error = ldel(&instruction).unwrap_err().to_string();
        assert!(error.contains("Aliased register writes"), "{error}");
        assert_assembly_error(
            &format!(".EQU $N #1\n{instruction}:target\nNOOP\n"),
            "Aliased register writes",
            "2:",
        );
        let legal = format!(".EQU $N #1\nLDEL a, ({offset}s)+\n:target\nNOOP\n");
        let _ = assemble_object(&legal);
    }
    for instruction in ["LDEL l, (#1, s)", "LDEL a, (#1, a)+", "LDEL a, (#1, p)+"] {
        assert_assembly_error(instruction, "Aliased register writes", "1:");
    }
}

#[test]
fn undefined_data_constant_does_not_write_an_object() {
    assert_assembly_error(".DB $MISSING\n", "Undefined constant", "1:5");
}

#[test]
fn linker_cli_writes_the_relocated_binary() {
    let directory = tempdir().unwrap();
    let (output, object) = assemble(
        directory.path(),
        "input",
        "MOVI r1, @target.l\n:target\nNOOP\n",
    );
    assert_success(&output);
    let binary = directory.path().join("program.bin");
    let output = Command::new(env!("CARGO_BIN_EXE_linker"))
        .arg(object)
        .arg("-o")
        .arg(&binary)
        .args(["-s", "100"])
        .output()
        .unwrap();
    assert_success(&output);
    assert_eq!(
        fs::read(binary).unwrap(),
        assemble_object("MOVI r1, #102\nNOOP\n").program
    );
}
