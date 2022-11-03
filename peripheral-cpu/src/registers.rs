use std::mem::size_of;
use std::ops::{Index, IndexMut};

use crate::instructions::alu::sign;
use crate::instructions::encoding::ADDRESS_MASK;

pub enum StatusRegisterFields {
    // Byte 1
    Zero = 0x0001,
    Negative = 0x0002,
    Carry = 0x0004,
    Overflow = 0x0008,
    // Byte 2
    SystemMode = 0x0010,
    InterruptMaskLow = 0x0020,
    InterruptMaskMed = 0x0040,
    InterruptMaskHigh = 0x0080,
    // Byte 3
    // TODO: Is this the best place for this flag? No program will be able to read it
    //       maybe move to internal register?
    WaitingForInterrupt = 0x0100,
    CpuHalted = 0x0200,
}

/// Should map 1:1 with the Registers struct
/// TODO: Can we enforce that the two data structures line up?
#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum RegisterName {
    X1 = 0,
    X2,
    X3,
    Y1,
    Y2,
    Y3,
    Z1,
    Z2,
    Z3,
    Ah,
    Al,
    Ph,
    Pl,
    Sh,
    Sl,
    Sr,
}

impl RegisterName {
    pub fn to_register_index(&self) -> u8 {
        num::ToPrimitive::to_u8(self).expect("Could not convert enum to u8")
    }
}

// Traits

pub trait RegisterIndexing {
    fn get_at_index(&mut self, index: u8) -> u16;
    fn set_at_index(&mut self, index: u8, value: u16);
}

pub trait SegmentedRegisterAccess {
    fn get_segmented_pc(&self) -> (u16, u16);
    fn get_segmented_address(&self) -> (u16, u16);
    fn get_segmented_sp(&self) -> (u16, u16);

    fn set_segmented_sp(&mut self, segmented_sp: (u16, u16));
}
pub trait SegmentedAddress {
    fn to_full_address(self) -> u32;
}

pub trait FullAddressRegisterAccess {
    fn get_full_pc_address(&self) -> u32;
    fn get_full_address_address(&self) -> u32;
    fn get_full_sp_address(&self) -> u32;

    fn set_full_address_address(&mut self, address: u32);
}

pub trait FullAddress {
    fn to_segmented_address(self) -> (u16, u16);
}

#[derive(Debug, Clone, Default)]
pub struct Registers {
    pub x1: u16,
    pub x2: u16,
    pub x3: u16,
    pub y1: u16,
    pub y2: u16,
    pub y3: u16,
    pub z1: u16,
    pub z2: u16,
    pub z3: u16,
    // Address Register
    pub ah: u16, // Base/segment address (8 bits concatenated with al/most significant 8 bits ignored)
    pub al: u16,
    // Program Counter
    pub ph: u16, // Base/segment address (8 bits concatenated with pl/most significant 8 bits ignored)
    pub pl: u16,
    // Stack Pointer (user, system) - depending on system status bit
    pub sh: (u16, u16), // Base/segment address (8 bits concatenated with sl/most significant 8 bits ignored)
    pub sl: (u16, u16),
    // Status Register
    // 0 - Last comparison result (e.g. 1 was success)
    // 1 - CPU is halted
    // 2 - Carry bit
    // 3 - Overflow bit
    pub sr: u16,

    // CPU Internal Registers (not directly accessible to programs)
    // No need to segment these registers into 16 bit registers because instructions don't address them

    // System RAM access is offset from here (e.g. interrupt vectors)
    pub system_ram_offset: u32,
}

impl Index<u8> for Registers {
    type Output = u16;

    fn index(&self, index: u8) -> &Self::Output {
        match index {
            0 => &self.x1,
            1 => &self.x2,
            2 => &self.x3,
            3 => &self.y1,
            4 => &self.y2,
            5 => &self.y3,
            6 => &self.z1,
            7 => &self.z2,
            8 => &self.z3,
            9 => &self.ah,
            10 => &self.al,
            11 => &self.ph,
            12 => &self.pl,
            13 => {
                if sr_bit_is_set(StatusRegisterFields::SystemMode, self) {
                    &self.sh.1
                } else {
                    &self.sh.0
                }
            }
            14 => {
                if sr_bit_is_set(StatusRegisterFields::SystemMode, self) {
                    &self.sl.1
                } else {
                    &self.sl.0
                }
            }
            15 => &self.sr,
            _ => panic!("Fatal: No register mapping for index [{}]", index),
        }
    }
}

impl IndexMut<u8> for Registers {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        match index {
            0 => &mut self.x1,
            1 => &mut self.x2,
            2 => &mut self.x3,
            3 => &mut self.y1,
            4 => &mut self.y2,
            5 => &mut self.y3,
            6 => &mut self.z1,
            7 => &mut self.z2,
            8 => &mut self.z3,
            9 => &mut self.ah,
            10 => &mut self.al,
            11 => &mut self.ph,
            12 => &mut self.pl,
            13 => {
                if sr_bit_is_set(StatusRegisterFields::SystemMode, self) {
                    &mut self.sh.1
                } else {
                    &mut self.sh.0
                }
            }
            14 => {
                if sr_bit_is_set(StatusRegisterFields::SystemMode, self) {
                    &mut self.sl.1
                } else {
                    &mut self.sl.0
                }
            }
            15 => &mut self.sr,
            _ => panic!("Fatal: No register mapping for index [{}]", index),
        }
    }
}

// TODO: Deprecate and remove this in favour of just using the Index/IndexMut traits
impl RegisterIndexing for Registers {
    fn get_at_index(&mut self, index: u8) -> u16 {
        self[index]
    }
    fn set_at_index(&mut self, index: u8, value: u16) {
        self[index] = value;
    }
}

impl SegmentedRegisterAccess for Registers {
    fn get_segmented_pc(&self) -> (u16, u16) {
        (self.ph, self.pl)
    }

    fn get_segmented_address(&self) -> (u16, u16) {
        (self.ah, self.al)
    }

    fn get_segmented_sp(&self) -> (u16, u16) {
        if sr_bit_is_set(StatusRegisterFields::SystemMode, self) {
            (self.sh.1, self.sl.1)
        } else {
            (self.sh.0, self.sl.0)
        }
    }

    fn set_segmented_sp(&mut self, segmented_sp: (u16, u16)) {
        if sr_bit_is_set(StatusRegisterFields::SystemMode, self) {
            (self.sh.1, self.sl.1) = segmented_sp;
        } else {
            (self.sh.0, self.sl.0) = segmented_sp;
        }
    }
}

impl SegmentedAddress for (u16, u16) {
    ///
    /// Converts a segmented address (the high/low component of a 24 bit address split
    /// into two 16 bit registers) into the combined 24 bit address.
    ///
    /// You will most likely need this when converting from the internal CPU registers
    /// to the address representation exposed by the virtual address pins of the CPU
    /// for something like peripheral_mem.
    ///
    /// ```
    /// use peripheral_cpu::registers::SegmentedAddress;
    ///
    /// let segmented_address = (0xCAFE, 0xCAFE);
    /// assert_eq!(segmented_address.to_full_address(), 0x00FECAFE);
    /// ```
    ///
    fn to_full_address(self) -> u32 {
        let (high, low) = self;
        let high_shifted = (high as u32) << (size_of::<u16>() * u8::BITS as usize);
        // Bitwise AND with address mask to ensure that the highest 8 bits are ignored
        // This CPU only supports 24 bit addressing
        (high_shifted | low as u32) & ADDRESS_MASK
    }
}

impl FullAddressRegisterAccess for Registers {
    fn get_full_pc_address(&self) -> u32 {
        self.get_segmented_pc().to_full_address()
    }

    fn get_full_address_address(&self) -> u32 {
        self.get_segmented_address().to_full_address()
    }

    fn get_full_sp_address(&self) -> u32 {
        self.get_segmented_sp().to_full_address()
    }

    fn set_full_address_address(&mut self, address: u32) {
        (self.ah, self.al) = address.to_segmented_address();
    }
}

///
/// Converts a 24 bit address to a segmented address (the high/low component of
/// a 24 bit address split into two 16 bit registers).
///
/// You will most likely need this when converting address exposed
/// by the virtual address pins of the CPU for something like peripheral_mem
/// to the representation in the internal CPU registers.
///
/// ```
/// use peripheral_cpu::registers::FullAddress;
///
/// let full_address = 0xCAFECAFE;
/// assert_eq!(full_address.to_segmented_address(), (0x00FE, 0xCAFE));
/// ```
///
impl FullAddress for u32 {
    fn to_segmented_address(self) -> (u16, u16) {
        // Bitwise AND with address mask to ensure that the highest 8 bits are ignored
        // This CPU only supports 24 bit addressing
        let masked = self & ADDRESS_MASK;
        let high = (masked >> (size_of::<u16>() * u8::BITS as usize)) as u16;
        let low = masked as u16;
        (high, low)
    }
}

///
/// Returns true if the given status register field is set in the
/// status register, otherwise false.
///
/// ```
/// use peripheral_cpu::registers::{sr_bit_is_set, Registers, StatusRegisterFields};
///
/// let mut registers = Registers {
///     sr: 0x0008,
///     ..Registers::default()
/// };
///
/// assert_eq!(true, sr_bit_is_set(StatusRegisterFields::Overflow, &registers));
/// assert_eq!(false, sr_bit_is_set(StatusRegisterFields::Carry, &registers));
/// ```
pub fn sr_bit_is_set(field: StatusRegisterFields, registers: &Registers) -> bool {
    let bit_mask = field as u16;
    registers.sr & bit_mask == bit_mask
}

///
/// Sets a register field (flag) in the status register.
///
/// ```
/// use peripheral_cpu::registers::{set_sr_bit, sr_bit_is_set, Registers, StatusRegisterFields};
///
/// let mut registers = Registers {
///     sr: 0x0000,
///     ..Registers::default()
/// };
///
/// set_sr_bit(StatusRegisterFields::Overflow, &mut registers);
///
/// assert_eq!(registers.sr, 0x0008);
/// assert_eq!(true, sr_bit_is_set(StatusRegisterFields::Overflow, &registers));
/// ```
pub fn set_sr_bit(field: StatusRegisterFields, registers: &mut Registers) {
    let bit_mask = field as u16;
    registers.sr |= bit_mask
}

pub fn set_alu_bits(
    registers: &mut Registers,
    value: u16,
    carry: bool,
    inputs_and_result: Option<(u16, u16, u16)>,
) {
    if value == 0 {
        set_sr_bit(StatusRegisterFields::Zero, registers);
    }
    if (value as i16) < 0 {
        set_sr_bit(StatusRegisterFields::Negative, registers);
    }
    if carry {
        set_sr_bit(StatusRegisterFields::Carry, registers);
    } else {
        clear_sr_bit(StatusRegisterFields::Carry, registers);
    }
    // See http://www.csc.villanova.edu/~mdamian/Past/csc2400fa16/labs/ALU.html
    // The logic is follows: when adding, if the sign of the two inputs is the same, but the result sign is different, then we have an overflow.
    if let Some((i1, i2, result)) = inputs_and_result {
        if sign(i1) == sign(i2) && sign(result) != sign(i1) {
            set_sr_bit(StatusRegisterFields::Overflow, registers);
        } else {
            clear_sr_bit(StatusRegisterFields::Overflow, registers);
        }
    }
}

///
/// Clears a register field (flag) in the status register.
///
/// ```
/// use peripheral_cpu::registers::{clear_sr_bit, sr_bit_is_set, Registers, StatusRegisterFields};
///
/// let mut registers = Registers {
///     sr: 0x000C,
///     ..Registers::default()
/// };
///
/// clear_sr_bit(StatusRegisterFields::Overflow, &mut registers);
///
/// assert_eq!(registers.sr, 0x0004);
/// assert_eq!(false, sr_bit_is_set(StatusRegisterFields::Overflow, &registers));
/// ```
pub fn clear_sr_bit(field: StatusRegisterFields, registers: &mut Registers) {
    let bit_mask = field as u16;
    registers.sr &= !bit_mask
}

///
/// Returns the value of the interrupt mask in the status register as a 4-bit integer.
///
/// ```
/// use peripheral_cpu::registers::{get_interrupt_mask, Registers};
///
/// let mut registers = Registers::default();
///
/// registers.sr = 0b0000_1010;
/// assert_eq!(0, get_interrupt_mask(&registers));
/// registers.sr = 0b0010_1010;
/// assert_eq!(1, get_interrupt_mask(&registers));
/// registers.sr = 0b0100_1010;
/// assert_eq!(2, get_interrupt_mask(&registers));
/// registers.sr = 0b0110_1010;
/// assert_eq!(3, get_interrupt_mask(&registers));
/// registers.sr = 0b1000_1010;
/// assert_eq!(4, get_interrupt_mask(&registers));
/// registers.sr = 0b1010_1010;
/// assert_eq!(5, get_interrupt_mask(&registers));
/// registers.sr = 0b1100_1010;
/// assert_eq!(6, get_interrupt_mask(&registers));
/// registers.sr = 0b1110_1010;
/// assert_eq!(7, get_interrupt_mask(&registers));
/// ```
pub fn get_interrupt_mask(registers: &Registers) -> u8 {
    let bit_mask = StatusRegisterFields::InterruptMaskHigh as u16
        | StatusRegisterFields::InterruptMaskMed as u16
        | StatusRegisterFields::InterruptMaskLow as u16;
    let masked_bits = registers.sr & bit_mask;
    // TODO: Can we work out this shift based on the fields position?
    (masked_bits >> 5) as u8
}

///
/// Sets the value of the interrupt mask in the status register as a 4-bit integer.
///
/// ```
/// use peripheral_cpu::registers::{set_interrupt_mask, Registers};
///
/// let mut registers = Registers::default();
/// registers.sr = 0b0000_1010;
///
/// set_interrupt_mask(&mut registers, 0);
/// assert_eq!(0b0000_1010, registers.sr);
/// set_interrupt_mask(&mut registers, 1);
/// assert_eq!(0b0010_1010, registers.sr);
/// set_interrupt_mask(&mut registers, 2);
/// assert_eq!(0b0100_1010, registers.sr);
/// set_interrupt_mask(&mut registers, 3);
/// assert_eq!(0b0110_1010, registers.sr);
/// set_interrupt_mask(&mut registers, 4);
/// assert_eq!(0b1000_1010, registers.sr);
/// set_interrupt_mask(&mut registers, 5);
/// assert_eq!(0b1010_1010, registers.sr);
/// set_interrupt_mask(&mut registers, 6);
/// assert_eq!(0b1100_1010, registers.sr);
/// set_interrupt_mask(&mut registers, 7);
/// assert_eq!(0b1110_1010, registers.sr);
/// set_interrupt_mask(&mut registers, 8);
/// assert_eq!(0b1110_1010, registers.sr);
/// ```
pub fn set_interrupt_mask(registers: &mut Registers, interrupt_mask: u8) {
    // TODO: Can we work out this shift based on the fields position?
    let shifted_value = (interrupt_mask as u16) << 5;
    let bit_mask = !(StatusRegisterFields::InterruptMaskHigh as u16
        | StatusRegisterFields::InterruptMaskMed as u16
        | StatusRegisterFields::InterruptMaskLow as u16);

    registers.sr = (registers.sr & bit_mask) | shifted_value;
}

///
/// Converts the string version of a register to the index used in instruction encoding.
///
/// This should probably only be used when parsing instructions in an assembler,
/// not in the actual functioning of the VM.
///
pub fn register_name_to_index(name: &str) -> u8 {
    match name {
        "x1" => 0,
        "x2" => 1,
        "x3" => 2,
        "y1" => 3,
        "y2" => 4,
        "y3" => 5,
        "z1" => 6,
        "z2" => 7,
        "z3" => 8,
        // [ah, al]
        "ah" => 9,  // Address high
        "al" => 10, // Address low
        // [ph, pl]
        "ph" => 11, // Program counter high
        "pl" => 12, // Program counter low
        // [sh, sl]
        "sh" => 13, // Program counter high
        "sl" => 14, // Program counter low
        "sr" => 15,
        _ => panic!("Fatal: No register mapping for name [{}]", name),
    }
}
