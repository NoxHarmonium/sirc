use log::trace;
use num::Integer;
// use log::trace;
use peripheral_bus::device::{BusAssertions, BusOperation};

use super::stages::shared::DecodedInstruction;
use crate::coprocessors::exception_unit::definitions::Faults;
use crate::coprocessors::processing_unit::definitions::Instruction;
use crate::coprocessors::processing_unit::stages::execution_effective_address::ExecutionEffectiveAddressExecutor;
use crate::coprocessors::processing_unit::stages::fetch_and_decode::decode_and_register_fetch;
use crate::coprocessors::processing_unit::stages::memory_access::MemoryAccessExecutor;
use crate::coprocessors::processing_unit::stages::shared::{IntermediateRegisters, StageExecutor};
use crate::coprocessors::processing_unit::stages::write_back::WriteBackExecutor;
use crate::coprocessors::shared::{ExecutionPhase, Executor};
use crate::raise_fault;
use crate::registers::{
    sr_bit_is_set, ExceptionUnitRegisters, FullAddressRegisterAccess, RegisterName, Registers,
    StatusRegisterFields,
};

const PRIVILEGED_REGISTERS: &[u8] = &[
    RegisterName::Ah as u8,
    RegisterName::Lh as u8,
    RegisterName::Ph as u8,
    RegisterName::Sh as u8,
];

pub fn check_privilege(instruction: &DecodedInstruction, registers: &Registers) -> bool {
    if !sr_bit_is_set(StatusRegisterFields::ProtectedMode, registers) {
        // No restrictions for system mode
        return false;
    }

    let writing_to_privileged_registers = PRIVILEGED_REGISTERS.contains(&instruction.des);
    // TODO: Extract mask
    let cop_opcode = instruction.sr_b_ & 0x0F00;
    // TODO: Is this brittle? Should we use the writeback decoder to determine if the instruction will write
    // to the pending cop register? Mainly worried about undefined instructions will inadvertently write to that
    // register and allow for privilege escalation
    let is_cop_instruction = instruction.ins == Instruction::CoprocessorCallImmediate
        || instruction.ins == Instruction::CoprocessorCallRegister
        || instruction.ins == Instruction::CompareShortImmediate;
    // Top half of the COP opcodes are privileged
    // TODO: Fix magic constant
    let calling_privileged_cop_opcode = is_cop_instruction && (cop_opcode > 0x0700);

    writing_to_privileged_registers || calling_privileged_cop_opcode
}

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

        trace!(
            "phase: {phase:?} sr: 0x{:X} bus_assertions: {bus_assertions:X?}",
            registers.sr
        );

        if registers.pl.is_odd() {
            eu_registers.pending_fault = raise_fault(
                registers,
                eu_registers,
                Faults::Alignment,
                *phase,
                &bus_assertions,
            );
        }

        let result = match phase {
            ExecutionPhase::InstructionFetchLow => {
                trace!("registers.pl: 0x{:X}", registers.pl);

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

                trace!("self.instruction: {:?}", self.instruction);

                self.decoded_instruction =
                    decode_and_register_fetch(u32::to_be_bytes(self.instruction), registers);

                trace!("self.decoded_instruction: {:?}", self.decoded_instruction);

                let privilege_violation = check_privilege(&self.decoded_instruction, registers);
                if privilege_violation {
                    eu_registers.pending_fault = raise_fault(
                        registers,
                        eu_registers,
                        Faults::PrivilegeViolation,
                        *phase,
                        &bus_assertions,
                    );
                }

                // TODO: I think this works, because branch will overwrite the PC anyway, otherwise we want to advance.
                // but we might need to think about how this would work in FPGA
                registers.pl = self.decoded_instruction.npc_l_;

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
