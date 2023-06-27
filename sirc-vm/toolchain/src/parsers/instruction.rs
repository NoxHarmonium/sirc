use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::character::complete::{char, multispace0, one_of, space0, space1};
use nom::combinator::{cut, eof, map, map_res, opt};
use nom::error::{ErrorKind, FromExternalError};
use nom::sequence::{delimited, pair, separated_pair};
use nom::Parser;
use nom_supreme::error::ErrorTree;
use nom_supreme::multi::collect_separated_terminated;
use nom_supreme::tag::complete::tag;
use nom_supreme::ParserExt;

use peripheral_cpu::instructions::definitions::{
    ConditionFlags, InstructionData, ShiftType, MAX_SHIFT_COUNT,
};
use peripheral_cpu::registers::{AddressRegisterName, RegisterName};

use crate::types::object::{RefType, SymbolDefinition};

use super::opcodes;
use super::shared::{
    lexeme, parse_comma_sep, parse_label, parse_number, parse_symbol_reference, AsmResult,
};

#[derive(Debug)]
pub struct LabelToken {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct RefToken {
    pub name: String,
    pub ref_type: RefType,
}

#[derive(Debug)]
pub struct InstructionToken {
    pub instruction: InstructionData,
    pub symbol_ref: Option<RefToken>,
}

#[derive(Debug)]
pub enum Address {
    Value(u32),
    SymbolRef(String),
}

#[derive(Debug)]
pub enum Token {
    Comment,
    Label(LabelToken),
    Instruction(InstructionToken),
}

pub fn override_ref_token_type_if_implied(
    ref_token: &RefToken,
    override_ref_type: RefType,
) -> RefToken {
    match ref_token.ref_type {
        RefType::Implied => RefToken {
            name: ref_token.name.to_owned(),
            ref_type: override_ref_type,
        },
        // TODO: Should try to do this without copying
        _ => RefToken {
            name: ref_token.name.to_owned(),
            ref_type: ref_token.ref_type,
        },
    }
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
    SymbolRef(RefToken),
}

pub enum OffsetType {
    Value(i16),
    SymbolRef(String),
}

#[derive(Debug)]
pub enum ShiftDefinitionData {
    Immediate(ShiftType, u8),
    Register(ShiftType, RegisterName),
}

#[derive(Debug)]
pub enum AddressingMode {
    // Immediate | #n | BRAN #12
    Immediate(ImmediateType),
    // Register Direct | xN, yN, zN, aB, sB, pB, sr | LOAD x1, y2
    DirectRegister(RegisterName),
    // Register range direct | rN->rM, | STMR (a), x1->z1
    // DirectRegisterRange((RegisterName, RegisterName)),
    // Address Register Direct | a, p, s | LJMP a
    DirectAddressRegister(AddressRegisterName),
    // Address register indirect with register displacement | (r, a) | STOR (y1, a), x1
    IndirectRegisterDisplacement(RegisterName, AddressRegisterName),
    // Address register indirect with immediate displacement | (#n, a) | LOAD y1, (#-3, a)
    IndirectImmediateDisplacement(ImmediateType, AddressRegisterName),
    // Address register indirect with post increment | (a)+ | LOAD (s)+, x1
    IndirectPostIncrement(AddressRegisterName),
    // Address register indirect with pre decrement | -(a) | STOR x1, -(s)
    IndirectPreDecrement(AddressRegisterName),
    // Shift Definition | LSL #4 | ADDI r1, #3, ASR #2
    ShiftDefinition(ShiftDefinitionData),
}

fn parse_value(i: &str) -> AsmResult<ImmediateType> {
    alt((
        // TODO: Add hash before number!
        map(parse_number, ImmediateType::Value).context("number"),
        map(parse_symbol_reference, |ref_token| {
            ImmediateType::SymbolRef(ref_token)
        })
        .context("symbol reference"),
    ))(i)
}

// Immediate | #n | BRAN #12
fn parse_immediate_addressing(i: &str) -> AsmResult<ImmediateType> {
    parse_value(i)
}

// Register Direct | xN, yN, zN, aB, sB, pB, sr | LOAD x1, y2
fn parse_direct_register(i: &str) -> AsmResult<RegisterName> {
    parse_register(i)
}

// Register Direct | a, p, s | LJMP a
fn parse_direct_address_register(i: &str) -> AsmResult<AddressRegisterName> {
    parse_address_register(i)
}

// Address register indirect with register displacement | (r, a) | STOR (y1, a), x1
fn parse_indirect_register_displacement(i: &str) -> AsmResult<(RegisterName, AddressRegisterName)> {
    let args = separated_pair(parse_register, parse_comma_sep, parse_address_register);
    delimited(char('('), args, char(')'))(i)
}

// Address register indirect with immediate displacement | (#n, a) | LOAD y1, (#-3, a)
fn parse_indirect_immediate_displacement(
    i: &str,
) -> AsmResult<(ImmediateType, AddressRegisterName)> {
    let args = separated_pair(parse_value, parse_comma_sep, parse_address_register);
    delimited(char('('), args, char(')'))(i)
}

// Address register indirect with post increment | (a)+ | LOAD (s)+, x1
fn parse_indirect_post_increment(i: &str) -> AsmResult<AddressRegisterName> {
    let (i, address_register) = delimited(char('('), parse_address_register, char(')'))(i)?;
    let (i, _) = char('+')(i)?;
    Ok((i, address_register))
}

// Address register indirect with pre decrement | -(a) | STOR x1, -(s)
fn parse_indirect_pre_decrement(i: &str) -> AsmResult<AddressRegisterName> {
    let (i, _) = char('-')(i)?;
    let (i, address_register) = delimited(char('('), parse_address_register, char(')'))(i)?;
    Ok((i, address_register))
}

#[allow(clippy::let_and_return)]
fn parse_shift_definition(i: &str) -> AsmResult<ShiftDefinitionData> {
    let (i, shift_type) = parse_shift_type(i)?;
    let (i, _) = space1(i)?;

    let parse_register_shift_definition = map(parse_register, |register_name| {
        ShiftDefinitionData::Register(shift_type, register_name)
    });

    let parse_immediate_shift_definition = map_res(parse_number, |shift_count| {
        if shift_count > MAX_SHIFT_COUNT {
            let error_string = format!(
                "Shift definitions can only be in the range of 0-{}, got {}",
                MAX_SHIFT_COUNT, shift_count
            );
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i.to_owned(),
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        } else {
            Ok(ShiftDefinitionData::Immediate(
                shift_type,
                shift_count as u8,
            ))
        }
    });

    let result = alt((
        parse_immediate_shift_definition,
        parse_register_shift_definition,
    ))(i);
    result
}

fn parse_addressing_mode(i: &str) -> AsmResult<AddressingMode> {
    // Immediate can have absolute label references (default lower 16 bit) (@label) or (@label.l explicit lower or @label.h higher (segment))
    // PC/SP relative can have relative label references (16 bit signed) (@label)

    let mut addressing_mode_parser = alt((
        map(parse_immediate_addressing, AddressingMode::Immediate)
            .context("immediate value (e.g. #3)"),
        map(parse_direct_register, AddressingMode::DirectRegister).context("register (e.g. x1)"),
        map(
            parse_direct_address_register,
            AddressingMode::DirectAddressRegister,
        )
        .context("address register (e.g. a)"),
        map(parse_indirect_register_displacement, |(r, ar)| {
            AddressingMode::IndirectRegisterDisplacement(r, ar)
        })
        .context("indirect with register displacement (e.g. (x1, a))"),
        map(parse_indirect_immediate_displacement, |(i, ar)| {
            AddressingMode::IndirectImmediateDisplacement(i, ar)
        })
        .context("indirect with immediate displacement (e.g. (#-1, a))"),
        map(parse_indirect_post_increment, |ar| {
            AddressingMode::IndirectPostIncrement(ar)
        })
        .context("indirect with post increment (e.g. (a)+)"),
        map(parse_indirect_pre_decrement, |ar| {
            AddressingMode::IndirectPreDecrement(ar)
        })
        .context("indirect with pre decrement (e.g. -(a))"),
        map(parse_shift_definition, |shift_definition| {
            AddressingMode::ShiftDefinition(shift_definition)
        })
        .context("shift definition (e.g. ASR #4)"),
    ));

    addressing_mode_parser(i)
}

pub fn parse_instruction_operands1(i: &str) -> AsmResult<Vec<AddressingMode>> {
    let mut parser =
        collect_separated_terminated(parse_addressing_mode, parse_comma_sep, one_of("\n\r"))
            .context("addressing modes");
    parser.parse(i)
}

fn parse_condition_code(i: &str) -> AsmResult<ConditionFlags> {
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

fn parse_shift_type(i: &str) -> AsmResult<ShiftType> {
    map(
        alt((
            tag("NUL"),
            tag("LSL"),
            tag("LSR"),
            tag("ASL"),
            tag("ASR"),
            tag("RTL"),
            tag("RTR"),
        )),
        |code| match code {
            "NUL" => ShiftType::None,
            "LSL" => ShiftType::LogicalLeftShift,
            "LSR" => ShiftType::LogicalRightShift,
            "ASL" => ShiftType::ArithmeticLeftShift,
            "ASR" => ShiftType::ArithmeticRightShift,
            "RTL" => ShiftType::RotateLeft,
            "RTR" => ShiftType::RotateRight,
            _ => panic!("Mismatch between this switch statement and parser tags"),
        },
    )(i)
}

pub fn parse_instruction_tag(
    instruction_tag: &str,
) -> impl FnMut(&str) -> AsmResult<(String, ConditionFlags)> + '_ {
    move |i: &str| {
        // TODO: Work out how to use the nom_supreme tag here (there are lifetime issues with the nested closures)
        let tag_parser = nom::bytes::complete::tag(instruction_tag);
        let (i, tag) = tag_parser(i)?;
        let (i, condition_code_specified) = opt(char('|'))(i)?;
        let (i, condition_code) = if condition_code_specified.is_some() {
            cut(parse_condition_code.context("condition code"))(i)?
        } else {
            (i, ConditionFlags::Always)
        };

        // TODO: Get lexeme working with this function to avoid this
        let (i, _) = space0(i)?;

        Ok((i, (String::from(tag), condition_code)))
    }
}

// Instruction Parsers

// OR should these refer to some constant in the shared crate?
// what happens if the order changes or things are added?
// TODO: use a function to build this up
pub fn parse_instruction_token_(i: &str) -> AsmResult<Token> {
    // Nested alts are to avoid hitting the maximum number of parsers that can be parsed in a single alt statement
    let (i, instruction_token) = alt((
        opcodes::arithmetic_immediate::arithmetic_immediate
            .context("Arithmetic Immediate Instruction"),
        opcodes::arithmetic_register::arithmetic_register
            .context("Arithmetic Register Instruction"),
        opcodes::branching::branching.context("Branching instruction"),
        opcodes::excp::excp.context("EXCP instruction"),
        opcodes::implied::implied.context("Implied instruction"),
        opcodes::ldea::ldea.context("LDEA instruction"),
        opcodes::ljmp::ljmp.context("LJMP instruction"),
        opcodes::ljsr::ljsr.context("LJSR instruction"),
        opcodes::load::load.context("LOAD instruction"),
        opcodes::store::stor.context("STOR instruction"),
    ))(i)?;

    Ok((i, Token::Instruction(instruction_token)))
}

fn parse_address_register_(i: &str) -> AsmResult<AddressRegisterName> {
    map(
        alt((tag("a"), tag("s"), tag("p"), tag("l"))),
        |tag| match tag {
            "a" => AddressRegisterName::Address,
            "p" => AddressRegisterName::ProgramCounter,
            "s" => AddressRegisterName::StackPointer,
            "l" => AddressRegisterName::LinkRegister,
            _ => panic!("Tag mismatch between parser and handler ({})", tag),
        },
    )(i)
}

fn parse_address_register(i: &str) -> AsmResult<AddressRegisterName> {
    lexeme(parse_address_register_)(i)
}

fn parse_register_(i: &str) -> AsmResult<&str> {
    alt((
        tag("r1"),
        tag("r2"),
        tag("r3"),
        tag("r4"),
        tag("r5"),
        tag("r6"),
        tag("r7"),
        tag("lh"),
        tag("ll"),
        tag("ah"),
        tag("al"),
        tag("ph"),
        tag("pl"),
        tag("sh"),
        tag("sl"),
        tag("sr"),
    ))(i)
}

fn parse_register(i: &str) -> AsmResult<RegisterName> {
    map(lexeme(parse_register_), |tag| match tag {
        "r1" => RegisterName::R1,
        "r2" => RegisterName::R2,
        "r3" => RegisterName::R3,
        "r4" => RegisterName::R4,
        "r5" => RegisterName::R5,
        "r6" => RegisterName::R6,
        "r7" => RegisterName::R7,
        "lh" => RegisterName::Lh,
        "ll" => RegisterName::Ll,
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

fn parse_comment_(i: &str) -> AsmResult<Token> {
    // TODO: Should there be a more flexible parser for eol?
    map(pair(char(';'), cut(is_not("\n\r"))), |_| Token::Comment)(i)
}

fn parse_comment(i: &str) -> AsmResult<Token> {
    lexeme(parse_comment_)(i)
}

fn parse_instruction_token(i: &str) -> AsmResult<Token> {
    lexeme(parse_instruction_token_)(i)
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

// TODO: Create object file struct and serialize with serde
// Addresses are replaced with indexes to object table and resolved by linker
pub fn parse_tokens(i: &str) -> AsmResult<Vec<Token>> {
    let mut parser = collect_separated_terminated(
        alt((
            parse_comment.context("comment"),
            parse_instruction_token.context("instruction"),
            parse_label_token.context("label"),
        )),
        multispace0,
        eof,
    );

    parser.parse(i)
}
