use peripheral_mem::MemoryPeripheral;

use crate::coprocessors::exception_unit::registers::ExceptionUnitRegisters;
use crate::coprocessors::processing_unit::definitions::Instruction;
use crate::coprocessors::shared::Executor;
use crate::execution::execution_effective_address::ExecutionEffectiveAddressExecutor;
use crate::execution::fetch_and_decode::decode_and_register_fetch;
use crate::execution::memory_access::MemoryAccessExecutor;
use crate::execution::shared::{IntermediateRegisters, StageExecutor};
use crate::execution::write_back::WriteBackExecutor;
use crate::instructions::fetch::fetch_instruction;
use crate::registers::{sr_bit_is_set, Registers, SegmentedRegisterAccess, StatusRegisterFields};
use crate::{Error, CYCLES_PER_INSTRUCTION};

pub struct ProcessingUnitExecutor {}

impl Executor for ProcessingUnitExecutor {
    #[allow(clippy::cast_possible_truncation)]
    fn step<'a>(
        registers: &'a mut Registers,
        eu_registers: &'a mut ExceptionUnitRegisters,
        mem: &MemoryPeripheral,
    ) -> Result<(&'a Registers, &'a mut ExceptionUnitRegisters, u32), Error> {
        // 1. Instruction Fetch (1/2)
        // 2. Instruction Fetch (2/2)
        let raw_instruction = fetch_instruction(mem, registers.get_segmented_pc());

        // 3. Decode/Register Fetch (ID)
        let decoded_instruction = decode_and_register_fetch(raw_instruction, registers);

        // Special instruction just for debugging purposes. Probably won't be in hardware
        if decoded_instruction.ins == Instruction::CoprocessorCallImmediate
            && decoded_instruction.sr_b_ == 0x14FF
        {
            return Err(Error::ProcessorHalted(*registers));
        }

        // TODO: On the real CPU these might have garbage in them?
        // maybe it should only be zeroed on first run and shared between invocations
        let mut intermediate_registers = IntermediateRegisters {
            alu_output: 0,
            alu_status_register: 0,
            lmd: 0,
            address_output: 0,
        };

        ExecutionEffectiveAddressExecutor::execute(
            &decoded_instruction,
            registers,
            &mut intermediate_registers,
            mem,
        );
        MemoryAccessExecutor::execute(
            &decoded_instruction,
            registers,
            &mut intermediate_registers,
            mem,
        );
        WriteBackExecutor::execute(
            &decoded_instruction,
            registers,
            &mut intermediate_registers,
            mem,
        );

        if sr_bit_is_set(StatusRegisterFields::CpuHalted, registers) {
            return Err(Error::ProcessorHalted(*registers));
        }

        Ok((registers, eu_registers, CYCLES_PER_INSTRUCTION))
    }
}
