use nom::{
    branch::alt,
    combinator::map,
    error::{ErrorKind, FromExternalError},
};
use nom_supreme::{error::ErrorTree, tag::complete::tag, ParserExt};

use super::{
    instruction::RefToken,
    shared::{lexeme, parse_number, parse_symbol_reference, AsmResult},
};

#[derive(Debug)]
pub enum DataType {
    Value(u32),
    SymbolRef(RefToken),
}

pub fn parse_data_type(i: &str) -> AsmResult<DataType> {
    alt((
        // TODO: Add hash before number!
        map(parse_number, DataType::Value).context("number"),
        map(parse_symbol_reference, |ref_token| {
            DataType::SymbolRef(ref_token)
        })
        .context("symbol reference"),
    ))(i)
}

pub fn parse_data_(i: &str) -> AsmResult<(u8, DataType)> {
    let (i, tag) = lexeme(alt((tag("DB"), tag("DW"), tag("DQ"))))(i)?;
    let (i, value) = parse_data_type(i)?;

    let size = match tag {
        "DB" => Ok(1),
        "DW" => Ok(2),
        "DQ" => Ok(4),
        _ => {
            let error_string = format!("Only DB (byte), DW (word) or DQ (quad) data directives are supported. Got [{tag}] ");
            Err(nom::Err::Error(ErrorTree::from_external_error(
                i,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }?;

    Ok((i, (size, value)))
}

pub fn parse_data(i: &str) -> AsmResult<(u8, DataType)> {
    lexeme(parse_data_)(i)
}
