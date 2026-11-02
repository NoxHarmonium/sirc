pub mod arithmetic_immediate;
pub mod arithmetic_register;
pub mod branching;
pub mod coprocessor;
pub mod coprocessor_meta;
pub mod exception;
pub mod ldea;
pub mod ldel;
pub mod ljmp;
pub mod ljsr;
pub mod load;
pub mod meta;
pub mod mov;
pub mod store;

use nom::error::{ErrorKind, FromExternalError};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::registers::{AddressRegisterName, RegisterName};

fn address_register_halves(address_register: &AddressRegisterName) -> (RegisterName, RegisterName) {
    match address_register {
        AddressRegisterName::LinkRegister => (RegisterName::Lh, RegisterName::Ll),
        AddressRegisterName::Address => (RegisterName::Ah, RegisterName::Al),
        AddressRegisterName::StackPointer => (RegisterName::Sh, RegisterName::Sl),
        AddressRegisterName::ProgramCounter => (RegisterName::Ph, RegisterName::Pl),
    }
}

fn direct_register_aliases_address_register(
    register: &RegisterName,
    address_register: &AddressRegisterName,
) -> bool {
    let (high_register, low_register) = address_register_halves(address_register);
    *register == high_register || *register == low_register
}

fn reject_aliased_address_register_write<'a>(
    input: &'a str,
    instruction: &str,
    details: &str,
) -> Result<(), nom::Err<ErrorTree<&'a str>>> {
    let error_string =
        format!("Aliased register writes are undefined for {instruction}: {details}");
    Err(nom::Err::Failure(ErrorTree::from_external_error(
        input,
        ErrorKind::Fail,
        error_string.as_str(),
    )))
}

/// Bind an immediate operand to its encoding field. Constants keep that field's
/// validation and original source context in the AST until object generation.
fn numeric_field<'a>(
    input: &'a str,
    operand: &crate::parsers::instruction::ImmediateType,
    field: crate::types::numeric::NumericField,
) -> Result<(u16, Vec<crate::types::numeric::NumericFixup>), nom::Err<ErrorTree<&'a str>>> {
    use crate::{
        parsers::instruction::ImmediateType,
        types::{
            numeric::{NumericField, NumericFixup},
            shared::{NumberToken, NumberType},
        },
    };
    match operand {
        ImmediateType::PlaceHolder(reference) => {
            Ok((0, vec![NumericFixup::new(reference.clone(), field)]))
        }
        ImmediateType::Value(value) => {
            let number = if matches!(field, NumericField::DmaRegisterCount) {
                NumberToken {
                    value: i32::from(value.cast_signed()).cast_unsigned(),
                    number_type: NumberType::SignedDecimal,
                }
            } else {
                NumberToken {
                    value: u32::from(*value),
                    number_type: NumberType::Hex,
                }
            };
            field
                .encode(&number)
                .map(|encoded| (encoded, Vec::new()))
                .map_err(|message| {
                    nom::Err::Failure(ErrorTree::from_external_error(
                        input,
                        ErrorKind::Verify,
                        message,
                    ))
                })
        }
        ImmediateType::SymbolRef(_) => Err(nom::Err::Failure(ErrorTree::from_external_error(
            input,
            ErrorKind::Verify,
            "Expected a number or named constant, not a label relocation",
        ))),
    }
}
