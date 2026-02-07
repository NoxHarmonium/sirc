use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not};
use nom::character::complete::{char, multispace0, one_of, space0};
use nom::combinator::{cut, eof, map, map_res, opt, recognize};
use nom::error::{ErrorKind, ParseError};
use nom::sequence::{pair, preceded, terminated, tuple};
use nom::{AsChar, IResult};
use nom::{Err, InputTakeAtPosition, Parser};
use nom_supreme::error::ErrorTree;
use nom_supreme::error::{BaseErrorKind, Expectation};
use nom_supreme::multi::collect_separated_terminated;
use nom_supreme::tag::complete::tag;
use nom_supreme::ParserExt;

use super::instruction::{parse_instruction_token, ShiftDefinitionData};
use crate::parsers::data::{parse_data_token, parse_equ_token};
use crate::types::data::RefToken;
use crate::types::object::RefType;
use crate::types::shared::{
    LabelToken, NumberToken, NumberType, OriginToken, Token, REF_TOKEN_LOWER_WORD_SUFFIX,
    REF_TOKEN_OFFSET_SUFFIX, REF_TOKEN_UPPER_WORD_SUFFIX,
};
use peripheral_cpu::coprocessors::processing_unit::definitions::{ShiftOperand, ShiftType};

pub type AsmResult<'a, 'b, O> = IResult<&'a str, O, ErrorTree<&'b str>>;

#[allow(clippy::type_repetition_in_bounds)]
pub fn lexeme<'a, F, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E> + 'a,
{
    terminated(inner, multispace0)
}

fn parse_label_name_(i: &str) -> AsmResult<&str> {
    i.split_at_position1_complete(
        |item| !item.is_alphanum() && item != '_',
        ErrorKind::AlphaNumeric,
    )
}

fn parse_bin_<T: num_traits::Num>(i: &str) -> AsmResult<T> {
    let (i, _) = tag("0b")(i)?;
    let (i, raw_digits) = is_a(&b"01_"[..])(i)?;
    let digits_without_underscores = raw_digits.replace('_', "");
    let bin_parse_result = T::from_str_radix(&digits_without_underscores, 2);
    bin_parse_result.map_or_else(
        |_| {
            Err(Err::Error(ErrorTree::Base {
                location: i,
                // TODO: Should we use a custom error here?
                kind: BaseErrorKind::Expected(Expectation::Tag("binary digit")),
            }))
        },
        |binary_value| Ok((i, binary_value)),
    )
}

fn parse_hex_<T: num_traits::Num>(i: &str) -> AsmResult<T> {
    let (i, _) = tag("0x")(i)?;
    let (i, raw_digits) = is_a(&b"0123456789abcdefABCDEF_"[..])(i)?;
    let digits_without_underscores = raw_digits.replace('_', "");
    let hex_parse_result = T::from_str_radix(&digits_without_underscores, 16);
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

#[allow(clippy::cast_sign_loss)]
fn parse_dec_(i: &str) -> AsmResult<u32> {
    map_res(
        tuple((opt(one_of("+-")), recognize(is_a("0123456789_")))),
        |(sign, number_string): (Option<char>, &str)| {
            let digits_without_underscores = number_string.replace('_', "");
            sign.map_or_else(
                || str::parse::<u32>(&digits_without_underscores),
                |sign_value| {
                    // TODO: Fix strangeness in number parsing
                    // category=Toolchain
                    // Re-concatenating the original string seems bad
                    // We should probably just get the original value or something
                    let full_number = format!("{sign_value}{digits_without_underscores}");
                    // Signed numbers represented in parser as unsigned for simplicity
                    str::parse::<i32>(full_number.as_str()).map(|signed| signed as u32)
                },
            )
        },
    )(i)
}

fn parse_number_(i: &str) -> AsmResult<NumberToken> {
    preceded(char('#'), alt((parse_hex, parse_bin, parse_dec)))(i)
}

fn parse_placeholder_(i: &str) -> AsmResult<String> {
    map(preceded(char('$'), parse_label_name_), ToOwned::to_owned)(i)
}

fn parse_comma_sep_(i: &str) -> AsmResult<()> {
    let (i, (_, _)) = pair(char(','), space0)(i)?;
    Ok((i, ()))
}

pub fn parse_label_(i: &str) -> AsmResult<&str> {
    preceded(char(':'), cut(parse_label_name_))(i)
}

pub fn parse_origin_(i: &str) -> AsmResult<NumberToken> {
    let (i, (_, value)) = tuple((lexeme(tag(".ORG")), alt((parse_hex, parse_dec))))(i)?;

    Ok((i, value))
}

pub fn parse_symbol_reference_postamble_(i: &str) -> AsmResult<Option<RefType>> {
    opt(map(
        alt((
            tag(REF_TOKEN_OFFSET_SUFFIX),
            tag(REF_TOKEN_UPPER_WORD_SUFFIX),
            tag(REF_TOKEN_LOWER_WORD_SUFFIX),
        )),
        |parsed_value| match parsed_value {
            REF_TOKEN_OFFSET_SUFFIX => RefType::Offset,
            REF_TOKEN_UPPER_WORD_SUFFIX => RefType::UpperWord,
            REF_TOKEN_LOWER_WORD_SUFFIX => RefType::LowerWord,
            _ => panic!("Unknown postamble {parsed_value}"),
        },
    ))(i)
}

pub fn parse_symbol_reference_(i: &str) -> AsmResult<RefToken> {
    let (i, name) = preceded(char('@'), parse_label_name_)(i)?;

    let (i, optional_postamble) = parse_symbol_reference_postamble_(i)?;

    Ok((
        i,
        RefToken {
            name: String::from(name),
            ref_type: optional_postamble.unwrap_or(RefType::Implied),
        },
    ))
}

pub fn parse_bin(i: &str) -> AsmResult<NumberToken> {
    map(lexeme(parse_bin_), |value| NumberToken {
        value,
        number_type: NumberType::Binary,
    })(i)
}

pub fn parse_hex(i: &str) -> AsmResult<NumberToken> {
    map(lexeme(parse_hex_), |value| NumberToken {
        value,
        number_type: NumberType::Hex,
    })(i)
}

pub fn parse_dec(i: &str) -> AsmResult<NumberToken> {
    map(lexeme(parse_dec_), |value| NumberToken {
        value,
        number_type: NumberType::Decimal,
    })(i)
}

pub fn parse_number(i: &str) -> AsmResult<NumberToken> {
    lexeme(parse_number_)(i)
}

pub fn parse_placeholder(i: &str) -> AsmResult<String> {
    lexeme(parse_placeholder_)(i)
}

pub fn parse_comma_sep(i: &str) -> AsmResult<()> {
    lexeme(parse_comma_sep_)(i)
}

pub fn parse_label(i: &str) -> AsmResult<&str> {
    lexeme(parse_label_)(i)
}

pub fn parse_origin(i: &str) -> AsmResult<u32> {
    map(lexeme(parse_origin_), |token| token.value)(i)
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
// TODO: Consider storing shift as an enum
// category=Refactoring
// Not sure if still valid, needs looking in to
// Should shift be stored as an enum in the instruction structs? then it could be reused by the parser and avoid this function
pub fn split_shift_definition_data(
    shift_definition_data: &ShiftDefinitionData,
) -> (ShiftOperand, ShiftType, u8) {
    match shift_definition_data {
        // TODO: More clean up in shared parser code
        // category=Refactoring
        // Probably can avoid this wrapping/unwrapping by using one type or something?
        ShiftDefinitionData::Immediate(shift_type, shift_count) => {
            (ShiftOperand::Immediate, *shift_type, *shift_count)
        }
        ShiftDefinitionData::Register(shift_type, shift_count) => (
            ShiftOperand::Register,
            *shift_type,
            shift_count.to_register_index(),
        ),
    }
}

fn parse_label_token(i: &str) -> AsmResult<Token> {
    let (i, name) = parse_label(i)?;
    Ok((
        i,
        Token::Label(LabelToken {
            name: String::from(name),
        }),
    ))
}

fn parse_origin_token(i: &str) -> AsmResult<Token> {
    let (i, offset) = parse_origin(i)?;
    Ok((i, Token::Origin(OriginToken { offset })))
}

fn parse_comment_(i: &str) -> AsmResult<Token> {
    // TODO: Should there be a more flexible parser for eol?
    // category=Toolchain
    // TODO: Comments with nothing after the semicolon currently fail
    // category=Toolchain
    // TODO: Comments with a semi-colon in the body  them seem to fail
    map(pair(char(';'), cut(is_not("\n\r"))), |(_, text)| {
        Token::Comment(String::from(text))
    })(i)
}

fn parse_comment(i: &str) -> AsmResult<Token> {
    lexeme(parse_comment_)(i)
}

// Addresses are replaced with indexes to object table and resolved by linker
pub fn parse_tokens(i: &str) -> AsmResult<Vec<Token>> {
    let mut parser = collect_separated_terminated(
        alt((
            parse_comment.context("comment"),
            parse_instruction_token.context("instruction"),
            parse_label_token.context("label"),
            parse_origin_token.context("origin"),
            parse_data_token.context("data directive"),
            parse_equ_token.context("equ directive"),
        )),
        multispace0,
        eof,
    );

    // Consume any extra space at the start
    let (i, _) = multispace0(i)?;
    parser.parse(i)
}
