use std::mem::size_of;
use std::ops::{Index, IndexMut};

use crate::instructions::encoding::ADDRESS_MASK;

pub enum StatusRegisterFields {
    LastComparisonResult = 0x01,
    CpuHalted = 0x02,
    Carry = 0x04,
    Overflow = 0x08,
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
}
pub trait SegmentedAddress {
    fn to_full_address(self) -> u32;
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
    // Stack Pointer
    pub sh: u16, // Base/segment address (8 bits concatenated with sl/most significant 8 bits ignored)
    pub sl: u16,
    // Status Register
    // 0 - Last comparison result (e.g. 1 was success)
    // 1 - CPU is halted
    // 2 - Carry bit
    // 3 - Overflow bit
    pub sr: u16,
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
            13 => &self.sh,
            14 => &self.sl,
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
            13 => &mut self.sh,
            14 => &mut self.sl,
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
        (self.sh, self.sl)
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
