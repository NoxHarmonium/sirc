use clap::Parser;
use nom::branch::alt;
use nom::bytes::complete::{is_a, tag, take_while1};
use nom::character::complete::{line_ending, space0};
use nom::character::streaming::space1;
use nom::combinator::eof;
use nom::error::ErrorKind;
use nom::multi::many1;
use nom::sequence::{pair, separated_pair, terminated};
use nom::Err;
use nom::IResult;
use shared::instructions::{
    encode_null_instruction, encode_reg_reg_instruction, encode_reg_val_instruction,
    encode_val_instruction,
};
use shared::registers::register_name_to_index;
use std::fs::{read_to_string, write};
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, value_name = "FILE")]
    input_file: PathBuf,

    #[clap(short, long, value_parser, value_name = "FILE")]
    output_file: PathBuf,
}

// Instruction formats:
//
// Register Value: (e.g. SET)
// 8 bit register identifier
// 16 bit value
//
// Dual Register: (e.g. COPY)
// 8 bit register identifier
// 8 bit register identifier
// 8 bit padding
//
// Single Value: (e.g. JUMP)
// 16 bit value
// 8 bit padding

// TODO: Should this be in the shared crate?
// OR should these refer to some constant in the shared crate?
// what happens if the order changes or things are added?
fn register_tag_to_id(register_tag: &str) -> u8 {
    match register_tag {
        "HALT" => 0,
        "SET" => 1,
        "COPY" => 2,
        "ADD" => 3,
        "SUBTRACT" => 4,
        "MULTIPLY" => 5,
        "DIVIDE" => 6,
        "CEQ" => 7,
        "CNEQ" => 8,
        "CLT" => 9,
        "CGT" => 10,
        "CLTE" => 11,
        "CGTE" => 12,
        "JUMP" => 13,
        "JUMPIF" => 14,
        "JUMPIFNOT" => 15,
        // TODO: Better error handling? Compile time checking?
        _ => panic!("Instruction not mapped: {}", register_tag),
    }
}

pub fn parse_register(i: &str) -> IResult<&str, &str> {
    alt((
        tag("x1"),
        tag("y1"),
        tag("z1"),
        tag("x2"),
        tag("y2"),
        tag("z2"),
        tag("x3"),
        tag("y3"),
        tag("z3"),
        tag("a1"),
        tag("a2"),
        tag("a3"),
        tag("pc"),
        tag("sr"),
        tag("sp"),
    ))(i)
}

pub fn parse_hex(i: &str) -> IResult<&str, u16> {
    let (i, _) = tag("0x")(i)?;
    let (i, raw_digits) = is_a(&b"0123456789abcdefABCDEF"[..])(i)?;
    let hex_parse_result = u16::from_str_radix(raw_digits, 16);
    match hex_parse_result {
        Ok(hex_value) => Ok((i, hex_value)),
        Err(_) => Err(Err::Error(nom::error::Error {
            input: i,
            code: ErrorKind::Fail,
        })),
    }
}

pub fn parse_dec(i: &str) -> IResult<&str, u16> {
    let (i, raw_digits) = is_a(&b"0123456789"[..])(i)?;
    let dec_parse_result = u16::from_str_radix(raw_digits, 10);
    match dec_parse_result {
        Ok(dec_value) => Ok((i, dec_value)),
        Err(_) => Err(Err::Error(nom::error::Error {
            input: i,
            code: ErrorKind::Fail,
        })),
    }
}

pub fn parse_number(i: &str) -> IResult<&str, u16> {
    alt((parse_hex, parse_dec))(i)
}

pub fn parse_comma_sep(i: &str) -> IResult<&str, ()> {
    let (i, (_, _)) = pair(tag(","), space0)(i)?;
    Ok((i, ()))
}

pub fn is_upper_alphabetic(chr: char) -> bool {
    chr.is_ascii_uppercase()
}

fn parse_instruction_tag(i: &str) -> IResult<&str, u8> {
    let (i, instruction_tag) = take_while1(is_upper_alphabetic)(i)?;
    Ok((i, register_tag_to_id(instruction_tag)))
}

fn parse_rv_instruction(i: &str) -> IResult<&str, [u16; 2]> {
    let (i, instruction_id) = parse_instruction_tag(i)?;
    let (i, _) = space1(i)?;
    let (i, (r1, value)) = separated_pair(parse_register, parse_comma_sep, parse_number)(i)?;
    Ok((
        i,
        encode_reg_val_instruction(instruction_id, register_name_to_index(r1), value as u16),
    ))
}

fn parse_rr_instruction(i: &str) -> IResult<&str, [u16; 2]> {
    let (i, instruction_id) = parse_instruction_tag(i)?;
    let (i, _) = space1(i)?;
    let (i, (r1, r2)) = separated_pair(parse_register, parse_comma_sep, parse_register)(i)?;
    Ok((
        i,
        encode_reg_reg_instruction(
            instruction_id,
            register_name_to_index(r1),
            register_name_to_index(r2),
        ),
    ))
}

fn parse_v_instruction(i: &str) -> IResult<&str, [u16; 2]> {
    let (i, instruction_id) = parse_instruction_tag(i)?;
    let (i, _) = space1(i)?;
    let (i, value) = parse_number(i)?;
    Ok((i, encode_val_instruction(instruction_id, value)))
}

fn parse_null_instruction(i: &str) -> IResult<&str, [u16; 2]> {
    let (i, instruction_id) = parse_instruction_tag(i)?;
    Ok((i, encode_null_instruction(instruction_id)))
}

// TODO: Validate that the instruction matches the instruction type
// E.G. HALT can't have args and JUMPIF can only have a value
// Support new line at end of file (LOL) lexeme?
fn parse_instruction(i: &str) -> IResult<&str, [u16; 2]> {
    alt((
        parse_rv_instruction,
        parse_rr_instruction,
        parse_v_instruction,
        parse_null_instruction,
    ))(i)
}

fn parse_instructions(i: &str) -> IResult<&str, Vec<u8>> {
    let (i, parsed_instructions) = many1(terminated(parse_instruction, line_ending))(i)?;
    let (i, _) = eof(i)?;
    let bytes: Vec<u8> = parsed_instructions
        .iter()
        .flat_map(|[b1, b2]| [u16::to_le_bytes(*b1), u16::to_le_bytes(*b2)])
        .flatten()
        .collect();
    Ok((i, bytes))
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let file_contents = read_to_string(args.input_file);

    match file_contents {
        Ok(contents) => {
            let parse_result = parse_instructions(contents.as_str());

            match parse_result {
                Ok((_, parsed_instructions)) => {
                    write(args.output_file, parsed_instructions)?;
                }
                Err(error) => {
                    panic!("Error during assembly: {}", error)
                }
            }
        }
        Err(err) => return Err(err),
    }

    Ok(())
}
