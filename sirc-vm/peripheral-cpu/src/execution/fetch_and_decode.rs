use crate::{
    instructions::{
        definitions::{Instruction, RegisterInstructionData, ShiftOperand, INSTRUCTION_SIZE_WORDS},
        encoding::{
            decode_immediate_instruction, decode_implied_instruction, decode_register_instruction,
            decode_short_immediate_instruction,
        },
    },
    registers::Registers,
};

use super::{alu::perform_shift, shared::DecodedInstruction};

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug)]
enum FetchAndDecodeStepInstructionType {
    Register,
    Immediate,
    ShortImmediate,
}

// TODO: Clean up this match and remove this exclusion
#[allow(clippy::match_same_arms)]
fn decode_fetch_and_decode_step_instruction_type(
    instruction: Instruction,
) -> FetchAndDecodeStepInstructionType {
    match num::ToPrimitive::to_u8(&instruction).unwrap() {
        0x00..=0x0F => FetchAndDecodeStepInstructionType::Immediate,
        0x10 | 0x12 | 0x14 | 0x16 | 0x18 | 0x1A | 0x1C | 0x1E => {
            FetchAndDecodeStepInstructionType::Immediate
        }
        0x11 | 0x13 | 0x15 | 0x17 | 0x19 | 0x1B | 0x1D | 0x1F => {
            FetchAndDecodeStepInstructionType::Register
        }
        0x20..=0x2F => FetchAndDecodeStepInstructionType::ShortImmediate,
        0x30..=0x3F => FetchAndDecodeStepInstructionType::Register,
        _ => panic!("No mapping for [{instruction:?}] to FetchAndDecodeStepInstructionType"),
    }
}

fn do_shift(
    registers: &Registers,
    sr_a_before_shift: u16,
    register_representation: &RegisterInstructionData,
    short_immediate: bool, // TODO: Find a smarter solution to this
) -> (u16, u16) {
    let shift_operand = register_representation.shift_operand;
    match shift_operand {
        ShiftOperand::Immediate => {
            // TODO: Think of a clever way to do this in hardward to save a barrel shifter?
            perform_shift(
                sr_a_before_shift,
                register_representation.shift_type,
                u16::from(register_representation.shift_count),
                short_immediate,
            )
        }
        ShiftOperand::Register => {
            let dereferenced_shift_count = registers[register_representation.shift_count];
            perform_shift(
                sr_a_before_shift,
                register_representation.shift_type,
                dereferenced_shift_count,
                short_immediate,
            )
        }
    }
}

///
/// Decodes the instruction and fetches all the referenced registers into an intermediate set of registers
///
/// Once all the data has been extracted and put into a `DecodedInstruction` it should contain everything required
/// for the following steps.
///
/// Should match the hardware as closely as possible.
///
/// ```
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set, StatusRegisterFields, set_sr_bit};
/// use peripheral_cpu::instructions::definitions::{Instruction, ConditionFlags, StatusRegisterUpdateSource};
/// use peripheral_cpu::execution::fetch_and_decode::decode_and_register_fetch;
///
/// let mut registers = Registers::default();
/// registers.r4 = 0xCE;
/// registers.sl = 0xFA;
/// registers.al = 0xCE;
/// registers.ah = 0xBB;
/// registers.sr = 0x00;
///
/// set_sr_bit(StatusRegisterFields::Negative, &mut registers);
///
/// let decoded = decode_and_register_fetch([0x81, 0x32, 0xBF, 0x9C], &registers);
///
/// assert_eq!(decoded.ins, Instruction::AddShortImmediate);
/// assert_eq!(decoded.des, 0x4);
/// assert_eq!(decoded.sr_a, 0xC);
/// assert_eq!(decoded.sr_b, 0xA);
/// assert_eq!(decoded.con, ConditionFlags::LessThan);
/// assert_eq!(decoded.adr, 1);
/// assert_eq!(decoded.ad_l, 11);
/// assert_eq!(decoded.ad_h, 10);
/// assert_eq!(decoded.sr_src, StatusRegisterUpdateSource::Alu);
/// assert_eq!(decoded.addr_inc, 0x0000);
/// assert_eq!(decoded.des_ad_l, 0x9);
/// assert_eq!(decoded.des_ad_h, 0x8);
/// assert_eq!(decoded.sr_shift, 0x00);
/// assert_eq!(decoded.sr_a_, 0x00CE);
/// assert_eq!(decoded.sr_b_, 0x00CA);
/// assert_eq!(decoded.ad_l_, 0x00CE);
/// assert_eq!(decoded.ad_h_, 0x00BB);
/// assert_eq!(decoded.con_, true);
///
/// assert_eq!(decoded.sr, registers.sr);
/// ```
///
#[must_use]
#[allow(
    clippy::similar_names,
    clippy::cast_possible_truncation,
    clippy::cast_lossless
)]
pub fn decode_and_register_fetch(
    raw_instruction: [u8; 4],
    registers: &Registers,
) -> DecodedInstruction {
    // Why don't we just match of the type of instruction and set all the irrelevant registers to zero?
    // Because we want to match the hardware as closely as possible. In the hardware representation,
    // the instruction bits will be broken up and stored in the intermediate registers the same way
    // regardless of the instruction type.
    // This means that, for example, sr_a, and sr_b will be filled with garbage when an immediate instruction
    // is being decoded, because the parts of the instruction that would normally be mapped there, are
    // actually the 'value' rather than register indexes.
    // If we filled these with zero, we might accidentally rely on the value being zero in our
    // simulated version, and then on the hardware it might go wrong because there is actually garbage there.
    let implied_representation = decode_implied_instruction(raw_instruction);
    let immediate_representation = decode_immediate_instruction(raw_instruction);
    let short_immediate_representation = decode_short_immediate_instruction(raw_instruction);
    let register_representation = decode_register_instruction(raw_instruction);

    // TODO: Is this decoded getting too complex? Probably
    let instruction_type =
        decode_fetch_and_decode_step_instruction_type(implied_representation.op_code);

    let addr_inc: i16 = match implied_representation.op_code {
        Instruction::LoadRegisterFromIndirectRegisterPostIncrement
        | Instruction::LoadRegisterFromIndirectImmediatePostIncrement => 1, // TODO: Match LOAD (a)+
        Instruction::StoreRegisterToIndirectRegisterPreDecrement
        | Instruction::StoreRegisterToIndirectImmediatePreDecrement => -1, // TODO: Match STOR -(a)
        _ => 0,
    };

    let des = immediate_representation.register;

    let sr_a = register_representation.r2;
    let sr_b = register_representation.r3;

    let des_ = registers[des];

    let (sr_a_, sr_b_, sr_shift) = match instruction_type {
        FetchAndDecodeStepInstructionType::Register => {
            let (sr_a_, sr_shift) =
                do_shift(registers, registers[sr_a], &register_representation, false);
            (sr_a_, registers[sr_b], sr_shift)
        }
        FetchAndDecodeStepInstructionType::Immediate => (des_, immediate_representation.value, 0x0),
        FetchAndDecodeStepInstructionType::ShortImmediate => {
            let (sr_a_, sr_shift) = do_shift(registers, des_, &register_representation, true);
            (sr_a_, short_immediate_representation.value as u16, sr_shift)
        }
    };

    // Address registers are 0x8-0xF - multiplying by two and setting the left most bit
    // converts it to a full register index
    // TODO: Extract to function
    let ad_l = 0x9 | immediate_representation.additional_flags << 1;
    let ad_h = 0x8 | immediate_representation.additional_flags << 1;
    let des_ad_l = 0x9 | immediate_representation.register << 1;
    let des_ad_h = 0x8 | immediate_representation.register << 1;
    let condition_flag = immediate_representation.condition_flag;
    let npc_l_ = registers.pl.wrapping_add(INSTRUCTION_SIZE_WORDS as u16);
    let npc_h_ = registers.ph;

    DecodedInstruction {
        ins: implied_representation.op_code,
        des,
        sr_a,
        sr_b,
        con: condition_flag,
        adr: immediate_representation.additional_flags,
        ad_l,
        ad_h,
        sr_src: num::FromPrimitive::from_u8(immediate_representation.additional_flags & 0x3)
            .expect("should fit in two bits"),
        addr_inc,
        des_ad_l,
        des_ad_h,
        sr_shift,
        sr_a_,
        sr_b_,
        ad_l_: registers[ad_l],
        ad_h_: registers[ad_h],
        con_: condition_flag.should_execute(registers),
        sr: registers.sr,
        npc_l_,
        npc_h_,
    }
}
