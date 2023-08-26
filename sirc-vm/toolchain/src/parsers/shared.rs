use nom::branch::alt;
use nom::bytes::complete::is_a;
use nom::character::complete::{char, digit1, one_of, space0};
use nom::combinator::{cut, map, map_res, opt, recognize};
use nom::error::{ErrorKind, ParseError};
use nom::sequence::{pair, preceded, terminated, tuple};
use nom::{AsChar, IResult};
use nom::{Err, InputTakeAtPosition};
use nom_supreme::error::ErrorTree;
use nom_supreme::error::{BaseErrorKind, Expectation};
use nom_supreme::tag::complete::tag;
use peripheral_cpu::coprocessors::processing_unit::definitions::{ShiftOperand, ShiftType};

use crate::types::object::RefType;

use super::instruction::{RefToken, ShiftDefinitionData};

pub type AsmResult<'a, 'b, O> = IResult<&'a str, O, ErrorTree<&'b str>>;

#[allow(clippy::type_repetition_in_bounds)]
pub fn lexeme<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    terminated(inner, space0)
}

fn parse_label_name_(i: &str) -> AsmResult<&str> {
    i.split_at_position1_complete(
        |item| !item.is_alphanum() && item != '_',
        ErrorKind::AlphaNumeric,
    )
}

fn parse_hex_<T: num_traits::Num>(i: &str) -> AsmResult<T> {
    let (i, _) = tag("0x")(i)?;
    let (i, raw_digits) = is_a(&b"0123456789abcdefABCDEF"[..])(i)?;
    let hex_parse_result = T::from_str_radix(raw_digits, 16);
    hex_parse_result.map_or_else(
        |_| {
            Err(Err::Error(ErrorTree::Base {
                location: i,
                kind: BaseErrorKind::Expected(Expectation::HexDigit),
            }))
        },
        |hex_value| Ok((i, hex_value)),
    )
}

// TODO: Allow u32 decimal
#[allow(clippy::cast_sign_loss)]
fn parse_dec_(i: &str) -> AsmResult<u32> {
    map_res(
        tuple((opt(one_of("+-")), recognize(digit1))),
        |(sign, number_string)| {
            sign.map_or_else(
                || str::parse::<u32>(number_string),
                |sign_value| {
                    // TODO: Re-concatenating the original string seems bad
                    // We should probably just get the original value or something
                    let full_number = format!("{sign_value}{number_string}");
                    // Signed numbers represented in parser as unsigned for simplicity
                    str::parse::<i32>(full_number.as_str()).map(|signed| signed as u32)
                },
            )
        },
    )(i)
}

fn parse_number_(i: &str) -> AsmResult<u32> {
    preceded(char('#'), alt((parse_hex, parse_dec)))(i)
}

fn parse_comma_sep_(i: &str) -> AsmResult<()> {
    let (i, (_, _)) = pair(char(','), space0)(i)?;
    Ok((i, ()))
}

pub fn parse_label_(i: &str) -> AsmResult<&str> {
    preceded(char(':'), cut(parse_label_name_))(i)
}

pub fn parse_origin_(i: &str) -> AsmResult<u32> {
    let (i, (_, value)) = tuple((lexeme(tag(".ORG")), alt((parse_hex, parse_dec))))(i)?;

    Ok((i, value))
}

pub fn parse_symbol_reference_postamble_(i: &str) -> AsmResult<Option<RefType>> {
    let (i, dot_parsed) = opt(tag("."))(i)?;

    match dot_parsed {
        Some(_) => map(
            alt((tag(".r"), tag(".u"), tag(".l"))),
            |parsed_value| match parsed_value {
                ".r" => Some(RefType::Offset),
                ".u" => Some(RefType::UpperWord),
                ".l" => Some(RefType::LowerWord),
                // TODO: Proper error handling again
                _ => panic!("Unknown postamble {parsed_value}"),
            },
        )(i),
        None => Ok((i, None)),
    }
}

pub fn parse_symbol_reference_(i: &str) -> AsmResult<RefToken> {
    let (i, name) = preceded(char('@'), parse_label_name_)(i)?;

    let (i, optional_preamble) = parse_symbol_reference_postamble_(i)?;

    Ok((
        i,
        RefToken {
            name: String::from(name),
            ref_type: optional_preamble.unwrap_or(RefType::Implied),
        },
    ))
}

pub fn parse_hex(i: &str) -> AsmResult<u32> {
    lexeme(parse_hex_)(i)
}

pub fn parse_dec(i: &str) -> AsmResult<u32> {
    lexeme(parse_dec_)(i)
}

pub fn parse_number(i: &str) -> AsmResult<u32> {
    lexeme(parse_number_)(i)
}

pub fn parse_comma_sep(i: &str) -> AsmResult<()> {
    lexeme(parse_comma_sep_)(i)
}

pub fn parse_label(i: &str) -> AsmResult<&str> {
    lexeme(parse_label_)(i)
}

pub fn parse_origin(i: &str) -> AsmResult<u32> {
    lexeme(parse_origin_)(i)
}

pub fn parse_symbol_reference(i: &str) -> AsmResult<RefToken> {
    lexeme(parse_symbol_reference_)(i)
}

///
/// Takes the parser representation of a shift and splits it out into the components
/// that get encoded into instructions.
///
/// In this case of instructions, the "`shift_count`" is either the index of the register,
/// or a constant depending on the value of `ShiftOperand`.
///
/// TODO: Should shift be stored as an enum in the instruction structs? then it could be reused
/// by the parser and avoid this function
///
pub fn split_shift_definition_data(
    shift_definition_data: &ShiftDefinitionData,
) -> (ShiftOperand, ShiftType, u8) {
    match shift_definition_data {
        // TODO: Probably can avoid this wrapping/unwrapping by using one type or something?
        crate::parsers::instruction::ShiftDefinitionData::Immediate(shift_type, shift_count) => {
            (ShiftOperand::Immediate, *shift_type, *shift_count)
        }
        crate::parsers::instruction::ShiftDefinitionData::Register(shift_type, shift_count) => (
            ShiftOperand::Register,
            *shift_type,
            shift_count.to_register_index(),
        ),
    }
}
