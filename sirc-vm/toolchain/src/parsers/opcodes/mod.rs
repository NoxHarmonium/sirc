pub mod arithmetic_immediate;
pub mod arithmetic_register;
pub mod branching;
pub mod exception;
pub mod ldea;
pub mod ldel;
pub mod ljmp;
pub mod ljsr;
pub mod load;
pub mod meta;
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
