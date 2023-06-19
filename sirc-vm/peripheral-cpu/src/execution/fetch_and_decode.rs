use crate::{
    instructions::{
        definitions::Instruction,
        encoding::{
            decode_immediate_instruction, decode_implied_instruction, decode_register_instruction,
        },
    },
    registers::Registers,
};

use super::shared::DecodedInstruction;

///
/// Decodes the instruction and fetches all the referenced registers into an intermediate set of registers
///
/// Once all the data has been extracted and put into a DecodedInstruction it should contain everything required
/// for the following steps.
///
/// Should match the hardware as closely as possible.
///
/// ```
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set, StatusRegisterFields, set_sr_bit};
/// use peripheral_cpu::instructions::definitions::{Instruction, ConditionFlags};
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
/// assert_eq!(decoded.ins, Instruction::BranchImmediate);
/// assert_eq!(decoded.des, 0x4);
/// // Garbage
/// assert_eq!(decoded.sr_a, 0xC);
/// // Garbage
/// assert_eq!(decoded.sr_b, 0xA);
/// assert_eq!(decoded.con, ConditionFlags::LessThan);
/// assert_eq!(decoded.imm, 0xCAFE);
/// assert_eq!(decoded.adr, 1);
/// assert_eq!(decoded.ad_l, 10);
/// assert_eq!(decoded.ad_h, 9);
/// // Garbage
/// assert_eq!(decoded.des_ad_l, 0x10);
/// // Garbage
/// assert_eq!(decoded.des_ad_h, 0x0F);
/// assert_eq!(decoded.addr_inc, 0x0000);
///
/// assert_eq!(decoded.des_, 0xCE);
/// // Garbage
/// assert_eq!(decoded.sr_a_, 0x00FA);
/// // Garbage
/// assert_eq!(decoded.sr_b_, 0x00CE);
/// assert_eq!(decoded.ad_l_, 0x00CE);
/// assert_eq!(decoded.ad_h_, 0x00BB);
/// assert_eq!(decoded.con_, true);
/// assert_eq!(decoded.sr, registers.sr);
/// ```
///
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
    let register_representation = decode_register_instruction(raw_instruction);

    let addr_inc: i8 = match implied_representation.op_code {
        Instruction::LoadRegisterFromIndirectRegisterPostIncrement => 1, // TODO: Match LOAD (a)+
        Instruction::StoreRegisterToIndirectRegisterPreDecrement => -1,  // TODO: Match STOR -(a)
        _ => 0,
    };

    let des = immediate_representation.register;
    let sr_a = register_representation.r2;
    let sr_b = register_representation.r3;

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
        imm: immediate_representation.value,
        adr: immediate_representation.additional_flags,
        ad_l,
        ad_h,
        des_ad_l,
        des_ad_h,
        addr_inc,
        des_: registers[des],
        sr_a_: registers[sr_a],
        sr_b_: registers[sr_b],
        ad_l_: registers[ad_l],
        ad_h_: registers[ad_h],
        con_: condition_flag.should_execute(registers),
        sr: registers.sr,
    }
}
