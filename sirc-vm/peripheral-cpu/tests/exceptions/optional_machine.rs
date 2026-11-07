use peripheral_bus::{
    conversion::bytes_to_words,
    device::{BusAccessType, BusAssertions, BusOperation, Device},
};
use peripheral_cpu::{
    coprocessors::processing_unit::{
        definitions::{
            ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
            RegisterInstructionData, ShiftOperand, ShiftType,
        },
        encoding::encode_instruction,
    },
    new_cpu_peripheral_with_config, CpuConfig, CpuPeripheral,
};
use std::collections::BTreeMap;

pub struct Machine {
    pub cpu: CpuPeripheral,
    pub memory: BTreeMap<u32, u16>,
    pub accesses: Vec<BusAssertions>,
    pub pending: Option<BusAssertions>,
    pub cycles: usize,
}
impl Machine {
    pub fn with_config(
        command: u16,
        register_call: bool,
        condition_flag: ConditionFlags,
        config: CpuConfig,
    ) -> Self {
        let mut cpu = new_cpu_peripheral_with_config(config).unwrap();
        cpu.registers.ph = 0xcd00;
        cpu.registers.pl = 0x0200;
        cpu.registers.ah = 0xab01;
        cpu.registers.al = 0x1000;
        cpu.registers.lh = 0xef02;
        cpu.registers.ll = 0x2000;
        cpu.registers.sh = 0xcd03;
        cpu.registers.sl = 0x3000;
        cpu.registers.sr = 0x00a5;
        for index in 0..7 {
            cpu.registers[index] = 0x1100 + u16::from(index);
        }
        cpu.registers.r7 = command;
        let instruction = if register_call {
            InstructionData::Register(RegisterInstructionData {
                op_code: Instruction::CoprocessorCallRegister,
                r1: 0,
                r2: 0,
                r3: 6,
                shift_operand: ShiftOperand::Immediate,
                shift_type: ShiftType::None,
                shift_count: 0,
                condition_flag,
                additional_flags: 0,
            })
        } else {
            InstructionData::Immediate(ImmediateInstructionData {
                op_code: Instruction::CoprocessorCallImmediate,
                register: 0,
                value: command,
                condition_flag,
                additional_flags: 0,
            })
        };
        let words = bytes_to_words(&encode_instruction(&instruction));
        Self {
            cpu,
            memory: BTreeMap::from([(0x0200, words[0]), (0x0201, words[1])]),
            accesses: Vec::new(),
            pending: None,
            cycles: 0,
        }
    }
    pub fn clock(&mut self, mut input: BusAssertions, wait: bool) -> BusAssertions {
        if let Some(request) = self.pending {
            if !wait && !input.bus_error && !input.bus_protection_error {
                input.bus_acknowledge = true;
                if matches!(request.op, BusOperation::Write) {
                    self.memory.insert(request.address, request.data);
                } else {
                    input.data = self.memory.get(&request.address).copied().unwrap_or(0);
                }
                self.accesses.push(request);
            }
        }
        let out = self.cpu.poll(input, true);
        self.pending = out.bus_access_strobe.then_some(out);
        self.cycles += 1;
        out
    }
    pub fn run_to_resume(&mut self) -> usize {
        for _ in 0..2000 {
            let out = self.clock(BusAssertions::default(), false);
            if out.bus_access_strobe
                && out.bus_access_type == BusAccessType::InstructionFetch
                && out.address == 0x0202
            {
                return self.cycles - 1;
            }
            assert!(
                self.cpu.eu_registers.pending_fault.is_none(),
                "unexpected fault after {} cycles",
                self.cycles
            );
        }
        panic!("DMA did not retire");
    }
}
