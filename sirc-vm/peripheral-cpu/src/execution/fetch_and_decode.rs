use crate::{
    instructions::{
        definitions::{Instruction, RegisterInstructionData, ShiftOperand},
        encoding::{
            decode_immediate_instruction, decode_implied_instruction, decode_register_instruction,
            decode_short_immediate_instruction,
        },
    },
    registers::Registers,
};

use super::{alu::perform_shift, shared::DecodedInstruction};

#[derive(PartialOrd, Ord, PartialEq, Eq)]
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
    match instruction {
        Instruction::AddImmediate => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::AddImmediateWithCarry => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::SubtractImmediate => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::SubtractImmediateWithCarry => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::AndImmediate => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::OrImmediate => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::XorImmediate => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::CompareImmediate => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::TestAndImmediate => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::TestXorImmediate => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::ShiftImmediate => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::BranchImmediate => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::BranchToSubroutineImmediate => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::ShortJumpImmediate => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::ShortJumpToSubroutineImmediate => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::Exception => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::LoadEffectiveAddressFromIndirectImmediate => {
            FetchAndDecodeStepInstructionType::Immediate
        }
        Instruction::LoadEffectiveAddressFromIndirectRegister => {
            FetchAndDecodeStepInstructionType::Register
        }
        Instruction::LongJumpWithImmediateDisplacement => {
            FetchAndDecodeStepInstructionType::Immediate
        }
        Instruction::LongJumpWithRegisterDisplacement => {
            FetchAndDecodeStepInstructionType::Register
        }
        Instruction::LongJumpToSubroutineWithImmediateDisplacement => {
            FetchAndDecodeStepInstructionType::Immediate
        }
        Instruction::LongJumpToSubroutineWithRegisterDisplacement => {
            FetchAndDecodeStepInstructionType::Register
        }
        Instruction::LoadRegisterFromImmediate => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::LoadRegisterFromRegister => FetchAndDecodeStepInstructionType::Register,
        Instruction::LoadRegisterFromIndirectImmediate => {
            FetchAndDecodeStepInstructionType::Immediate
        }
        Instruction::LoadRegisterFromIndirectRegister => {
            FetchAndDecodeStepInstructionType::Register
        }
        Instruction::StoreRegisterToIndirectImmediate => {
            FetchAndDecodeStepInstructionType::Immediate
        }
        Instruction::StoreRegisterToIndirectRegister => FetchAndDecodeStepInstructionType::Register,
        Instruction::LoadRegisterFromIndirectRegisterPostIncrement => {
            FetchAndDecodeStepInstructionType::Register
        }
        Instruction::StoreRegisterToIndirectRegisterPreDecrement => {
            FetchAndDecodeStepInstructionType::Register
        }
        Instruction::AddShortImmediate => FetchAndDecodeStepInstructionType::ShortImmediate,
        Instruction::AddShortImmediateWithCarry => {
            FetchAndDecodeStepInstructionType::ShortImmediate
        }
        Instruction::SubtractShortImmediate => FetchAndDecodeStepInstructionType::ShortImmediate,
        Instruction::SubtractShortImmediateWithCarry => {
            FetchAndDecodeStepInstructionType::ShortImmediate
        }
        Instruction::AndShortImmediate => FetchAndDecodeStepInstructionType::ShortImmediate,
        Instruction::OrShortImmediate => FetchAndDecodeStepInstructionType::ShortImmediate,
        Instruction::XorShortImmediate => FetchAndDecodeStepInstructionType::ShortImmediate,
        Instruction::CompareShortImmediate => FetchAndDecodeStepInstructionType::ShortImmediate,
        Instruction::TestAndShortImmediate => FetchAndDecodeStepInstructionType::ShortImmediate,
        Instruction::TestXorShortImmediate => FetchAndDecodeStepInstructionType::ShortImmediate,
        Instruction::ShiftShortImmediate => FetchAndDecodeStepInstructionType::ShortImmediate,
        Instruction::BranchShortImmediate => FetchAndDecodeStepInstructionType::ShortImmediate,
        Instruction::BranchToSubroutineShortImmediate => {
            FetchAndDecodeStepInstructionType::ShortImmediate
        }
        Instruction::ShortJumpShortImmediate => FetchAndDecodeStepInstructionType::ShortImmediate,
        Instruction::ShortJumpToSubroutineShortImmediate => {
            FetchAndDecodeStepInstructionType::ShortImmediate
        }
        Instruction::ExceptionShort => FetchAndDecodeStepInstructionType::ShortImmediate,
        Instruction::AddRegister => FetchAndDecodeStepInstructionType::Register,
        Instruction::AddRegisterWithCarry => FetchAndDecodeStepInstructionType::Register,
        Instruction::SubtractRegister => FetchAndDecodeStepInstructionType::Register,
        Instruction::SubtractRegisterWithCarry => FetchAndDecodeStepInstructionType::Register,
        Instruction::AndRegister => FetchAndDecodeStepInstructionType::Register,
        Instruction::OrRegister => FetchAndDecodeStepInstructionType::Register,
        Instruction::XorRegister => FetchAndDecodeStepInstructionType::Register,
        Instruction::CompareRegister => FetchAndDecodeStepInstructionType::Register,
        Instruction::TestAndRegister => FetchAndDecodeStepInstructionType::Register,
        Instruction::TestXorRegister => FetchAndDecodeStepInstructionType::Register,
        Instruction::ShiftRegister => FetchAndDecodeStepInstructionType::Register,
        Instruction::ReturnFromSubroutine => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::NoOperation => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::WaitForException => FetchAndDecodeStepInstructionType::Immediate,
        Instruction::ReturnFromException => FetchAndDecodeStepInstructionType::Immediate,
    }
}

fn do_shift(
    registers: &Registers,
    sr_b_before_shift: u16,
    register_representation: &RegisterInstructionData,
) -> (u16, u16) {
    let shift_operand = register_representation.shift_operand;
    match shift_operand {
        ShiftOperand::Immediate => {
            // TODO: Think of a clever way to do this in hardward to save a barrel shifter?
            perform_shift(
                sr_b_before_shift,
                register_representation.shift_type,
                u16::from(register_representation.shift_count),
            )
        }
        ShiftOperand::Register => {
            let dereferenced_shift_count = registers[register_representation.shift_count];
            perform_shift(
                sr_b_before_shift,
                register_representation.shift_type,
                dereferenced_shift_count,
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
/// registers.r5 = 0xCE;
/// registers.sl = (0xFA, 0xFA);
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
/// assert_eq!(decoded.ad_l, 10);
/// assert_eq!(decoded.ad_h, 9);
/// assert_eq!(decoded.sr_src, StatusRegisterUpdateSource::Alu);
/// assert_eq!(decoded.addr_inc, 0x0000);
/// assert_eq!(decoded.des_ad_l, 0x10);
/// assert_eq!(decoded.des_ad_h, 0x0F);
/// assert_eq!(decoded.sr_shift, 0x00);
/// assert_eq!(decoded.sr_a_, 0x00CE);
/// assert_eq!(decoded.sr_b_, 0x00CA);
/// assert_eq!(decoded.ad_l_, 0x00CE);
/// assert_eq!(decoded.ad_h_, 0x00BB);
/// assert_eq!(decoded.con_, true);
/// assert_eq!(decoded.sr, registers.sr);
/// ```
///
#[must_use]
#[allow(clippy::similar_names)]
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

    let addr_inc: i8 = match implied_representation.op_code {
        Instruction::LoadRegisterFromIndirectRegisterPostIncrement => 1, // TODO: Match LOAD (a)+
        Instruction::StoreRegisterToIndirectRegisterPreDecrement => -1,  // TODO: Match STOR -(a)
        _ => 0,
    };

    let des = immediate_representation.register;

    let sr_a = register_representation.r2;
    let sr_b = register_representation.r3;

    let des_ = registers[des];

    let (sr_a_, sr_b_, sr_shift) = match instruction_type {
        FetchAndDecodeStepInstructionType::Register => {
            let (sr_b_, sr_shift) = do_shift(registers, registers[sr_b], &register_representation);
            (registers[sr_a], sr_b_, sr_shift)
        }
        FetchAndDecodeStepInstructionType::Immediate => (des_, immediate_representation.value, 0x0),
        FetchAndDecodeStepInstructionType::ShortImmediate => {
            let (sr_b_, sr_shift) = do_shift(
                registers,
                u16::from(short_immediate_representation.value),
                &register_representation,
            );
            (des_, sr_b_, sr_shift)
        }
    };

    let ad_l = (immediate_representation.additional_flags * 2) + 8;
    let ad_h = (immediate_representation.additional_flags * 2) + 7;
    let des_ad_l = (immediate_representation.register * 2) + 8;
    let des_ad_h = (immediate_representation.register * 2) + 7;
    let condition_flag = immediate_representation.condition_flag;

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
    }
}
