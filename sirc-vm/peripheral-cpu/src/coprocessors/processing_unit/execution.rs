use log::trace;
use peripheral_bus::BusPeripheral;

use crate::coprocessors::processing_unit::definitions::Instruction;
use crate::coprocessors::processing_unit::stages::execution_effective_address::ExecutionEffectiveAddressExecutor;
use crate::coprocessors::processing_unit::stages::fetch_and_decode::decode_and_register_fetch;
use crate::coprocessors::processing_unit::stages::memory_access::MemoryAccessExecutor;
use crate::coprocessors::processing_unit::stages::shared::{IntermediateRegisters, StageExecutor};
use crate::coprocessors::processing_unit::stages::write_back::WriteBackExecutor;
use crate::coprocessors::shared::{ExecutionPhase, Executor};
use crate::registers::{
    sr_bit_is_set, ExceptionUnitRegisters, FullAddressRegisterAccess, Registers,
    StatusRegisterFields,
};
use crate::Error;

use super::stages::shared::DecodedInstruction;

#[derive(Default)]
pub struct ProcessingUnitExecutor {
    pub instruction: u32,
    pub decoded_instruction: DecodedInstruction,
    pub intermediate_registers: IntermediateRegisters,
}

impl Executor for ProcessingUnitExecutor {
    const COPROCESSOR_ID: u8 = 0;

    #[allow(clippy::cast_possible_truncation)]
    fn step<'a>(
        &mut self,
        phase: &ExecutionPhase,
        _: u16,
        registers: &'a mut Registers,
        eu_registers: &'a mut ExceptionUnitRegisters,
        mem: &BusPeripheral,
    ) -> Result<(&'a Registers, &'a mut ExceptionUnitRegisters), Error> {
        match phase {
            ExecutionPhase::InstructionFetchLow => {
                // TODO: Alignment check?
                self.instruction =
                    u32::from(mem.read_address(registers.get_full_pc_address()).to_owned())
                        << u16::BITS;
            }
            ExecutionPhase::InstructionFetchHigh => {
                self.instruction |= u32::from(
                    mem.read_address(registers.get_full_pc_address() + 1)
                        .to_owned(),
                );
            }
            ExecutionPhase::InstructionDecode => {
                self.decoded_instruction =
                    decode_and_register_fetch(u32::to_be_bytes(self.instruction), registers);
                trace!("EU: {:X?}", self.decoded_instruction);

                // Special instruction just for debugging purposes. Probably won't be in hardware
                if self.decoded_instruction.ins == Instruction::CoprocessorCallImmediate
                    && self.decoded_instruction.sr_b_ == 0x14FF
                {
                    return Err(Error::ProcessorHalted(*registers));
                }
            }
            ExecutionPhase::ExecutionEffectiveAddressExecutor => {
                ExecutionEffectiveAddressExecutor::execute(
                    &self.decoded_instruction,
                    registers,
                    eu_registers,
                    &mut self.intermediate_registers,
                    mem,
                );
            }
            ExecutionPhase::MemoryAccessExecutor => {
                MemoryAccessExecutor::execute(
                    &self.decoded_instruction,
                    registers,
                    eu_registers,
                    &mut self.intermediate_registers,
                    mem,
                );
            }
            ExecutionPhase::WriteBackExecutor => {
                WriteBackExecutor::execute(
                    &self.decoded_instruction,
                    registers,
                    eu_registers,
                    &mut self.intermediate_registers,
                    mem,
                );
            }
        }

        if sr_bit_is_set(StatusRegisterFields::CpuHalted, registers) {
            return Err(Error::ProcessorHalted(*registers));
        }

        Ok((registers, eu_registers))
    }
}
