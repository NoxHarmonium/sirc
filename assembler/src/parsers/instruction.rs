use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::space0;
use nom::character::streaming::space1;
use nom::combinator::{eof, map};
use nom::multi::many1;
use nom::sequence::{separated_pair, terminated};
use nom::IResult;
use peripheral_cpu::instructions::{
    encode_null_instruction, encode_reg_reg_instruction, encode_reg_val_instruction,
    encode_val_instruction,
};
use peripheral_cpu::registers::register_name_to_index;

use super::shared::{lexeme, parse_comma_sep, parse_number};

// OR should these refer to some constant in the shared crate?
// what happens if the order changes or things are added?
// TODO: use a function to build this up
pub fn parse_instruction_tag(i: &str) -> IResult<&str, u8> {
    alt((
        map(terminated(tag("HALT"), space0), |_| 0x00),
        map(terminated(tag("SET"), space1), |_| 0x01),
        map(terminated(tag("COPY"), space1), |_| 0x02),
        map(terminated(tag("ADD"), space1), |_| 0x03),
        map(terminated(tag("SUBTRACT"), space1), |_| 0x04),
        map(terminated(tag("MULTIPLY"), space1), |_| 0x05),
        map(terminated(tag("DIVIDE"), space1), |_| 0x06),
        map(terminated(tag("CEQ"), space1), |_| 0x07),
        map(terminated(tag("CNEQ"), space1), |_| 0x08),
        map(terminated(tag("CLT"), space1), |_| 0x09),
        map(terminated(tag("CGT"), space1), |_| 0x0A),
        map(terminated(tag("CLTE"), space1), |_| 0x0B),
        map(terminated(tag("CGTE"), space1), |_| 0x0C),
        map(terminated(tag("JUMP"), space1), |_| 0x0D),
        map(terminated(tag("JUMPIF"), space1), |_| 0x0E),
        map(terminated(tag("JUMPIFNOT"), space1), |_| 0x0F),
    ))(i)
}

fn parse_register_(i: &str) -> IResult<&str, &str> {
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

fn parse_register(i: &str) -> IResult<&str, &str> {
    lexeme(parse_register_)(i)
}

fn parse_rv_instruction(i: &str) -> IResult<&str, [u16; 2]> {
    let (i, instruction_id) = parse_instruction_tag(i)?;
    let (i, (r1, value)) = separated_pair(parse_register, parse_comma_sep, parse_number)(i)?;
    Ok((
        i,
        encode_reg_val_instruction(instruction_id, register_name_to_index(r1), value as u16),
    ))
}

fn parse_rr_instruction(i: &str) -> IResult<&str, [u16; 2]> {
    let (i, instruction_id) = parse_instruction_tag(i)?;
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
fn parse_instruction_(i: &str) -> IResult<&str, [u16; 2]> {
    alt((
        parse_rv_instruction,
        parse_rr_instruction,
        parse_v_instruction,
        parse_null_instruction,
    ))(i)
}

fn parse_instruction(i: &str) -> IResult<&str, [u16; 2]> {
    lexeme(parse_instruction_)(i)
}

pub fn parse_instructions(i: &str) -> IResult<&str, Vec<u8>> {
    let (i, parsed_instructions) = many1(parse_instruction)(i)?;
    let (i, _) = eof(i)?;
    let bytes: Vec<u8> = parsed_instructions
        .iter()
        .flat_map(|[b1, b2]| [u16::to_le_bytes(*b1), u16::to_le_bytes(*b2)])
        .flatten()
        .collect();
    Ok((i, bytes))
}
