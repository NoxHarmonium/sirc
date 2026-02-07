use crate::printers::data::{print_data_token, print_equ_token};
use crate::types::data::RefToken;
use crate::types::object::RefType;
use crate::types::shared::{
    LabelToken, NumberToken, NumberType, Token, REF_TOKEN_LOWER_WORD_SUFFIX,
    REF_TOKEN_OFFSET_SUFFIX, REF_TOKEN_UPPER_WORD_SUFFIX,
};
use itertools::Itertools;

/// Adds underscore separators to a string representation of a number
/// Separators are inserted every 4 characters from the right
fn add_underscores(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();

    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i).is_multiple_of(4) {
            result.push('_');
        }
        result.push(*ch);
    }

    result
}

/// Prints the AST representation of a `NumberToken` to a string
///
///```
/// use toolchain::printers::shared::print_number_token;
/// use toolchain::types::shared::{NumberToken, NumberType};
/// let printed = print_number_token(&NumberToken {
///    value: 0xCAFE,
///    number_type: NumberType::Decimal
/// });
/// assert_eq!(String::from("#5_1966"), printed);
/// ```
pub fn print_number_token(number_token: &NumberToken) -> String {
    match number_token.number_type {
        NumberType::Hex => {
            let hex_str = if number_token.value > 0xFFFF {
                format!("{:08X}", number_token.value)
            } else {
                format!("{:04X}", number_token.value)
            };
            format!("#0x{}", add_underscores(&hex_str))
        }
        NumberType::Decimal => {
            let dec_str = format!("{}", number_token.value);
            format!("#{}", add_underscores(&dec_str))
        }
        NumberType::Binary => {
            let bin_str = format!("{:b}", number_token.value);
            format!("#0b{}", add_underscores(&bin_str))
        }
    }
}

/// Prints the AST representation of a `RefToken` to a string
///
///```
/// use toolchain::printers::data::print_equ_token;
/// use toolchain::printers::shared::print_ref_token;
/// use toolchain::types::data::{EquToken, RefToken};
/// use toolchain::types::object::RefType;
/// use toolchain::types::shared::{NumberToken, NumberType};
/// let printed = print_ref_token(&RefToken {
///    name: String::from("some_other_symbol"),
///    ref_type: RefType::UpperWord
/// });
/// assert_eq!(String::from("@some_other_symbol.u"), printed);
/// ```
pub fn print_ref_token(ref_token: &RefToken) -> String {
    let optional_postamble = match ref_token.ref_type {
        RefType::Offset => String::from(REF_TOKEN_OFFSET_SUFFIX),
        RefType::LowerWord => String::from(REF_TOKEN_LOWER_WORD_SUFFIX),
        RefType::UpperWord => String::from(REF_TOKEN_UPPER_WORD_SUFFIX),
        _ => String::new(),
    };

    format!("@{}{}", ref_token.name, optional_postamble)
}

/// Prints an AST `Token` to a string
///
/// This is the generic function that should work over any token
///
///```
/// use toolchain::printers::shared::print_token;
/// use toolchain::types::data::{EquToken, RefToken};
/// use toolchain::types::object::RefType;
/// use toolchain::types::shared::{NumberToken, NumberType, Token};
/// let printed = print_token(&Token::Equ(
///    EquToken {
///        placeholder_name: String::from("BAR"),
///        number_token: NumberToken {
///            number_type: NumberType::Decimal,
///            value: 1234
///        }
///    }
/// ));
/// assert_eq!(String::from(".EQU $BAR #1234"), printed);
/// ```
pub fn print_token(token: &Token) -> String {
    match token {
        Token::Comment(text) => format!(";{text}"),
        Token::Label(LabelToken { name }) => format!(":{name}"),
        Token::Instruction(_) => todo!(),
        Token::Origin(_) => todo!(),
        Token::Data(data_token) => print_data_token(data_token),
        Token::Equ(equ_token) => print_equ_token(equ_token),
    }
}

/// Prints an AST tree to a string
///
///```
/// use toolchain::printers::shared::print_tokens;
/// use toolchain::types::data::{EquToken, RefToken};
/// use toolchain::types::object::RefType;
/// use toolchain::types::shared::{NumberToken, NumberType, Token};
/// let printed = print_tokens(&vec![
///    Token::Equ(
///        EquToken {
///            placeholder_name: String::from("FOO"),
///            number_token: NumberToken {
///                number_type: NumberType::Decimal,
///                value: 1234
///            }
///        }
///    ),
///    Token::Equ(
///        EquToken {
///            placeholder_name: String::from("BAR"),
///            number_token: NumberToken {
///                number_type: NumberType::Hex,
///                value: 0xCAFE
///            }
///        }
///    ),
/// ]);
/// assert_eq!(r".EQU $FOO #1234
///.EQU $BAR #0xCAFE
///", printed);
/// ```
pub fn print_tokens(tokens: &[Token]) -> String {
    // TODO: Don't discard whitespace when printing
    // category=Toolchain
    // If you parse a file and then print it out again, you lose all the vertical whitespace
    tokens.iter().map(print_token).join("\n") + "\n"
}
