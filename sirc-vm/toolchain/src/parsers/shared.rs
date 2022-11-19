use nom::branch::alt;
use nom::bytes::complete::{is_a, tag};
use nom::character::complete::{alphanumeric1, digit1, multispace0, one_of, space0};
use nom::combinator::{map, map_res, opt, recognize};
use nom::error::{ErrorKind, ParseError};
use nom::sequence::{pair, terminated, tuple};
use nom::Err;
use nom::IResult;

use crate::types::object::RefType;

use super::instruction::RefToken;

pub fn lexeme<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    terminated(inner, multispace0)
}

fn parse_hex_(i: &str) -> IResult<&str, u16> {
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

fn parse_dec_(i: &str) -> IResult<&str, u16> {
    map_res(
        tuple((opt(one_of("+-")), recognize(digit1))),
        |(sign, number_string)| {
            if sign.is_some() {
                // Signed numbers represented in parser as unsigned for simplicity
                str::parse::<i16>(number_string).map(|signed| signed as u16)
            } else {
                str::parse::<u16>(number_string)
            }
        },
    )(i)
}

fn parse_number_(i: &str) -> IResult<&str, u16> {
    alt((parse_hex, parse_dec))(i)
}

fn parse_comma_sep_(i: &str) -> IResult<&str, ()> {
    let (i, (_, _)) = pair(tag(","), space0)(i)?;
    Ok((i, ()))
}

fn parse_range_sep_(i: &str) -> IResult<&str, ()> {
    let (i, (_, _)) = pair(tag("->"), space0)(i)?;
    Ok((i, ()))
}

pub fn parse_label_(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(":")(i)?;
    alphanumeric1(i)
}

pub fn parse_symbol_reference_postamble_(i: &str) -> IResult<&str, Option<RefType>> {
    let (i, dot_parsed) = opt(tag("."))(i)?;

    match dot_parsed {
        Some(_) => map(
            alt((tag(".r"), tag(".u"), tag(".l"))),
            |parsed_value| match parsed_value {
                ".r" => Some(RefType::Offset),
                ".u" => Some(RefType::UpperByte),
                ".l" => Some(RefType::LowerByte),
                // TODO: Proper error handling again
                _ => panic!("Unknown postamble {}", parsed_value),
            },
        )(i),
        None => Ok((i, None)),
    }
}

pub fn parse_symbol_reference_(i: &str) -> IResult<&str, RefToken> {
    let (i, _) = tag("@")(i)?;
    let (i, name) = alphanumeric1(i)?;
    let (i, optional_preamble) = parse_symbol_reference_postamble_(i)?;

    Ok((
        i,
        RefToken {
            name: String::from(name),
            ref_type: optional_preamble.unwrap_or(RefType::Implied),
        },
    ))
}

pub fn parse_hex(i: &str) -> IResult<&str, u16> {
    lexeme(parse_hex_)(i)
}

pub fn parse_dec(i: &str) -> IResult<&str, u16> {
    lexeme(parse_dec_)(i)
}

pub fn parse_number(i: &str) -> IResult<&str, u16> {
    lexeme(parse_number_)(i)
}

pub fn parse_comma_sep(i: &str) -> IResult<&str, ()> {
    lexeme(parse_comma_sep_)(i)
}

pub fn parse_range_sep(i: &str) -> IResult<&str, ()> {
    lexeme(parse_range_sep_)(i)
}

pub fn parse_label(i: &str) -> IResult<&str, &str> {
    lexeme(parse_label_)(i)
}

pub fn parse_symbol_reference(i: &str) -> IResult<&str, RefToken> {
    lexeme(parse_symbol_reference_)(i)
}
