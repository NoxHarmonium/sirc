use super::{alu::perform_shift, shared::DecodedInstruction};
use crate::coprocessors::processing_unit::definitions::{
    Instruction, ShiftOperand, ShiftType, INSTRUCTION_SIZE_WORDS,
};
use crate::coprocessors::processing_unit::encoding::{
    decode_immediate_instruction, decode_register_instruction, decode_short_immediate_instruction,
};
use crate::coprocessors::processing_unit::stages::shared::ShiftParameters;
use crate::registers::{
    sr_bit_is_set, RegisterName, Registers, StatusRegisterFields, SR_REDACTION_MASK,
};

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug)]
enum FetchAndDecodeStepInstructionType {
    Register,
    Immediate,
    ShortImmediate,
}

// TODO: Clean up `clippy::match_same_arms` violation in fetch_and_decode module
// category=Refactoring
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

fn get_register_value(registers: &Registers, index: u8) -> u16 {
    let should_redact_sr = sr_bit_is_set(StatusRegisterFields::ProtectedMode, registers);
    let sr_register_index = RegisterName::Sr as u8;
    if should_redact_sr && index == sr_register_index {
        registers[index] & SR_REDACTION_MASK
    } else {
        registers[index]
    }
}

fn do_shift(
    registers: &Registers,
    sr_a_before_shift: u16,
    shift_params: &ShiftParameters,
) -> (u16, u16) {
    let ShiftParameters {
        shift_count,
        shift_operand,
        shift_type,
    } = *shift_params;
    match shift_operand {
        ShiftOperand::Immediate => {
            perform_shift(sr_a_before_shift, shift_type, u16::from(shift_count))
        }
        ShiftOperand::Register => {
            let dereferenced_shift_count = get_register_value(registers, shift_count);
            perform_shift(sr_a_before_shift, shift_type, dereferenced_shift_count)
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
/// use peripheral_cpu::coprocessors::processing_unit::definitions::{Instruction, ConditionFlags, StatusRegisterUpdateSource};
/// use peripheral_cpu::coprocessors::processing_unit::stages::fetch_and_decode::decode_and_register_fetch;
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
/// let (decoded, overflow) = decode_and_register_fetch([0x81, 0x32, 0xBF, 0x9C], &registers);
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
/// assert_eq!(overflow, false);
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
) -> (DecodedInstruction, bool) {
    // Why don't we just match of the type of instruction and set all the irrelevant registers to zero?
    // Because we want to match the hardware as closely as possible. In the hardware representation,
    // the instruction bits will be broken up and stored in the intermediate registers the same way
    // regardless of the instruction type.
    // This means that, for example, sr_a, and sr_b will be filled with garbage when an immediate instruction
    // is being decoded, because the parts of the instruction that would normally be mapped there, are
    // actually the 'value' rather than register indexes.
    // If we filled these with zero, we might accidentally rely on the value being zero in our
    // simulated version, and then on the hardware it might go wrong because there is actually garbage there.
    let immediate_representation = decode_immediate_instruction(raw_instruction);
    let short_immediate_representation = decode_short_immediate_instruction(raw_instruction);
    let register_representation = decode_register_instruction(raw_instruction);

    // All the representations have the op_code field.
    // immediate_representation was chosen arbitrarily but it could have been any of them
    let op_code = immediate_representation.op_code;

    // TODO: Is this decoded getting too complex? Probably
    let instruction_type = decode_fetch_and_decode_step_instruction_type(op_code);

    let addr_inc: i16 = match op_code {
        Instruction::LoadRegisterFromIndirectRegisterPostIncrement
        | Instruction::LoadRegisterFromIndirectImmediatePostIncrement => 1,
        Instruction::StoreRegisterToIndirectRegisterPreDecrement
        | Instruction::StoreRegisterToIndirectImmediatePreDecrement => -1,
        _ => 0,
    };

    let des = immediate_representation.register;

    let sr_a = register_representation.r2;
    let sr_b = register_representation.r3;

    let des_ = get_register_value(registers, des);

    let shift_params = match instruction_type {
        FetchAndDecodeStepInstructionType::Register
        | FetchAndDecodeStepInstructionType::ShortImmediate => ShiftParameters {
            shift_count: register_representation.shift_count,
            shift_operand: register_representation.shift_operand,
            shift_type: register_representation.shift_type,
        },
        FetchAndDecodeStepInstructionType::Immediate => ShiftParameters {
            shift_count: 0,
            shift_operand: ShiftOperand::Immediate,
            shift_type: ShiftType::None,
        },
    };

    let (sr_a_, sr_b_, sr_shift) = match instruction_type {
        FetchAndDecodeStepInstructionType::Register => {
            let (sr_a_, sr_shift) = do_shift(
                registers,
                get_register_value(registers, sr_a),
                &shift_params,
            );
            (sr_a_, get_register_value(registers, sr_b), sr_shift)
        }
        FetchAndDecodeStepInstructionType::Immediate => (des_, immediate_representation.value, 0x0),
        FetchAndDecodeStepInstructionType::ShortImmediate => {
            let (sr_a_, sr_shift) = do_shift(registers, des_, &shift_params);
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
    let (npc_l_, npc_overflowed) = registers.pl.overflowing_add(INSTRUCTION_SIZE_WORDS as u16);

    let npc_h_ = registers.ph;

    (
        DecodedInstruction {
            ins: op_code,
            des,
            sr_a,
            sr_b,
            con: condition_flag,
            adr: immediate_representation.additional_flags,
            ad_l,
            ad_h,
            sr_src: num::FromPrimitive::from_u8(immediate_representation.additional_flags & 0x3)
                .expect("should fit in two bits"),
            shift_params,
            addr_inc,
            des_ad_l,
            des_ad_h,
            sr_shift,
            sr_a_,
            sr_b_,
            ad_l_: registers[ad_l],
            ad_h_: registers[ad_h],
            con_: condition_flag.should_execute(registers),
            npc_l_,
            npc_h_,
        },
        npc_overflowed,
    )
}
