use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, multispace0};
use nom::combinator::{eof, map, opt};
use nom::multi::{many1, separated_list0};
use nom::sequence::{delimited, separated_pair};
use nom::IResult;

use peripheral_cpu::instructions::definitions::{ConditionFlags, Instruction};
use peripheral_cpu::registers::RegisterName;

use crate::types::object::SymbolDefinition;

use super::opcodes;
use super::shared::{lexeme, parse_comma_sep, parse_label, parse_number, parse_symbol_reference};

#[derive(Debug)]
pub struct LabelToken {
    pub name: String,
}

#[derive(Debug)]
pub struct InstructionToken {
    pub instruction: Instruction,
    pub symbol_ref: Option<LabelToken>,
}

#[derive(Debug)]
pub enum Address {
    Value(u32),
    SymbolRef(String),
}

#[derive(Debug)]
pub enum Token {
    Label(LabelToken),
    Instruction(InstructionToken),
}

/// These three registers are special in that they can be used
/// as combined 32-bit wide registers, but only for addressing.
#[derive(Debug)]
pub enum AddressRegisterName {
    // a (ah, al)
    Address,
    // p (ph, pl)
    ProgramCounter,
    // s (sh, sl)
    StackPointer,
}

pub fn extract_address_arguments(address: Address) -> (u32, Option<SymbolDefinition>) {
    match address {
        Address::SymbolRef(name) => (
            0x0,
            Some(SymbolDefinition {
                name,
                // First argument after instruction ID
                offset: 1,
            }),
        ),
        Address::Value(value) => (value, None),
    }
}

// Instruction Arguments Parsers

#[derive(Debug)]
pub enum ImmediateType {
    Value(u16),
    SymbolRef(String),
}

pub enum OffsetType {
    Value(i16),
    SymbolRef(String),
}

#[derive(Debug)]
pub enum AddressingMode {
    // Immediate | #n | BRAN #12
    Immediate(ImmediateType),
    // Register Direct | xN, yN, zN, aB, sB, pB, sr | LOAD x1, y2
    DirectRegister(RegisterName),
    // Address register indirect with register displacement | (r, a) | STOR (y1, a), x1
    IndirectRegisterDisplacement(RegisterName, AddressRegisterName),
    // Address register indirect with immediate displacement | (#n, a) | LOAD y1, (#-3, a)
    IndirectImmediateDisplacement(ImmediateType, AddressRegisterName),
}

fn parse_value(i: &str) -> IResult<&str, ImmediateType> {
    alt((
        // TODO: Add hash before number!
        map(parse_number, ImmediateType::Value),
        map(parse_symbol_reference, |symbol_name| {
            ImmediateType::SymbolRef(String::from(symbol_name))
        }),
    ))(i)
}

// Immediate | #n | BRAN #12
fn parse_immediate_addressing(i: &str) -> IResult<&str, ImmediateType> {
    parse_value(i)
}

// Register Direct | xN, yN, zN, aB, sB, pB, sr | LOAD x1, y2
fn parse_direct_register(i: &str) -> IResult<&str, RegisterName> {
    parse_register(i)
}

// Address register indirect with register displacement | (r, a) | STOR (y1, a), x1
fn parse_indirect_register_displacement(
    i: &str,
) -> IResult<&str, (RegisterName, AddressRegisterName)> {
    let args = separated_pair(parse_register, parse_comma_sep, parse_address_register);
    delimited(char('('), args, char(')'))(i)
}

// Address register indirect with immediate displacement | (#n, a) | LOAD y1, (#-3, a)
fn parse_indirect_immediate_displacement(
    i: &str,
) -> IResult<&str, (ImmediateType, AddressRegisterName)> {
    let args = separated_pair(parse_value, parse_comma_sep, parse_address_register);
    delimited(char('('), args, char(')'))(i)
}

fn parse_addressing_mode(i: &str) -> IResult<&str, AddressingMode> {
    // TODO: parse the enum above
    // TODO: Parse conditions (<instruction>[|<condition>] [<target_address>,] [<source_address>])
    //
    // Immediate can have absolute label references (default lower 16 bit) (@label) or (@label.l explicit lower or @label.h higher (segment))
    // PC/SP relative can have relative label references (16 bit signed) (@label)
    // Will have to store the above in the symbol table
    // ALMOST THERE

    alt((
        map(parse_immediate_addressing, AddressingMode::Immediate),
        map(parse_direct_register, AddressingMode::DirectRegister),
        map(parse_indirect_register_displacement, |(r, ar)| {
            AddressingMode::IndirectRegisterDisplacement(r, ar)
        }),
        map(parse_indirect_immediate_displacement, |(i, ar)| {
            AddressingMode::IndirectImmediateDisplacement(i, ar)
        }),
    ))(i)
}

pub fn parse_instruction_operands(i: &str) -> IResult<&str, Vec<AddressingMode>> {
    separated_list0(parse_comma_sep, parse_addressing_mode)(i)
}

fn parse_condition_code(i: &str) -> IResult<&str, ConditionFlags> {
    map(
        alt((
            tag("AL"),
            tag("=="),
            tag("!="),
            tag("CS"),
            tag("CC"),
            tag("NS"),
            tag("NC"),
            tag("OS"),
            tag("OC"),
            tag("HI"),
            tag("LO"),
            tag(">="),
            tag("<<"),
            tag(">>"),
            tag("<="),
            tag("NV"),
        )),
        |code| match code {
            "AL" => ConditionFlags::Always,
            "==" => ConditionFlags::Equal,
            "!=" => ConditionFlags::NotEqual,
            "CS" => ConditionFlags::CarrySet,
            "CC" => ConditionFlags::CarryClear,
            "NS" => ConditionFlags::NegativeSet,
            "NC" => ConditionFlags::NegativeClear,
            "OS" => ConditionFlags::OverflowSet,
            "OC" => ConditionFlags::OverflowClear,
            "HI" => ConditionFlags::UnsignedHigher,
            "LO" => ConditionFlags::UnsignedLowerOrSame,
            ">=" => ConditionFlags::GreaterOrEqual,
            "<<" => ConditionFlags::LessThan,
            ">>" => ConditionFlags::GreaterThan,
            "<=" => ConditionFlags::LessThanOrEqual,
            "NV" => ConditionFlags::Never,
            _ => panic!("Mismatch between this switch statement and parser tags"),
        },
    )(i)
}

pub fn parse_instruction_tag(
    instruction_tag: &str,
) -> impl FnMut(&str) -> IResult<&str, ConditionFlags> + '_ {
    move |i: &str| {
        let (i, _) = tag(instruction_tag)(i)?;
        let (i, condition_code_specified) = opt(char('|'))(i)?;
        let (i, condition_code) = if condition_code_specified.is_some() {
            parse_condition_code(i)?
        } else {
            (i, ConditionFlags::Always)
        };

        // TODO: Get lexeme working with this function to avoid this
        let (i, _) = multispace0(i)?;

        Ok((i, condition_code))
    }
}

// Instruction Parsers

// OR should these refer to some constant in the shared crate?
// what happens if the order changes or things are added?
// TODO: use a function to build this up
pub fn parse_instruction_token_(i: &str) -> IResult<&str, Token> {
    let (i, instruction_token) = alt((
        opcodes::halt::halt,
        opcodes::addr::addr,
        opcodes::subr::subr,
        opcodes::mulr::mulr,
        opcodes::divr::divr,
        opcodes::andr::andr,
        opcodes::orrr::orrr,
        opcodes::xorr::xorr,
        opcodes::comp::comp,
        opcodes::sjmp::sjmp,
        opcodes::ljmp::ljmp,
        opcodes::bran::bran,
        opcodes::load::load,
        opcodes::stor::stor,
        opcodes::wait::wait,
        opcodes::reti::reti,
        opcodes::excp::excp,
        opcodes::inof::inof,
        opcodes::inon::inon,
        opcodes::brsr::brsr,
        // Nested alt to avoid hitting the maximum number of parsers that can be parsed to alt
        alt((
            opcodes::sjsr::sjsr,
            opcodes::ljsr::ljsr,
            opcodes::rets::rets,
            opcodes::lslr::lslr,
            opcodes::lsrr::lsrr,
            opcodes::aslr::aslr,
            opcodes::asrr::asrr,
            opcodes::rotl::rotl,
            opcodes::rotr::rotr,
            opcodes::noop::noop,
            opcodes::clsr::clsr,
            opcodes::splt::splt,
            opcodes::join::join,
            opcodes::push::push,
            opcodes::popr::popr,
        )),
    ))(i)?;

    Ok((i, Token::Instruction(instruction_token)))
}

fn parse_address_register_(i: &str) -> IResult<&str, AddressRegisterName> {
    map(alt((tag("a"), tag("s"), tag("p"))), |tag| match tag {
        "a" => AddressRegisterName::Address,
        "p" => AddressRegisterName::ProgramCounter,
        "s" => AddressRegisterName::StackPointer,
        // There is only 2 bits of room anyway so may as well make the 4th bit just the address registers
        _ => AddressRegisterName::Address,
    })(i)
}

fn parse_address_register(i: &str) -> IResult<&str, AddressRegisterName> {
    lexeme(parse_address_register_)(i)
}

fn parse_register_(i: &str) -> IResult<&str, &str> {
    alt((
        tag("x1"),
        tag("y1"),
        tag("z1"),
        tag("x2"),
        tag("y2"),
        tag("z2"),
        tag("x3"),
        tag("y3"),
        tag("z3"),
        tag("ah"),
        tag("al"),
        tag("ph"),
        tag("pl"),
        tag("sh"),
        tag("sl"),
        tag("sr"),
    ))(i)
}

fn parse_register(i: &str) -> IResult<&str, RegisterName> {
    map(lexeme(parse_register_), |tag| match tag {
        "x1" => RegisterName::X1,
        "y1" => RegisterName::Y1,
        "z1" => RegisterName::Z1,
        "x2" => RegisterName::X2,
        "y2" => RegisterName::Y2,
        "z2" => RegisterName::Z2,
        "x3" => RegisterName::X3,
        "y3" => RegisterName::Y3,
        "z3" => RegisterName::Z3,
        "ah" => RegisterName::Ah,
        "al" => RegisterName::Al,
        "ph" => RegisterName::Ph,
        "pl" => RegisterName::Pl,
        "sh" => RegisterName::Sh,
        "sl" => RegisterName::Sl,
        "sr" => RegisterName::Sr,
        _ => panic!("Mismatch between parser and enum mapping"),
    })(i)
}

fn parse_label_token(i: &str) -> IResult<&str, Token> {
    let (i, name) = parse_label(i)?;
    Ok((
        i,
        Token::Label(LabelToken {
            name: String::from(name),
        }),
    ))
}

fn parse_instruction_token(i: &str) -> IResult<&str, Token> {
    lexeme(parse_instruction_token_)(i)
}

// TODO: Create object file struct and serialize with serde
// Addresses are replaced with indexes to object table and resolved by linker
pub fn parse_tokens(i: &str) -> IResult<&str, Vec<Token>> {
    let (i, tokens) = many1(alt((parse_instruction_token, parse_label_token)))(i)?;
    let (i, _) = eof(i)?;

    Ok((i, tokens))
}
