use nom::branch::alt;
use nom::bytes::complete::{is_a, tag};
use nom::character::complete::{alphanumeric1, multispace0, space0};
use nom::error::{ErrorKind, ParseError};
use nom::sequence::{pair, terminated};
use nom::Err;
use nom::IResult;

pub fn lexeme<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
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

fn parse_number_(i: &str) -> IResult<&str, u16> {
    alt((parse_hex, parse_dec))(i)
}

fn parse_comma_sep_(i: &str) -> IResult<&str, ()> {
    let (i, (_, _)) = pair(tag(","), space0)(i)?;
    Ok((i, ()))
}

pub fn parse_label_(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag(":")(i)?;
    alphanumeric1(i)
}

pub fn parse_symbol_reference_(i: &str) -> IResult<&str, &str> {
    let (i, _) = tag("@")(i)?;
    alphanumeric1(i)
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

pub fn parse_label(i: &str) -> IResult<&str, &str> {
    lexeme(parse_label_)(i)
}

pub fn parse_symbol_reference(i: &str) -> IResult<&str, &str> {
    lexeme(parse_symbol_reference_)(i)
}
