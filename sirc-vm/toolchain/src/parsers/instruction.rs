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

use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ConditionFlags, ImmediateInstructionData, Instruction, InstructionData, ShiftType,
    MAX_SHIFT_COUNT,
};
use peripheral_cpu::registers::{AddressRegisterName, RegisterName};
use serde::Serialize;

use crate::types::object::{RefType, SymbolDefinition};

use super::data::{parse_data, parse_equ, DataType};
use super::opcodes;
use super::shared::{
    lexeme, parse_comma_sep, parse_label, parse_number, parse_origin, parse_placeholder,
    parse_symbol_reference, AsmResult,
};

#[derive(Debug, Serialize)]
pub struct LabelToken {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RefToken {
    pub name: String,
    pub ref_type: RefType,
}

#[derive(Debug)]
pub struct InstructionToken {
    pub instruction: InstructionData,
    pub symbol_ref: Option<RefToken>,
    pub placeholder_name: Option<String>,
}

impl Default for InstructionToken {
    fn default() -> Self {
        Self {
            instruction: InstructionData::Immediate(ImmediateInstructionData {
                op_code: Instruction::AddImmediate,
                register: 0x0,
                value: 0x0,
                condition_flag: ConditionFlags::Always,
                additional_flags: 0x0,
            }),
            symbol_ref: None,
            placeholder_name: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OriginToken {
    pub offset: u32,
}

#[derive(Debug, Serialize)]
pub struct DataToken {
    pub size_bytes: u8,
    pub value: DataType,
}

#[derive(Debug, Serialize)]
pub struct EquToken {
    pub placeholder_name: String,
    pub value: u32,
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
    Origin(OriginToken),
    Data(DataToken),
    Equ(EquToken),
}

pub fn override_ref_token_type_if_implied(
    ref_token: &RefToken,
    override_ref_type: RefType,
) -> RefToken {
    match ref_token.ref_type {
        RefType::Implied => RefToken {
            name: ref_token.name.clone(),
            ref_type: override_ref_type,
        },
        // TODO: Should try to do this without copying
        _ => RefToken {
            name: ref_token.name.clone(),
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
    PlaceHolder(String),
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
    // Register Direct | rN, lB, aB, sB, pB, sr | LOAD r1, r2
    DirectRegister(RegisterName),
    // Address Register Direct | a, p, s | LJMP a
    DirectAddressRegister(AddressRegisterName),
    // Address register indirect with register displacement | (r, a) | STOR (r1, a), r2
    IndirectRegisterDisplacement(RegisterName, AddressRegisterName),
    // Address register indirect with immediate displacement | (#n, a) | LOAD r1, (#-3, a)
    IndirectImmediateDisplacement(ImmediateType, AddressRegisterName),
    // Address register indirect with post increment | (#n, a)+ | LOAD (#-3, s)+, r1
    IndirectRegisterDisplacementPostIncrement(RegisterName, AddressRegisterName),
    // Address register indirect with post increment | (r, a)+ | LOAD (r2, s)+, r1
    IndirectImmediateDisplacementPostIncrement(ImmediateType, AddressRegisterName),
    // Address register indirect with pre decrement | -(#n, a) | STOR x1, -(#-3, s)
    IndirectRegisterDisplacementPreDecrement(RegisterName, AddressRegisterName),
    // Address register indirect with pre decrement | -(r, a) | STOR r1, -(r2, s)
    IndirectImmediateDisplacementPreDecrement(ImmediateType, AddressRegisterName),
    // Shift Definition | LSL #4 | ADDI r1, #3, ASR #2
    ShiftDefinition(ShiftDefinitionData),
}

#[allow(clippy::cast_possible_truncation)]
pub fn parse_value(i: &str) -> AsmResult<ImmediateType> {
    alt((
        // TODO: Check this cast down to u16?
        map(parse_number, |n| ImmediateType::Value(n as u16)).context("number"),
        map(parse_symbol_reference, |ref_token| {
            ImmediateType::SymbolRef(ref_token)
        })
        .context("symbol reference"),
        map(parse_placeholder, |placeholder_name| {
            ImmediateType::PlaceHolder(placeholder_name)
        })
        .context("placeholder"),
    ))(i)
}

// Immediate | #n | BRAN #12
fn parse_immediate_addressing(i: &str) -> AsmResult<ImmediateType> {
    parse_value(i)
}

// Register Direct | rN, lB, aB, sB, pB, sr | LOAD r1, r2
fn parse_direct_register(i: &str) -> AsmResult<RegisterName> {
    parse_register(i)
}

// Register Direct | l, a, p, s | LJMP a
fn parse_direct_address_register(i: &str) -> AsmResult<AddressRegisterName> {
    parse_address_register(i)
}

// Address register indirect with register displacement | (r, a) | STOR (y1, a), x1
fn parse_indirect_register_displacement(i: &str) -> AsmResult<(RegisterName, AddressRegisterName)> {
    let args = separated_pair(parse_register, parse_comma_sep, parse_address_register);
    delimited(char('('), args, char(')'))(i)
}

// Address register indirect with immediate displacement | (#n, a) | LOAD r1, (#-3, a)
fn parse_indirect_immediate_displacement(
    i: &str,
) -> AsmResult<(ImmediateType, AddressRegisterName)> {
    let args = separated_pair(parse_value, parse_comma_sep, parse_address_register);
    delimited(char('('), args, char(')'))(i)
}

// Address register indirect with immediate displacement and post increment | (#n, a)+ | LOAD (#-3, s)+, x1
fn parse_indirect_immediate_post_increment(
    i: &str,
) -> AsmResult<(ImmediateType, AddressRegisterName)> {
    let (i, args) = parse_indirect_immediate_displacement(i)?;
    let (i, _) = char('+')(i)?;
    Ok((i, args))
}

// Address register indirect with register displacement and post increment | (r, a)+ | LOAD (r1, s)+, x1
fn parse_indirect_register_post_increment(
    i: &str,
) -> AsmResult<(RegisterName, AddressRegisterName)> {
    let (i, args) = parse_indirect_register_displacement(i)?;
    let (i, _) = char('+')(i)?;
    Ok((i, args))
}

// Address register indirect with immediate displacement and pre decrement | -(#n, a) | STOR x1, -(#-3, s)
fn parse_indirect_immediate_pre_decrement(
    i: &str,
) -> AsmResult<(ImmediateType, AddressRegisterName)> {
    let (i, _) = char('-')(i)?;
    let (i, args) = parse_indirect_immediate_displacement(i)?;
    Ok((i, args))
}

// Address register indirect with register displacement and pre decrement | -(r, a) | STOR x1, -(r1, s)
fn parse_indirect_register_pre_decrement(
    i: &str,
) -> AsmResult<(RegisterName, AddressRegisterName)> {
    let (i, _) = char('-')(i)?;
    let (i, args) = parse_indirect_register_displacement(i)?;
    Ok((i, args))
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
                "Shift definitions can only be in the range of 0-{MAX_SHIFT_COUNT}, got {shift_count}"
            );
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i.to_owned(),
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        } else {
            Ok(ShiftDefinitionData::Immediate(
                shift_type,
                shift_count
                    .try_into()
                    .expect("shift_count should fit into MAX_SHIFT_COUNT as it is checked above"),
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
        map(parse_indirect_register_post_increment, |(i, ar)| {
            AddressingMode::IndirectRegisterDisplacementPostIncrement(i, ar)
        })
        .context("indirect with register displacement and post increment (e.g. (r1, a)+)"),
        map(parse_indirect_immediate_post_increment, |(i, ar)| {
            AddressingMode::IndirectImmediateDisplacementPostIncrement(i, ar)
        })
        .context("indirect with immediate displacement and post increment (e.g. (#-1, a)+)"),
        map(parse_indirect_register_pre_decrement, |(i, ar)| {
            AddressingMode::IndirectRegisterDisplacementPreDecrement(i, ar)
        })
        .context("indirect with register displacement and pre decrement (e.g. -(r1, a))"),
        map(parse_indirect_immediate_pre_decrement, |(i, ar)| {
            AddressingMode::IndirectImmediateDisplacementPreDecrement(i, ar)
        })
        .context("indirect with immediate displacement and pre decrement (e.g. -(#-1, a))"),
        map(parse_indirect_register_displacement, |(r, ar)| {
            AddressingMode::IndirectRegisterDisplacement(r, ar)
        })
        .context("indirect with register displacement (e.g. (x1, a))"),
        map(parse_indirect_immediate_displacement, |(i, ar)| {
            AddressingMode::IndirectImmediateDisplacement(i, ar)
        })
        .context("indirect with immediate displacement (e.g. (#-1, a))"),
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
    instruction_tag: &'static str,
) -> impl FnMut(&str) -> AsmResult<(String, ConditionFlags)> + '_ {
    move |i: &str| {
        let tag_parser = tag(instruction_tag);
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
            _ => panic!("Tag mismatch between parser and handler ({tag})"),
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

fn parse_origin_token(i: &str) -> AsmResult<Token> {
    let (i, offset) = parse_origin(i)?;
    Ok((i, Token::Origin(OriginToken { offset })))
}

fn parse_data_token(i: &str) -> AsmResult<Token> {
    let (i, (size_bytes, value)) = parse_data(i)?;

    let override_value = match value {
        // TODO: Is there a better way to do this without the ugly unwrap/wrap
        DataType::Value(value) => DataType::Value(value),
        DataType::SymbolRef(ref_token) => DataType::SymbolRef(override_ref_token_type_if_implied(
            &ref_token,
            RefType::FullAddress,
        )),
        DataType::PlaceHolder(placeholder_name) => DataType::PlaceHolder(placeholder_name),
    };

    Ok((
        i,
        Token::Data(DataToken {
            size_bytes,
            value: override_value,
        }),
    ))
}

pub fn parse_equ_token(i: &str) -> AsmResult<Token> {
    let (i, (placeholder_name, value)) = parse_equ(i)?;

    Ok((
        i,
        Token::Equ(EquToken {
            placeholder_name,
            value,
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
            parse_origin_token.context("origin"),
            parse_data_token.context("data directive"),
            parse_equ_token.context("equ directive"),
        )),
        multispace0,
        eof,
    );

    parser.parse(i)
}
