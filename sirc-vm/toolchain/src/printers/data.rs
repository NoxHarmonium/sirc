use crate::printers::shared::{print_number_token, print_ref_token};
use crate::types::data::{
    DataToken, DataType, EquToken, DB_TOKEN, DB_VALUE, DQ_TOKEN, DQ_VALUE, DW_TOKEN, DW_VALUE,
    EQU_TOKEN,
};
use crate::types::object::RefType;

/// Prints the AST representation of a `DataToken` to a string
///
///```
/// use toolchain::printers::data::print_data_token;
/// use toolchain::types::data::{DataToken, DataType, RefToken};
/// use toolchain::types::object::{RefType, SymbolRef};
/// use toolchain::types::shared::{NumberToken, NumberType};
/// let printed = print_data_token(&DataToken {
///    size_bytes: 4,
///    value: DataType::SymbolRef(RefToken {
///        name: String::from("some_symbol"),
///        ref_type: RefType::FullAddress,
///    })
/// });
/// assert_eq!(String::from(".DQ @some_symbol"), printed);
/// ```
pub fn print_data_token(data_token: &DataToken) -> String {
    let token_string = match data_token.size_bytes {
        DB_VALUE => DB_TOKEN,
        DW_VALUE => DW_TOKEN,
        DQ_VALUE => DQ_TOKEN,
        _ => panic!(
            "Only 1 (byte), 2 (word) or 4 (quad) data sizes are supported. Got [{}]",
            data_token.size_bytes
        ),
    };

    let value_string = match &data_token.value {
        DataType::Value(value) => print_number_token(value),
        DataType::SymbolRef(ref_token) => match ref_token.ref_type {
            RefType::FullAddress => print_ref_token(ref_token),
            _ => panic!(
                "Only FullAddress reference type is valid for data tokens. Got [{:?}]",
                ref_token.ref_type
            ),
        },
        DataType::PlaceHolder(placeholder_name) => format!("${placeholder_name}"),
    };

    format!("{token_string} {value_string}")
}

/// Prints the AST representation of an `EquToken` to a string
///
///```
/// use toolchain::printers::data::print_equ_token;
/// use toolchain::types::data::EquToken;
/// use toolchain::types::shared::{NumberToken, NumberType};
/// let printed = print_equ_token(&EquToken {
///    placeholder_name: String::from("FOO"),
///    number_token: NumberToken {
///        number_type: NumberType::Hex,
///        value: 0xCAFE
///    }
/// });
/// assert_eq!(String::from(".EQU $FOO #0xCAFE"), printed);
/// ```
pub fn print_equ_token(equ_token: &EquToken) -> String {
    format!(
        "{EQU_TOKEN} ${} {}",
        equ_token.placeholder_name,
        print_number_token(&equ_token.number_token)
    )
}
