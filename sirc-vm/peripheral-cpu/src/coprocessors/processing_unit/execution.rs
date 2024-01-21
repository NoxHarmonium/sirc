// use log::trace;
use peripheral_bus::device::{BusAssertions, BusOperation};

use super::stages::shared::DecodedInstruction;
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
        bus_assertions: BusAssertions,
    ) -> BusAssertions {
        if eu_registers.pending_fault.is_some() {
            // Abort CPU execution if fault detected
            return BusAssertions::default();
        }
        let result = match phase {
            ExecutionPhase::InstructionFetchLow => {
                // TODO: Alignment check?

                BusAssertions {
                    address: registers.get_full_pc_address(),
                    op: BusOperation::Read,
                    ..BusAssertions::default()
                }
            }
            ExecutionPhase::InstructionFetchHigh => {
                self.instruction = u32::from(bus_assertions.data) << u16::BITS;

                BusAssertions {
                    address: registers.get_full_pc_address() + 1,
                    op: BusOperation::Read,
                    ..BusAssertions::default()
                }
            }
            ExecutionPhase::InstructionDecode => {
                self.instruction |= u32::from(bus_assertions.data);

                self.decoded_instruction =
                    decode_and_register_fetch(u32::to_be_bytes(self.instruction), registers);

                // Special instruction just for debugging purposes. Probably won't be in hardware
                if self.decoded_instruction.ins == Instruction::CoprocessorCallImmediate
                    && self.decoded_instruction.sr_b_ == 0x14FF
                {
                    BusAssertions {
                        exit_simulation: true,
                        ..BusAssertions::default()
                    }
                } else {
                    BusAssertions::default()
                }
            }
            ExecutionPhase::ExecutionEffectiveAddressExecutor => {
                ExecutionEffectiveAddressExecutor::execute(
                    &self.decoded_instruction,
                    registers,
                    eu_registers,
                    &mut self.intermediate_registers,
                    bus_assertions,
                )
            }
            ExecutionPhase::MemoryAccessExecutor => MemoryAccessExecutor::execute(
                &self.decoded_instruction,
                registers,
                eu_registers,
                &mut self.intermediate_registers,
                bus_assertions,
            ),
            ExecutionPhase::WriteBackExecutor => WriteBackExecutor::execute(
                &self.decoded_instruction,
                registers,
                eu_registers,
                &mut self.intermediate_registers,
                bus_assertions,
            ),
        };

        if sr_bit_is_set(StatusRegisterFields::CpuHalted, registers) {
            BusAssertions {
                exit_simulation: true,
                ..BusAssertions::default()
            }
        } else {
            result
        }
    }
}
