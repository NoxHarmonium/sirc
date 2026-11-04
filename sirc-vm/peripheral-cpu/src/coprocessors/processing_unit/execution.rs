use log::trace;
// use log::trace;
use peripheral_bus::device::{BusAccessType, BusAssertions, BusOperation};

use super::stages::shared::DecodedInstruction;
use crate::coprocessors::exception_unit::{
    definitions::{vectors::USER_EXCEPTION_VECTOR_START, ExceptionUnitOpCodes, Faults},
    execution::construct_exception_unit_command,
};
use crate::coprocessors::processing_unit::definitions::Instruction;
use crate::coprocessors::processing_unit::stages::execution_effective_address::ExecutionEffectiveAddressExecutor;
use crate::coprocessors::processing_unit::stages::fetch_and_decode::decode_and_register_fetch;
use crate::coprocessors::processing_unit::stages::memory_access::MemoryAccessExecutor;
use crate::coprocessors::processing_unit::stages::shared::{IntermediateRegisters, StageExecutor};
use crate::coprocessors::processing_unit::stages::write_back::{
    writes_register_field, WriteBackExecutor,
};
use crate::coprocessors::shared::{ExecutionPhase, Executor};
use crate::raise_fault;
use crate::registers::{
    sr_bit_is_set, ExceptionUnitRegisters, FullAddressRegisterAccess, RegisterName, Registers,
    SegmentedAddress, StatusRegisterFields,
};

const PRIVILEGED_REGISTERS: &[u8] = &[
    RegisterName::Ah as u8,
    RegisterName::Lh as u8,
    RegisterName::Ph as u8,
    RegisterName::Sh as u8,
];

fn is_coprocessor_call(instruction: Instruction) -> bool {
    matches!(
        instruction,
        Instruction::CoprocessorCallImmediate
            | Instruction::CoprocessorCallRegister
            | Instruction::CoprocessorCallShortImmediate
    )
}

pub fn check_privilege(instruction: &DecodedInstruction, registers: &Registers) -> bool {
    if !sr_bit_is_set(StatusRegisterFields::ProtectedMode, registers) {
        // No restrictions for system mode
        return false;
    }

    if !instruction.con_ {
        return false;
    }

    let is_cop_instruction = is_coprocessor_call(instruction.ins);
    // Only an actual write to a privileged register is a violation: compare, test and store
    // instructions merely read the register named by their register field.
    let writing_to_privileged_registers =
        writes_register_field(instruction) && PRIVILEGED_REGISTERS.contains(&instruction.des);
    // TODO: Magic numbers in `check_privilege`
    // category=Refactoring
    let cop_opcode = instruction.sr_b_ & 0x0F00;
    // Top half of the COP opcodes are privileged
    let calling_privileged_cop_opcode = is_cop_instruction && (cop_opcode > 0x0700);

    writing_to_privileged_registers || calling_privileged_cop_opcode
}

/// Linking operations still use next PC as their return address. Only executed
/// non-link jumps and actual ALU/LOAD writes to pl replace it completely.
fn uses_advanced_pc(decoded: &DecodedInstruction) -> bool {
    if !decoded.con_ {
        return true;
    }
    let non_link_jump = matches!(
        decoded.ins,
        Instruction::LoadEffectiveAddressFromIndirectImmediate
            | Instruction::LoadEffectiveAddressFromIndirectRegister
            | Instruction::LoadEffectiveAddressFromIndirectImmediatePreDecrement
            | Instruction::LoadEffectiveAddressFromIndirectRegisterPreDecrement
    ) && decoded.des_ad_l == RegisterName::Pl.to_register_index();
    let writes_pl =
        writes_register_field(decoded) && decoded.des == RegisterName::Pl.to_register_index();
    !(non_link_jump || writes_pl)
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

        let result = match phase {
            ExecutionPhase::InstructionFetchLow => {
                trace!("registers.pl: 0x{:X}", registers.pl);

                BusAssertions {
                    address: registers.get_full_pc_address(),
                    op: BusOperation::Read,
                    bus_access_strobe: true,
                    bus_access_type: BusAccessType::InstructionFetch,
                    ..BusAssertions::default()
                }
            }
            ExecutionPhase::InstructionFetchHigh => {
                self.instruction = u32::from(bus_assertions.data) << u16::BITS;

                BusAssertions {
                    address: registers.get_full_pc_address() + 1,
                    op: BusOperation::Read,
                    bus_access_strobe: true,
                    bus_access_type: BusAccessType::InstructionFetch,
                    ..BusAssertions::default()
                }
            }
            ExecutionPhase::InstructionDecode => {
                self.instruction |= u32::from(bus_assertions.data);

                trace!("self.instruction: {:?}", self.instruction);

                let (decoded_instruction, _) =
                    decode_and_register_fetch(u32::to_be_bytes(self.instruction), registers);
                self.decoded_instruction = decoded_instruction;

                trace!("self.decoded_instruction: {:?}", self.decoded_instruction);

                let privilege_violation = check_privilege(&self.decoded_instruction, registers);
                if privilege_violation {
                    // Captured before the `registers.pl = npc_l_` advance below, so the fault's
                    // return address is this instruction's own, not the next one's.
                    raise_fault(
                        eu_registers,
                        Faults::PrivilegeViolation,
                        &bus_assertions,
                        registers.get_full_pc_address(),
                    );
                }

                // TODO: Define how PC will be incremented in hardware
                // category=Hardware
                // I think this works, because branch will overwrite the PC anyway, otherwise we want to advance.
                // but we might need to think about how this would work in FPGA
                registers.pl = self.decoded_instruction.npc_l_;

                BusAssertions::default()
            }
            ExecutionPhase::ExecutionEffectiveAddressExecutor => {
                let result = ExecutionEffectiveAddressExecutor::execute(
                    &self.decoded_instruction,
                    registers,
                    eu_registers,
                    &mut self.intermediate_registers,
                    bus_assertions,
                );
                // Validate the effective EXCP command before CSR write-back and
                // handler-level discard. A fault must retry the issuing COP, not
                // the continuation PC already installed during decode.
                let command = self.intermediate_registers.alu_output;
                let reserved_traps =
                    construct_exception_unit_command(&ExceptionUnitOpCodes::SoftwareException, 0)
                        ..construct_exception_unit_command(
                            &ExceptionUnitOpCodes::SoftwareException,
                            USER_EXCEPTION_VECTOR_START,
                        );
                if self.decoded_instruction.con_
                    && is_coprocessor_call(self.decoded_instruction.ins)
                    && sr_bit_is_set(StatusRegisterFields::ProtectedMode, registers)
                    && reserved_traps.contains(&command)
                {
                    raise_fault(
                        eu_registers,
                        Faults::PrivilegeViolation,
                        &bus_assertions,
                        (
                            self.decoded_instruction.pc_h_,
                            self.decoded_instruction.pc_l_,
                        )
                            .to_full_address(),
                    );
                }
                // Address calculations have first fault precedence. Check the
                // PC advance belonging to this instruction before any access or
                // write-back, never at a later instruction/handler fetch.
                let decoded = &self.decoded_instruction;
                if eu_registers.pending_fault.is_none()
                    && sr_bit_is_set(StatusRegisterFields::TrapOnAddressOverflow, registers)
                    && decoded.npc_l_ < decoded.pc_l_
                    && uses_advanced_pc(decoded)
                {
                    raise_fault(
                        eu_registers,
                        Faults::SegmentOverflow,
                        &BusAssertions {
                            address: (decoded.npc_h_, decoded.npc_l_).to_full_address(),
                            bus_access_type: BusAccessType::None,
                            ..BusAssertions::default()
                        },
                        (decoded.pc_h_, decoded.pc_l_).to_full_address(),
                    );
                }
                result
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

        if eu_registers.cpu_halted {
            BusAssertions {
                exit_simulation: true,
                ..BusAssertions::default()
            }
        } else {
            result
        }
    }
}
