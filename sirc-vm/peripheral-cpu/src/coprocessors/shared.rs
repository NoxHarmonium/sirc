use peripheral_bus::{device::BusAssertions, BusPeripheral};

use crate::registers::{ExceptionUnitRegisters, Registers, SegmentedAddress};

#[derive(Default, FromPrimitive, ToPrimitive, Debug, PartialEq, Eq)]
pub enum ExecutionPhase {
    #[default]
    InstructionFetchLow = 0x0,
    InstructionFetchHigh = 0x1,
    InstructionDecode = 0x2,
    ExecutionEffectiveAddressExecutor = 0x3,
    MemoryAccessExecutor = 0x4,
    WriteBackExecutor = 0x5,
}

pub trait Executor {
    const COPROCESSOR_ID: u8;

    fn step<'a>(
        &mut self,
        phase: &ExecutionPhase,
        cause_register_value: u16,
        registers: &'a mut Registers,
        eu_registers: &'a mut ExceptionUnitRegisters,
        bus_assertions: BusAssertions,
    ) -> BusAssertions;
}

/// Extends an 8 bit value that is signed to a 16 bit value
///
/// Most of the values we pass around in the virtual machine are represented as unsigned
/// (even if they are technically 2's compliment signed)
/// This is fine most of the time, however, if we want to cast a smaller value to a
/// larger value, if the compiler doesn't know that the bits actually represent a
/// 2's complement number, it will fill the new space with zeros which will cause
/// the number's value to change.
///
///
/// ```
/// use peripheral_cpu::coprocessors::shared::sign_extend_small_offset;
///
/// assert_eq!(sign_extend_small_offset(0xFF), 0xFFFF);
/// assert_eq!(sign_extend_small_offset(0x00), 0x0000);
/// ```
///
#[must_use]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub fn sign_extend_small_offset(small_offset: u8) -> u16 {
    // 1. Convert to i8 so the compiler knows we are dealing with a signed number
    // 2. Extend it to i16, the compiler knows it is signed so it preserves the sign
    // 3. Cast it back to unsigned which we deal with
    i16::from(small_offset as i8) as u16
}

// TODO: Rename to something more generic like fetch double word
#[must_use]
pub fn fetch_instruction(mem: &mut BusPeripheral, pc: (u16, u16)) -> [u8; 4] {
    // Only the CPU knows that the address is split into two 16 bit registers
    // Any other peripheral will only see the 24 address lines
    let full_address: u32 = pc.to_full_address();

    // TODO: Alignment check?
    // TODO: Do we need to copy here?
    let [b1, b2] = u16::to_be_bytes(mem.read_address(full_address).to_owned());
    let [b3, b4] = u16::to_be_bytes(mem.read_address(full_address + 1).to_owned());
    [b1, b2, b3, b4]
}
