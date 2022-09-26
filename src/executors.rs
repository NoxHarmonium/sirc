use crate::instructions::*;
use crate::registers::*;

const last_comparison_sr_bit: u16 = 0x01;

fn set_comparison_result(sr: &mut u16, result: bool) -> () {
    if result {
        *sr |= last_comparison_sr_bit
    } else {
        *sr &= !last_comparison_sr_bit
    }
}

pub trait Executor {
    fn execute(&self, registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> ();
}

// Register Transfer

impl Executor for SetInstructionData {
    fn execute(&self, registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> () {
        registers.set_at_index(self.register, self.value);
    }
}

impl Executor for CopyInstructionData {
    fn execute(&self, registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> () {
        let src_value = registers.get_at_index(self.src_register);
        registers.set_at_index(self.dest_register, src_value)
    }
}

// Arithmetic

impl Executor for AddInstructionData {
    // TODO: Set status bits for overflow/carry etc.
    fn execute(&self, registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> () {
        let val_1 = registers.get_at_index(self.src_register);
        let val_2 = registers.get_at_index(self.dest_register);
        registers.set_at_index(self.dest_register, val_1 + val_2)
    }
}

impl Executor for SubtractInstructionData {
    // TODO: Set status bits for overflow/carry etc.
    fn execute(&self, registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> () {
        let val_1 = registers.get_at_index(self.src_register);
        let val_2 = registers.get_at_index(self.dest_register);
        registers.set_at_index(self.dest_register, val_2 - val_1)
    }
}

impl Executor for MultiplyInstructionData {
    // TODO: Set status bits for overflow/carry etc.
    fn execute(&self, registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> () {
        let val_1 = registers.get_at_index(self.src_register);
        let val_2 = registers.get_at_index(self.dest_register);
        registers.set_at_index(self.dest_register, val_2 * val_1)
    }
}

impl Executor for DivideInstructionData {
    // TODO: Set status bits for overflow/carry etc.
    fn execute(&self, registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> () {
        let val_1 = registers.get_at_index(self.src_register);
        let val_2 = registers.get_at_index(self.dest_register);
        registers.set_at_index(self.dest_register, val_2 / val_1)
    }
}

// Comparison

impl Executor for IsEqualInstructionData {
    fn execute(&self, registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> () {
        let val_1 = registers.get_at_index(self.src_register);
        let val_2 = registers.get_at_index(self.dest_register);
        set_comparison_result(&mut registers.sr, val_1 == val_2)
    }
}

impl Executor for IsNotEqualInstructionData {
    fn execute(&self, registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> () {
        let val_1 = registers.get_at_index(self.src_register);
        let val_2 = registers.get_at_index(self.dest_register);
        set_comparison_result(&mut registers.sr, val_1 != val_2)
    }
}

impl Executor for IsLessThanInstructionData {
    fn execute(&self, registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> () {
        let val_1 = registers.get_at_index(self.src_register);
        let val_2 = registers.get_at_index(self.dest_register);
        set_comparison_result(&mut registers.sr, val_1 < val_2)
    }
}

impl Executor for IsGreaterThanInstructionData {
    fn execute(&self, registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> () {
        let val_1 = registers.get_at_index(self.src_register);
        let val_2 = registers.get_at_index(self.dest_register);
        set_comparison_result(&mut registers.sr, val_1 > val_2)
    }
}

impl Executor for IsLessOrEqualThanInstructionData {
    fn execute(&self, registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> () {
        let val_1 = registers.get_at_index(self.src_register);
        let val_2 = registers.get_at_index(self.dest_register);
        set_comparison_result(&mut registers.sr, val_1 <= val_2)
    }
}

impl Executor for IsGreaterOrEqualThanInstructionData {
    fn execute(&self, registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> () {
        let val_1 = registers.get_at_index(self.src_register);
        let val_2 = registers.get_at_index(self.dest_register);
        set_comparison_result(&mut registers.sr, val_1 >= val_2)
    }
}

// Flow Control

impl Executor for JumpInstructionData {
    fn execute(&self, registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> () {
        registers.pc = self.new_pc;
    }
}

impl Executor for JumpIfInstructionData {
    fn execute(&self, registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> () {
        let last_comparison_result = registers.sr & last_comparison_sr_bit;
        if last_comparison_result == last_comparison_sr_bit {
            registers.pc = self.new_pc;
        }
    }
}

impl Executor for JumpIfNotInstructionData {
    fn execute(&self, registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> () {
        let last_comparison_result = registers.sr & last_comparison_sr_bit;
        if last_comparison_result != last_comparison_sr_bit {
            registers.pc = self.new_pc;
        }
    }
}
