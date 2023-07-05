use peripheral_mem::MemoryPeripheral;

use crate::{
    instructions::definitions::{ConditionFlags, Instruction, StatusRegisterUpdateSource},
    registers::Registers,
};

/**
* The instruction mapped out into components.
*
* Simulates the temporary registers the CPU would have when an instruction
* is being decoded.
*
* To avoid microcode/branching etc. all instructions are mapped out to the the
* same set of registers, however, depending on the instruction, some of the
* fields might be zero or full of garbage. You will need to make sure
* you know what instruction you are using before interpreting these
* registers.
*
* Future work: it might be a good idea in the future to type this so
* only the relevant registers are available for each instruction type.

*/
#[derive(Debug, Default, PartialEq, Eq)]
pub struct DecodedInstruction {
    // Raw Instruction Decode
    pub ins: Instruction,
    pub des: u8,
    pub sr_a: u8,
    pub sr_b: u8,
    pub con: ConditionFlags,
    pub adr: u8,
    pub sr_src: StatusRegisterUpdateSource,
    // Inferred
    pub ad_l: u8,
    pub ad_h: u8,
    pub addr_inc: i8,
    pub des_ad_l: u8,
    pub des_ad_h: u8,
    pub sr_shift: u16,
    // Dereferenced
    pub sr_a_: u16,
    pub sr_b_: u16,
    pub ad_l_: u16,
    pub ad_h_: u16,
    pub con_: bool,
    pub sr: u16,
    pub npc_l_: u16,
    pub npc_h_: u16,
}

#[derive(Debug, Default)]
pub struct IntermediateRegisters {
    pub alu_output: u16,
    pub alu_status_register: u16,
    pub lmd: u16,
}

pub trait StageExecutor {
    fn execute(
        decoded_instruction: &DecodedInstruction,
        registers: &mut Registers,
        intermediate_registers: &mut IntermediateRegisters,
        // TODO: Only the memory access stage needs this. Maybe there is a clever way to only provide each stage what they need?
        mem: &MemoryPeripheral,
    );
}

pub enum ExecutionStage {
    // TODO: Work out a cleaner way to specify the data in each stage
    Execution,
    MemoryAccessAndBranchCompletion,
    WriteBack,
}
