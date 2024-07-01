use nom::{
    branch::alt,
    combinator::map,
    error::{ErrorKind, FromExternalError},
};
use nom_supreme::error::ErrorTree;
use nom_supreme::tag::complete::tag;
use nom_supreme::ParserExt;

use super::shared::{lexeme, parse_number, parse_placeholder, parse_symbol_reference, AsmResult};
use crate::types::data::{
    RefToken, DB_TOKEN, DB_VALUE, DQ_TOKEN, DQ_VALUE, DW_TOKEN, DW_VALUE, EQU_TOKEN,
};
use crate::types::shared::{NumberToken, Token};
use crate::types::{
    data::{DataToken, DataType, EquToken},
    object::RefType,
};

pub fn override_ref_token_type_if_implied(
    ref_token: &RefToken,
    override_ref_type: RefType,
) -> RefToken {
    match ref_token.ref_type {
        RefType::Implied => RefToken {
            name: ref_token.name.clone(),
            ref_type: override_ref_type,
        },
        _ => RefToken {
            name: ref_token.name.clone(),
            ref_type: ref_token.ref_type,
        },
    }
}

fn parse_data_type(i: &str) -> AsmResult<DataType> {
    alt((
        // TODO: Make sure that numbers are defined consistently in SIRC Asm
        // category=Toolchain
        // There should probably be a hash before the number here. (Also check .ORG)
        map(parse_number, DataType::Value).context("number"),
        map(parse_symbol_reference, |ref_token| {
            DataType::SymbolRef(ref_token)
        })
        .context("symbol reference"),
    ))(i)
}

fn parse_data_(i: &str) -> AsmResult<(u8, DataType)> {
    let (i, tag) = lexeme(alt((tag(DB_TOKEN), tag(DW_TOKEN), tag(DQ_TOKEN))))(i)?;
    let (i, value) = parse_data_type(i)?;

    let size = match tag {
        DB_TOKEN => Ok(DB_VALUE),
        DW_TOKEN => Ok(DW_VALUE),
        DQ_TOKEN => Ok(DQ_VALUE),
        _ => {
            let error_string = format!(
                "Only DB (byte), DW (word) or DQ (quad) data directives are supported. Got [{tag}]"
            );
            Err(nom::Err::Error(ErrorTree::from_external_error(
                i,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }?;

    Ok((i, (size, value)))
}

fn parse_data(i: &str) -> AsmResult<(u8, DataType)> {
    lexeme(parse_data_)(i)
}

fn parse_equ_(i: &str) -> AsmResult<(String, NumberToken)> {
    let (i, _) = lexeme(tag(EQU_TOKEN))(i)?;
    let (i, placeholder_name) = parse_placeholder(i)?;
    let (i, value) = parse_number(i)?;

    Ok((i, (placeholder_name, value)))
}

fn parse_equ(i: &str) -> AsmResult<(String, NumberToken)> {
    lexeme(parse_equ_)(i)
}

pub fn parse_data_token_(i: &str) -> AsmResult<Token> {
    let (i, (size_bytes, value)) = parse_data(i)?;

    let override_value = if let DataType::SymbolRef(ref_token) = value {
        DataType::SymbolRef(override_ref_token_type_if_implied(
            &ref_token,
            RefType::FullAddress,
        ))
    } else {
        value
    };

    Ok((
        i,
        Token::Data(DataToken {
            size_bytes,
            value: override_value,
        }),
    ))
}

pub fn parse_data_token(i: &str) -> AsmResult<Token> {
    lexeme(parse_data_token_)(i)
}

pub fn parse_equ_token_(i: &str) -> AsmResult<Token> {
    let (i, (placeholder_name, value)) = parse_equ(i)?;

    Ok((
        i,
        Token::Equ(EquToken {
            placeholder_name,
            number_token: value,
        }),
    ))
}

pub fn parse_equ_token(i: &str) -> AsmResult<Token> {
    lexeme(parse_equ_token_)(i)
}
