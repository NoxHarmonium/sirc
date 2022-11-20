use enum_dispatch::enum_dispatch;

use peripheral_mem::MemoryPeripheral;

use crate::instructions::definitions::{
    AddInstructionData, AddWithCarryInstructionData, AndInstructionData,
    ArithmeticShiftLeftInstructionData, ArithmeticShiftRightInstructionData, BranchInstructionData,
    BranchToSubroutineData, CompareInstructionData, DivideInstructionData, HaltInstructionData,
    LoadEffectiveAddressFromIndirectImmediateData, LoadEffectiveAddressFromIndirectRegisterData,
    LoadManyRegisterFromAddressRegisterData, LoadRegisterFromImmediateData,
    LoadRegisterFromIndirectImmediateData, LoadRegisterFromIndirectRegisterData,
    LoadRegisterFromRegisterData, LogicalShiftLeftInstructionData,
    LogicalShiftRightInstructionData, LongJumpToSubroutineWithAddressRegisterData,
    LongJumpWithAddressRegisterData, MultiplyInstructionData, NoOperationInstructionData,
    OrInstructionData, PopInstructionData, PushInstructionData, ReturnFromInterruptData,
    ReturnFromSubroutineData, RotateLeftInstructionData, RotateRightInstructionData,
    ShortJumpToSubroutineWithImmediateData, ShortJumpWithImmediateData,
    StoreManyRegisterFromAddressRegisterData, StoreRegisterToIndirectImmediateData,
    StoreRegisterToIndirectRegisterData, SubtractInstructionData, SubtractWithCarryInstructionData,
    TriggerSoftwareInterruptData, WaitForInterruptInstructionData, XorInstructionData,
};
use crate::microcode::address::{
    calculate_effective_address_with_immediate, calculate_effective_address_with_pc_offset,
    calculate_effective_address_with_register, sign_extend_small_offset,
};
use crate::microcode::interrupts::{jump_to_interrupt, return_from_interrupt};
use crate::microcode::stack::{
    pop_address_from_stack, pop_value_from_stack, push_address_to_stack, push_value_to_stack,
};
use crate::registers::{
    is_valid_register_range, set_alu_bits, set_sr_bit, sr_bit_is_set, AddressRegisterIndexing,
    FullAddress, FullAddressRegisterAccess, RegisterIndexing, Registers, SegmentedRegisterAccess,
    StatusRegisterFields,
};

#[enum_dispatch]
pub trait Executor {
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral);
}

// Implied Argument Instructions

///
/// Halts the CPU by setting the CpuHalted flag.
///
/// Usually used for debugging or for preventing run away computation
/// (e.g. if the PC runs into an empty chunk of memory it will be interpreted as a halt).
///
/// If you want to wait for an event, another instruction should be used such as
/// the WAIT instruction.
///
impl Executor for HaltInstructionData {
    // ID: 0x00
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        set_sr_bit(StatusRegisterFields::CpuHalted, registers);
    }
}

impl Executor for NoOperationInstructionData {
    // ID: 0x01
    fn execute(&self, _registers: &mut Registers, _mem: &MemoryPeripheral) {}
}

impl Executor for WaitForInterruptInstructionData {
    // ID: 0x02
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        set_sr_bit(StatusRegisterFields::WaitingForInterrupt, registers);
    }
}

impl Executor for ReturnFromSubroutineData {
    // ID: 0x03
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let return_address = pop_address_from_stack(registers, mem);
        if sr_bit_is_set(StatusRegisterFields::SystemMode, registers) {
            (registers.ph, registers.pl) = return_address.to_segmented_address();
        } else {
            // Make sure that stack can't be used to do a unprivileged long jump
            (_, registers.pl) = return_address.to_segmented_address();
        }
    }
}

impl Executor for ReturnFromInterruptData {
    // ID: 0x04
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        return_from_interrupt(registers, mem);
    }
}

// Arithmetic

// TODO: Surely there is a way to extract common logic from all the ALU executors

///
/// Executes an addition operation on two registers, storing the result in
/// the first operand.
///
/// If an unsigned overflow occurs, the carry status flag will be set.
/// If a signed overflow occurs, the overflow status flag will be set.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, AddInstructionData, ConditionFlags};
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set, StatusRegisterFields};
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
/// // Thanks: https://stackoverflow.com/a/69125543/1153203
///
/// let mut mem = new_memory_peripheral();
///
/// let mut registers = Registers::default();
///
/// let instructionData = AddInstructionData {
///     data: RegisterInstructionData {
///         r1: 0x00, // x1
///         r2: 0x03, // x2
///         r3: 0x00, // unused
///         condition_flag: ConditionFlags::Always,
///         additional_flags: 0x00,
///     }
/// };
///
/// // Unsigned Overflow
/// registers.x1 = 0xFFFF;
/// registers.x2 = 0x0001;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0x0000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), false);
///
/// // Signed Overflow
/// registers.x1 = 0x7FFF;
/// registers.x2 = 0x2000;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0x9FFF);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), false);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), true);
///
/// // Both Overflow
/// registers.x1 = 0x9FFF;
/// registers.x2 = 0x9000;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0x2FFF);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), true);
/// ```
///
impl Executor for AddInstructionData {
    // ID: 0x05
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        let (result, carry) = val_1.overflowing_add(val_2);
        set_alu_bits(registers, result, carry, Some((val_1, val_2, result)));

        registers.set_at_index(self.data.r1, result);
    }
}

///
/// Executes an addition operation on two registers, storing the result in
/// the first operand.
///
/// If the carry flag is set, it will add one to the result. This
/// allows addition operations to be "chained" over multiple registers.
///
/// If an unsigned overflow occurs, the carry status flag will be set.
/// If a signed overflow occurs, the overflow status flag will be set.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, AddWithCarryInstructionData, ConditionFlags};
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set, StatusRegisterFields};
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
/// // Thanks: https://stackoverflow.com/a/69125543/1153203
///
/// let mut mem = new_memory_peripheral();
///
/// let mut registers = Registers::default();
///
/// let instructionData = AddWithCarryInstructionData {
///     data: RegisterInstructionData {
///         r1: 0x00, // x1
///         r2: 0x03, // x2
///         r3: 0x00, // unused
///         condition_flag: ConditionFlags::Always,
///         additional_flags: 0x00,
///     }
/// };
///
/// // Operation that produces carry
/// registers.x1 = 0xFFFF;
/// registers.x2 = 0xFFFF;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0xFFFE);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
///
/// // Do the same operation now that the carry bit is set
/// registers.x1 = 0xFFFF;
/// registers.x2 = 0xFFFF;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0xFFFF);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
///
/// // Wrapping to zero
/// registers.x1 = 0x0000;
/// registers.x2 = 0xFFFF;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0x0000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// ```
///
impl Executor for AddWithCarryInstructionData {
    // ID: 0x06
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        let carry_from_previous = u16::from(sr_bit_is_set(StatusRegisterFields::Carry, registers));

        let (r1, c1) = val_1.overflowing_add(val_2);
        let (r2, c2) = r1.overflowing_add(carry_from_previous);
        set_alu_bits(registers, r2, c1 | c2, Some((val_1, val_2, r2)));

        registers.set_at_index(self.data.r1, r2);
    }
}

///
/// Executes an subtraction operation on two registers, storing the result in
/// the first operand.
///
/// The second operand is subtracted from the first operand.
///
/// If an unsigned overflow occurs, the carry status flag will be set.
/// If a signed overflow occurs, the overflow status flag will be set.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, SubtractInstructionData, ConditionFlags};
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set, StatusRegisterFields};
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
/// // Thanks: http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
///
/// let mut mem = new_memory_peripheral();
///
/// let mut registers = Registers::default();
///
/// let instructionData = SubtractInstructionData {
///     data: RegisterInstructionData {
///         r1: 0x00, // x1
///         r2: 0x03, // x2
///         r3: 0x00, // unused
///         condition_flag: ConditionFlags::Always,
///         additional_flags: 0x00,
///     }
/// };
///
/// // Unsigned Overflow
/// registers.x1 = 0x5FFF;
/// registers.x2 = 0xFFFF;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0x6000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), false);
///
/// // Signed Overflow
/// registers.x1 = 0xDFFF;
/// registers.x2 = 0x7FFF;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0x6000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), false);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), true);
///
/// // Both Overflow
/// registers.x1 = 0x5FFF;
/// registers.x2 = 0xBFFF;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0xA000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), true);
/// ```
///
impl Executor for SubtractInstructionData {
    // ID: 0x07
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let (result, carry) = val_1.overflowing_sub(val_2);
        //The ones compliment of val_2 is used here for the overflow calculation because it is subtraction
        set_alu_bits(registers, result, carry, Some((val_1, !val_2, result)));

        registers.set_at_index(self.data.r1, result);
    }
}

///
/// Executes an subtraction operation on two registers, storing the result in
/// the first operand.
///
/// The second operand is subtracted from the first operand.
///
/// If the carry flag (which is really borrow flag in this context) is set,
/// it will subtract one from the result. This allows subtraction operations to be "chained"
/// over multiple registers.
///
/// If an unsigned overflow occurs, the carry status flag will be set.
/// If a signed overflow occurs, the overflow status flag will be set.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, SubtractWithCarryInstructionData, ConditionFlags};
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set, StatusRegisterFields};
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
/// // Thanks: http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
///
/// let mut mem = new_memory_peripheral();
///
/// let mut registers = Registers::default();
///
/// let instructionData = SubtractWithCarryInstructionData {
///     data: RegisterInstructionData {
///         r1: 0x00, // x1
///         r2: 0x03, // x2
///         r3: 0x00, // unused
///         condition_flag: ConditionFlags::Always,
///         additional_flags: 0x00,
///     }
/// };
///
/// // Subtraction that causes borrow
/// registers.x1 = 0x0000;
/// registers.x2 = 0xFFFF;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0x0001);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
///
/// // Subtraction with borrow (carry) bit set
/// registers.x1 = 0x0000;
/// registers.x2 = 0xFFFF;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0x0000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// ```
///
impl Executor for SubtractWithCarryInstructionData {
    // ID: 0x08
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        let carry_from_previous = u16::from(sr_bit_is_set(StatusRegisterFields::Carry, registers));

        let (r1, c1) = val_1.overflowing_sub(val_2);
        let (r2, c2) = r1.overflowing_sub(carry_from_previous);
        //The ones compliment of val_2 is used here for the overflow calculation because it is subtraction
        set_alu_bits(registers, r2, c1 | c2, Some((val_1, !val_2, r2)));

        registers.set_at_index(self.data.r1, r2);
    }
}

///
/// Executes an multiplication operation on two registers, storing the result in
/// the first operand.
///
/// If the result is too big to fit in 16 bits, it will wrap around and the carry
/// status flag will be set. However, this does not have the same meaning as
/// a ADDR/SUBR carry, and you will have to be careful when using condition codes.
/// (e.g. condition codes like >= will compare the negative and overflow flags which
/// will not be set with with this instruction.)
/// The zero flag will be set though so ==/!= will work as expected.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, MultiplyInstructionData, ConditionFlags};
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set, StatusRegisterFields};
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
/// let mut mem = new_memory_peripheral();
///
/// let mut registers = Registers::default();
///
/// let instructionData = MultiplyInstructionData {
///     data: RegisterInstructionData {
///         r1: 0x00, // x1
///         r2: 0x03, // x2
///         r3: 0x00, // unused
///         condition_flag: ConditionFlags::Always,
///         additional_flags: 0x00,
///     }
/// };
///
/// // Unsigned Overflow
/// registers.x1 = 0x8000;
/// registers.x2 = 0x0002;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0x0000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), false);
///
/// // Unsigned Overflow Again
/// registers.x1 = 0xFFFF;
/// registers.x2 = 0xFFFF;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0x0001);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), false);
///
/// // No Overflow
/// registers.x1 = 0x1234;
/// registers.x2 = 0x0005;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0x5B04);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), false);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), false);
/// ```
///
impl Executor for MultiplyInstructionData {
    // ID: 0x09
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let (result, carry) = val_1.overflowing_mul(val_2);
        set_alu_bits(registers, result, carry, None);

        registers.set_at_index(self.data.r1, result);
    }
}

///
/// Executes an division operation on two registers, storing the result in
/// the first operand.
///
/// If there is a remainder the carry bit will be set. However, this does not have
/// the same meaning as a ADDR/SUBR carry, and you will have to be careful when using
/// condition codes. (e.g. condition codes like >= will compare the negative and overflow
/// flags which will not be set with with this instruction.)
/// The zero flag will be set though so ==/!= will work as expected.
///
/// There is not currently a way to get the value of the remainder if one exists
/// it will currently have to be determined via a software routine. Future
/// revisions may add another operand to store the remainder.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, DivideInstructionData, ConditionFlags};
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set, StatusRegisterFields};
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
/// let mut mem = new_memory_peripheral();
///
/// let mut registers = Registers::default();
///
/// let instructionData = DivideInstructionData {
///     data: RegisterInstructionData {
///         r1: 0x00, // x1
///         r2: 0x03, // x2
///         r3: 0x00, // unused
///         condition_flag: ConditionFlags::Always,
///         additional_flags: 0x00,
///     }
/// };
///
/// // Remainder of 1
/// registers.x1 = 0x1234;
/// registers.x2 = 0x0003;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0x0611);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), false);
///
/// // No remainder
/// registers.x1 = 0xFFFF;
/// registers.x2 = 0xFFFF;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0x0001);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), false);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), false);
///
/// // No remainder
/// registers.x1 = 0x8000;
/// registers.x2 = 0x0002;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0x4000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), false);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), false);
/// ```
///
impl Executor for DivideInstructionData {
    // ID: 0x0A
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        // TODO: We should probably store the remainder somewhere,
        // although we would then need to have two destination registers
        // We could also pack them into the same registers like the 68k does
        // but that would only allow for 8 bit results
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let (result, carry) = (val_1 / val_2, (val_1 % val_2) != 0);
        set_alu_bits(registers, result, carry, None);

        registers.set_at_index(self.data.r1, result);
    }
}

// Logic

impl Executor for AndInstructionData {
    // ID: 0x0B
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let result = val_1 & val_2;
        set_alu_bits(registers, result, false, Some((val_1, val_2, result)));

        registers.set_at_index(self.data.r1, result);
    }
}

impl Executor for OrInstructionData {
    // ID: 0x0C
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let result = val_1 | val_2;
        set_alu_bits(registers, result, false, Some((val_1, val_2, result)));

        registers.set_at_index(self.data.r1, result);
    }
}

impl Executor for XorInstructionData {
    // ID: 0x0D
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let result = val_1 ^ val_2;
        set_alu_bits(registers, result, false, Some((val_1, val_2, result)));

        registers.set_at_index(self.data.r1, result);
    }
}

// Shifts

impl Executor for LogicalShiftLeftInstructionData {
    // ID: 0x0E
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let (result, carry) = val_1.overflowing_shl(val_2 as u32);
        set_alu_bits(registers, result, carry, None);

        registers.set_at_index(self.data.r1, result);
    }
}

impl Executor for LogicalShiftRightInstructionData {
    // ID: 0x0F
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let (result, carry) = val_1.overflowing_shr(val_2 as u32);
        set_alu_bits(registers, result, carry, None);

        registers.set_at_index(self.data.r1, result);
    }
}

impl Executor for ArithmeticShiftLeftInstructionData {
    // ID: 0x10
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let (result, carry) = val_1.overflowing_shl(val_2 as u32);

        // There can never be an overflow bit set because the sign bit is preserved
        set_alu_bits(registers, result, carry, None);

        registers.set_at_index(self.data.r1, result);
    }
}

impl Executor for ArithmeticShiftRightInstructionData {
    // ID: 0x11
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        let sign_bit = val_1 & 0x8000;

        // TODO: Check this - I think logical/arithmetic shift right are different
        // but arithmetic right shift can be simulated with:
        // - Copy sign bit into carry
        // - Rotate right
        // See: https://stackoverflow.com/a/59658066/1153203
        // But better study the 68k because it has separate instructions
        let (result, carry) = val_1.overflowing_shr(val_2 as u32);
        let signed_result = result | sign_bit;

        // There can never be an overflow bit set because the sign bit is preserved
        set_alu_bits(registers, signed_result, carry, None);

        registers.set_at_index(self.data.r1, signed_result);
    }
}

impl Executor for RotateLeftInstructionData {
    // ID: 0x12
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        let result = val_1.rotate_left(val_2 as u32);
        // Bit shifted out goes into carry (that's what the 68k does :shrug:)
        // TODO: Extract bit checking to function
        // TODO: Check the purpose of bit shifted out going to carry
        let carry = result & 1 == 1;
        set_alu_bits(registers, result, carry, None);

        registers.set_at_index(self.data.r1, result);
    }
}

impl Executor for RotateRightInstructionData {
    // ID: 0x13
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        let result = val_1.rotate_right(val_2 as u32);
        // Bit shifted out goes into carry (that's what the 68k does :shrug:)
        // TODO: Extract bit checking to function
        // TODO: Check the purpose of bit shifted out going to carry
        let carry = (result >> 15) & 1 == 1;
        set_alu_bits(registers, result, carry, None);

        registers.set_at_index(self.data.r1, result);
    }
}

// Comparison

impl Executor for CompareInstructionData {
    // ID: 0x14
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        // TODO: Deduplicate this with the subtract executor
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let (result, carry) = val_1.overflowing_sub(val_2);
        set_alu_bits(registers, result, carry, Some((val_1, !val_2, result)));

        // Exactly the same as a subtraction but does not store the result (only the status bits)
    }
}

// Stack Manipulation

///
/// Pushes a 16 bit word stored in a register into the stack.
///
/// The first operand is the register to push.
/// The stack pointer is always determined by the s register (sh/sl).
///
impl Executor for PushInstructionData {
    // ID: 0x15
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let value = registers.get_at_index(self.data.r1);
        push_value_to_stack(registers, mem, value);
    }
}

///
/// Pops a 16 bit word from the stack and places it in a register.
///
/// The first operand is the register to push.
/// The stack pointer is always determined by the s register (sh/sl).
///
impl Executor for PopInstructionData {
    // ID: 0x16
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let value = pop_value_from_stack(registers, mem);
        registers.set_at_index(self.data.r1, value);
    }
}

// Flow Control

impl Executor for TriggerSoftwareInterruptData {
    // ID: 0x17
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        // Instruction immediate operand is truncated to 7 bits and used as the vector offset
        // for the exception. In effect this means that 128 user exceptions can be defined
        let vector_offset = (self.data.value & 0x0000007F) as u8;
        jump_to_interrupt(vector_offset, registers, mem);
    }
}

///
/// Transfer the lower word address register to the lower word of the program counter.
///
/// Allows a jump to anywhere within a segment.
///
/// Not a privileged operation because it only allows the program to jump within its
/// current segment.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{ImmediateInstructionData, ShortJumpWithImmediateData, ConditionFlags};
/// use peripheral_cpu::registers::{Registers, FullAddressRegisterAccess};
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let jumpInstruction = ShortJumpWithImmediateData {
///   data: ImmediateInstructionData {
///     register: 0x0, // unused
///     value: 0xCAFE,
///     condition_flag: ConditionFlags::Always,
///     additional_flags: 0x0
///   }
/// };
/// let mut registers = Registers::default();
/// registers.set_full_address_address(0x00CAFECA);
/// let mem = new_memory_peripheral();
/// jumpInstruction.execute(&mut registers, &mem);
///
/// assert_eq!((registers.ph, registers.pl), (0x0000, 0xCAFE));
///
/// ```
///
impl Executor for ShortJumpWithImmediateData {
    // ID: 0x18
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        registers.pl = self.data.value;
    }
}

impl Executor for ShortJumpToSubroutineWithImmediateData {
    // ID: 0x19
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        // Even though it is only a short jump and we only need a partial address
        // to return, we push the entire register to be consistent and avoid issues
        // if the system mode bit is flipped during a subroutine
        push_address_to_stack(registers, mem, registers.get_full_pc_address());
        registers.pl = self.data.value;
    }
}

impl Executor for BranchToSubroutineData {
    // ID: 0x1A
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        // Even though it is only a short jump and we only need a partial address
        // to return, we push the entire register to be consistent and avoid issues
        // if the system mode bit is flipped during a subroutine
        push_address_to_stack(registers, mem, registers.get_full_pc_address());
        let effective_address =
            calculate_effective_address_with_pc_offset(registers, self.data.value);
        registers.set_full_pc_address(effective_address);
    }
}

///
/// Moves the CPU program counter by the specified offset, relative
/// to the current program counter.
///
/// Useful for making position independent programs (and avoiding having to
/// store in the address register)
///
///
impl Executor for BranchInstructionData {
    // ID: 0x1B
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let effective_address =
            calculate_effective_address_with_pc_offset(registers, self.data.value);
        registers.set_full_pc_address(effective_address);
    }
}

///
/// Transfer the address registers to the program counter.
///
/// Allows a full 24-bit address to be loaded into the address registers
/// and used to specify where execution starts.
///
/// This should be a privileged operation because it allows the program to jump
/// anywhere it wants (escape its segment).
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, LongJumpWithAddressRegisterData, ConditionFlags};
/// use peripheral_cpu::registers::{Registers, FullAddressRegisterAccess};
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let jumpInstruction = LongJumpWithAddressRegisterData {
///   data: RegisterInstructionData {
///     r1: 0x0,
///     r2: 0x0,
///     r3: 0x0,
///     condition_flag: ConditionFlags::Always,
///     additional_flags: 0x0
///   }
/// };
/// let mut registers = Registers::default();
/// registers.set_full_address_address(0x00CAFECA);
/// let mem = new_memory_peripheral();
/// jumpInstruction.execute(&mut registers, &mem);
///
/// assert_eq!((registers.ph, registers.pl), (0x00CA, 0xFECA));
///
/// ```
///
impl Executor for LongJumpWithAddressRegisterData {
    // ID: 0x1C
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        // TODO: allow address register change (additional_flags)
        (registers.ph, registers.pl) = (registers.ah, registers.al);
    }
}

// Note: Privileged as stack could be overwritten to jump anywhere
impl Executor for LongJumpToSubroutineWithAddressRegisterData {
    // ID: 0x1D
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        push_address_to_stack(registers, mem, registers.get_full_pc_address());
        (registers.ph, registers.pl) = registers.get_segmented_address();
        // TODO: Should these long jumps be able to jump to s or p register sets too?
        // probably not, but maybe that would make it more consistent
    }
}

// Data Access

impl Executor for LoadEffectiveAddressFromIndirectImmediateData {
    // ID: 0x1F
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let source_address_register = self.data.additional_flags;
        let dest_address_register = self.data.register;

        let new_address = calculate_effective_address_with_immediate(
            registers,
            source_address_register,
            self.data.value,
        );

        registers.set_address_register_at_index(dest_address_register, new_address);
    }
}

impl Executor for LoadEffectiveAddressFromIndirectRegisterData {
    // ID: 0x20
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let dest_address_register = self.data.r1;
        let displacement_register = self.data.r2;
        let source_address_register = self.data.r3;

        let new_address = calculate_effective_address_with_register(
            registers,
            source_address_register,
            displacement_register,
        );

        registers.set_address_register_at_index(dest_address_register, new_address);
    }
}

///
/// Loads multiple 16 bit values out of sequential memory addresses into a range of registers.
///
/// The base address value is specified by the given address register.
/// The only supported source addressing mode is immediate displacement but only an offset of +- 128
/// due to encoding constraints.
///
/// The address will be incremented for each value that is read into a register.
///
/// E.g. the following instruction:
///
/// LDMR x1->z1, (#1, s)
///
/// where s is 0x06
///
/// Will result in the following reads
///
/// x1 <- 0x07
/// y1 <- 0x08
/// z1 <- 0x09
///
/// The given address register will be overwritten with the final address that was written to.
/// This is to make reading a range of registers out of the stack faster and easier.
/// If you are reading from a structure that isn't the stack, you'll need to take that into account
/// and possibly use the size of the structure as the immediate displacement.
///
/// E.g.
///
/// LDMR x1->z1, (#-3, a)
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, LoadManyRegisterFromAddressRegisterData, ConditionFlags};
/// use peripheral_cpu::registers::{Registers, SegmentedRegisterAccess, SegmentedAddress};
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let loadManyRegisterFromAddressRegister = LoadManyRegisterFromAddressRegisterData {
///   data: RegisterInstructionData {
///     r1: 0x00, // x1
///     r2: 0x02, // z1
///     r3: 0x02, // s (sh/sl)
///     condition_flag: ConditionFlags::Always,
///     additional_flags: !0u8, // -1
///   }
/// };
/// let start_address_h = 0x1011;
/// let start_address_l = 0x1110;
/// let mut registers = Registers::default();
/// registers.set_segmented_sp((start_address_h, start_address_l));
///
/// let mut mem = new_memory_peripheral();
/// mem.map_segment("TEST", 0x00110000, 0xFFFF, true);
///
/// // Offsets are -1 because of the immediate displacement of -1 (see additional_flags)
/// mem.write_address((start_address_h, start_address_l - 1).to_full_address(), 0xCAFE);
/// mem.write_address((start_address_h, start_address_l).to_full_address(), 0xEFAC);
/// mem.write_address((start_address_h, start_address_l + 1).to_full_address(), 0xBEEF);
///
/// loadManyRegisterFromAddressRegister.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0xCAFE);
/// assert_eq!(registers.y1, 0xEFAC);
/// assert_eq!(registers.z1, 0xBEEF);
/// let (_, final_stack_address_low) = registers.get_segmented_sp();
/// assert_eq!(final_stack_address_low, start_address_l + 1);
///
/// ```
///
impl Executor for LoadManyRegisterFromAddressRegisterData {
    // ID: 0x21
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let start_index = self.data.r1;
        let end_index = self.data.r2;
        let address_register_index = self.data.r3;
        let immediate_offset = sign_extend_small_offset(self.data.additional_flags);
        let offset_operator = u16::wrapping_add;

        // TODO: Work out what would happen on the hardware here
        // Probably something undefined, depending on how the logic is written
        // I doubt it would be worth putting validation in but I'll see
        assert!(is_valid_register_range(start_index, end_index));

        let mut read_address = 0x0;
        for register_index in start_index..=end_index {
            let incremental_offset = register_index - start_index;
            read_address = calculate_effective_address_with_immediate(
                registers,
                address_register_index,
                offset_operator(immediate_offset, incremental_offset as u16),
            );
            let read_value = mem.read_address(read_address);
            registers.set_at_index(register_index, read_value);
        }

        // Only update the address register at the end, otherwise if you are storing the same
        // address register the stored value will be undefined.
        // TODO: Is that too complicated? Should we just make the programmer modify the address register?
        // TODO: I think we probably want to ban doing a many load/store on the address registers as it could lead to all sorts of complexities
        registers.set_address_register_at_index(address_register_index, read_address);
    }
}

///
/// Loads an immediate 16 bit value encoded in an instruction into a register.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{ImmediateInstructionData, LoadRegisterFromImmediateData, ConditionFlags};
/// use peripheral_cpu::registers::Registers;
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let loadRegisterFromImmediateInstruction = LoadRegisterFromImmediateData {
///   data: ImmediateInstructionData {
///     register: 0x02, // z1
///     value: 0xCAFE,
///     condition_flag: ConditionFlags::Always,
///     additional_flags: 0x00,
///   }
/// };
///
/// let mut registers = Registers::default();
///
/// let mut mem = new_memory_peripheral();
///
/// loadRegisterFromImmediateInstruction.execute(&mut registers, &mem);
///
/// assert_eq!(registers.z1, 0xCAFE);
///
/// ```
///
impl Executor for LoadRegisterFromImmediateData {
    // ID: 0x22
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        registers.set_at_index(self.data.register, self.data.value);
    }
}

///
/// Loads a 16 bit value from a register into another register.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, LoadRegisterFromRegisterData, ConditionFlags};
/// use peripheral_cpu::registers::Registers;
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let loadRegisterFromRegisterData = LoadRegisterFromRegisterData {
///   data: RegisterInstructionData {
///     r1: 0x02, // z1
///     r2: 0x00, // x1
///     r3: 0x00, // Unused
///     condition_flag: ConditionFlags::Always,
///     additional_flags: 0x00,
///   }
/// };
/// let mut registers = Registers { x1: 0xCAFE, ..Registers::default() };
///
/// let mut mem = new_memory_peripheral();
///
/// loadRegisterFromRegisterData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.z1, 0xCAFE);
///
/// ```
///
impl Executor for LoadRegisterFromRegisterData {
    // ID: 0x23
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let source_value = registers.get_at_index(self.data.r2);
        registers.set_at_index(self.data.r1, source_value)
    }
}

///
/// Loads a 16 bit value out of a memory address into a register.
///
/// The base address value is specified by the address registers (ah/al)
/// the first operand is the destination register and the operand is an
/// immediate offset to add to the base address value.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{ImmediateInstructionData, LoadRegisterFromIndirectImmediateData, ConditionFlags};
/// use peripheral_cpu::registers::Registers;
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let loadRegisterFromIndirectImmediate = LoadRegisterFromIndirectImmediateData {
///   data: ImmediateInstructionData {
///     register: 0x02, // z1
///     value: 0x0001,
///     condition_flag: ConditionFlags::Always,
///     additional_flags: 0x00,
///   }
/// };
/// let mut registers = Registers { ah: 0x1011, al: 0x1110, ..Registers::default() };
///
/// let mut mem = new_memory_peripheral();
/// mem.map_segment("TEST", 0x00110000, 0xFFFF, true);
/// mem.write_address(0x00111111, 0xCAFE);
///
/// loadRegisterFromIndirectImmediate.execute(&mut registers, &mem);
///
/// assert_eq!(registers.z1, 0xCAFE);
///
/// ```
///
impl Executor for LoadRegisterFromIndirectImmediateData {
    // ID: 0x24
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let address_to_read = calculate_effective_address_with_immediate(
            registers,
            self.data.additional_flags,
            self.data.value,
        );
        let read_value = mem.read_address(address_to_read);
        registers.set_at_index(self.data.register, read_value);
    }
}

///
/// Loads a 16 bit value out of a memory address into a register.
///
/// The base address value is specified by the address registers (ah/al)
/// the first operand is the destination register and the second register
/// is an offset to add to the base address value.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, LoadRegisterFromIndirectRegisterData, ConditionFlags};
/// use peripheral_cpu::registers::Registers;
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let loadOffsetRegisterInstruction = LoadRegisterFromIndirectRegisterData {
///   data: RegisterInstructionData {
///     r1: 0x00, // x1
///     r2: 0x03, // x2
///     r3: 0x00, // unused
///     condition_flag: ConditionFlags::Always,
///     additional_flags: 0x00,
///   }
/// };
/// let mut registers = Registers { ah: 0x1011, al: 0x1110, x2: 0x0001, ..Registers::default() };
///
/// let mut mem = new_memory_peripheral();
/// mem.map_segment("TEST", 0x00110000, 0xFFFF, true);
/// mem.write_address(0x00111111, 0xCAFE);
///
/// loadOffsetRegisterInstruction.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0xCAFE);
///
/// ```
///
impl Executor for LoadRegisterFromIndirectRegisterData {
    // ID: 0x25
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let address_to_read =
            calculate_effective_address_with_register(registers, self.data.r3, self.data.r2);
        let read_value = mem.read_address(address_to_read);
        registers.set_at_index(self.data.r1, read_value);
    }
}

///
/// Stores a 16 bit value from a register into a memory address.
///
/// The base address value is specified by the address registers (ah/al)
/// the first operand is the destination register and the operand is an
/// immediate offset to add to the base address value.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{ImmediateInstructionData, StoreRegisterToIndirectImmediateData, ConditionFlags};
/// use peripheral_cpu::registers::Registers;
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let storeRegisterFromIndirectImmediate = StoreRegisterToIndirectImmediateData {
///   data: ImmediateInstructionData {
///     register: 0x02, // z1
///     value: 0x0001,
///     condition_flag: ConditionFlags::Always,
///     additional_flags: 0x00,
///   }
/// };
/// let mut registers = Registers { z1: 0xCAFE, ah: 0x1011, al: 0x1110, ..Registers::default() };
///
/// let mut mem = new_memory_peripheral();
/// mem.map_segment("TEST", 0x00110000, 0xFFFF, true);
///
/// storeRegisterFromIndirectImmediate.execute(&mut registers, &mem);
/// let stored_value = mem.read_address(0x00111111);
///
/// assert_eq!(stored_value, 0xCAFE);
///
/// ```
///
impl Executor for StoreRegisterToIndirectImmediateData {
    // ID: 0x26
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let address_to_write = calculate_effective_address_with_immediate(
            registers,
            self.data.additional_flags,
            self.data.value,
        );
        mem.write_address(address_to_write, registers.get_at_index(self.data.register));
    }
}

///
/// Stores a 16 bit value from a register into a memory address.
///
/// The base address value is specified by the address registers (ah/al)
/// the first operand is the destination register and the second register
/// is an offset to add to the base address value.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, StoreRegisterToIndirectRegisterData, ConditionFlags};
/// use peripheral_cpu::registers::Registers;
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let storeRegisterToIndirectRegister = StoreRegisterToIndirectRegisterData {
///   data: RegisterInstructionData {
///     r1: 0x00, // x1
///     r2: 0x03, // x2
///     r3: 0x00, // unused
///     condition_flag: ConditionFlags::Always,
///     additional_flags: 0x00,
///   }
/// };
/// let mut registers = Registers { ah: 0x1011, al: 0x1110, x1: 0xCAFE, x2: 0x0001, ..Registers::default() };
///
/// let mut mem = new_memory_peripheral();
/// mem.map_segment("TEST", 0x00110000, 0xFFFF, true);
///
/// storeRegisterToIndirectRegister.execute(&mut registers, &mem);
/// let stored_value = mem.read_address(0x00111111);
///
/// assert_eq!(stored_value, 0xCAFE);
/// ```
///
impl Executor for StoreRegisterToIndirectRegisterData {
    // ID: 0x27
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let address_to_write =
            calculate_effective_address_with_register(registers, self.data.r3, self.data.r2);
        mem.write_address(address_to_write, registers.get_at_index(self.data.r1));
    }
}

///
/// Stores multiple 16 bit values out a range of registers into sequential memory addresses.
///
/// The base address value is specified by the given address register.
/// The only supported source addressing mode is immediate displacement but only an offset of +- 128
/// due to encoding constraints.
///
/// The address will be decremented for each value that is written into memory.
///
/// E.g. the following instruction:
///
/// STMR (#-1, s), x1->z1
///
/// where s is 0x0A
///
/// Will result in the following writes
///
/// 0x09 <- z1
/// 0x08 <- y1
/// 0x07 <- x1
///
/// The given address register will be overwritten with the final address that was written to.
/// This is to make storing a range of registers into the stack faster and easier.
/// If you are storing into a structure that isn't a stack, you'll need to take that into account
/// and possibly use the size of the structure as the immediate displacement.
///
/// E.g.
///
/// STMR (#3, a), x1->z1
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, StoreManyRegisterFromAddressRegisterData, ConditionFlags};
/// use peripheral_cpu::registers::{Registers, SegmentedRegisterAccess, SegmentedAddress};
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let storeManyRegisterFromAddressRegister = StoreManyRegisterFromAddressRegisterData {
///   data: RegisterInstructionData {
///     r1: 0x00, // x1
///     r2: 0x02, // z1
///     r3: 0x02, // s (sh/sl)
///     condition_flag: ConditionFlags::Always,
///     additional_flags: !0u8, // -1
///   }
/// };
/// let start_address_h = 0x1011;
/// let start_address_l = 0x1110;
/// let mut registers = Registers::default();
/// registers.set_segmented_sp((start_address_h, start_address_l));
///
/// registers.x1 = 0xCAFE;
/// registers.y1 = 0xEFAC;
/// registers.z1 = 0xBEEF;
///
/// let mut mem = new_memory_peripheral();
/// mem.map_segment("TEST", 0x00110000, 0xFFFF, true);
///
/// storeManyRegisterFromAddressRegister.execute(&mut registers, &mem);
///
/// // Offsets are -1 because of the immediate displacement of -1 (see additional_flags)
/// let written_x1 = mem.read_address((start_address_h, start_address_l - 1).to_full_address());
/// let written_y1 = mem.read_address((start_address_h, start_address_l - 2).to_full_address());
/// let written_z1 = mem.read_address((start_address_h, start_address_l - 3).to_full_address());
///
/// assert_eq!(written_x1, 0xCAFE);
/// assert_eq!(written_y1, 0xEFAC);
/// assert_eq!(written_z1, 0xBEEF);
/// let (_, final_stack_address_low) = registers.get_segmented_sp();
/// assert_eq!(final_stack_address_low, start_address_l - 3);
///
/// ```
///
impl Executor for StoreManyRegisterFromAddressRegisterData {
    // ID: 0x28
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let start_index = self.data.r1;
        let end_index = self.data.r2;
        let address_register_index = self.data.r3;
        let immediate_offset = sign_extend_small_offset(self.data.additional_flags);
        let offset_operator = u16::wrapping_sub;

        // TODO: Work out what would happen on the hardware here
        // Probably something undefined, depending on how the logic is written
        // I doubt it would be worth putting validation in but I'll see
        assert!(is_valid_register_range(start_index, end_index));

        let mut write_address = 0x0;
        for register_index in start_index..=end_index {
            let incremental_offset = register_index - start_index;
            write_address = calculate_effective_address_with_immediate(
                registers,
                address_register_index,
                offset_operator(immediate_offset, incremental_offset as u16),
            );
            let write_value = registers.get_at_index(register_index);
            mem.write_address(write_address, write_value);
        }

        // Only update the address register at the end, otherwise if you are storing the same
        // address register the stored value will be undefined.
        // TODO: Is that too complicated? Should we just make the programmer modify the address register?
        // TODO: I think we probably want to ban doing a many load/store on the address registers as it could lead to all sorts of complexities
        registers.set_address_register_at_index(address_register_index, write_address);
    }
}
