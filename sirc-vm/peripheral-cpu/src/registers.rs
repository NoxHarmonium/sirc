use std::mem::size_of;
use std::ops::{Index, IndexMut};

/// The bits of an address register pair that actually gets mapped to physical pins
/// (Only 24 bit addressing)
pub const ADDRESS_MASK: u32 = 0x00FFFFFF;

pub enum StatusRegisterFields {
    // Byte 1
    Zero = 0b0000_0001,
    Negative = 0b0000_0010,
    Carry = 0b0000_0100,
    Overflow = 0b0000_1000,

    // Byte 3 (privileged)
    SystemMode = 0b0000_0001 << u8::BITS,
    InterruptMaskLow = 0b0000_0010 << u8::BITS,
    InterruptMaskMed = 0b0000_0100 << u8::BITS,
    InterruptMaskHigh = 0b0000_1000 << u8::BITS,
    // TODO: Is this the best place for this flag? No program will be able to read it
    //       maybe move to internal register?
    WaitingForInterrupt = 0b0001_0000 << u8::BITS,
    CpuHalted = 0b0010_0000 << u8::BITS,
}

/// Should map 1:1 with the Registers struct
/// TODO: Can we enforce that the two data structures line up?
#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum RegisterName {
    R1 = 0,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    Lh,
    Ll,
    Ah,
    Al,
    Sh,
    Sl,
    Ph,
    Pl,
    Sr,
}

impl RegisterName {
    pub fn to_register_index(&self) -> u8 {
        num::ToPrimitive::to_u8(self).expect("Could not convert enum to u8")
    }
}

/// These three registers are special in that they can be used
/// as combined 32-bit wide registers, but only for addressing.
#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum AddressRegisterName {
    // l (lh, ll)
    LinkRegister,
    // a (ah, al)
    Address,
    // s (sh, sl)
    StackPointer,
    // p (ph, pl)
    ProgramCounter,
}

impl AddressRegisterName {
    pub fn to_register_index(&self) -> u8 {
        num::ToPrimitive::to_u8(self).expect("Could not convert enum to u8")
    }
    pub fn from_register_index(index: u8) -> AddressRegisterName {
        num::FromPrimitive::from_u8(index).expect("Could not convert u8 to enum")
    }
}

// Traits

pub trait RegisterIndexing {
    fn get_at_index(&self, index: u8) -> u16;
    fn set_at_index(&mut self, index: u8, value: u16);
}

pub trait AddressRegisterIndexing {
    fn get_address_register_at_index(&self, index: u8) -> u32;
    fn set_address_register_at_index(&mut self, index: u8, value: u32);
}

pub trait SegmentedRegisterAccess {
    fn get_segmented_link(&self) -> (u16, u16);
    fn get_segmented_pc(&self) -> (u16, u16);
    fn get_segmented_address(&self) -> (u16, u16);
    fn get_segmented_sp(&self) -> (u16, u16);

    fn set_segmented_sp(&mut self, segmented_sp: (u16, u16));
}
pub trait SegmentedAddress {
    fn to_full_address(self) -> u32;
}

pub trait FullAddressRegisterAccess {
    fn get_full_link_address(&self) -> u32;
    fn get_full_pc_address(&self) -> u32;
    fn get_full_address_address(&self) -> u32;
    fn get_full_sp_address(&self) -> u32;

    fn set_full_link_address(&mut self, address: u32);
    fn set_full_pc_address(&mut self, address: u32);
    fn set_full_address_address(&mut self, address: u32);
    fn set_full_sp_address(&mut self, address: u32);
}

pub trait FullAddress {
    fn to_segmented_address(self) -> (u16, u16);
}

#[derive(Debug, Clone, Default)]
pub struct Registers {
    pub r1: u16,
    pub r2: u16,
    pub r3: u16,
    pub r4: u16,
    pub r5: u16,
    pub r6: u16,
    pub z: u16,
    // Link Register
    pub lh: u16, // Base/segment address (8 bits concatenated with al/most significant 8 bits ignored)
    pub ll: u16,
    // Address Register
    pub ah: u16, // Base/segment address (8 bits concatenated with al/most significant 8 bits ignored)
    pub al: u16,
    // Stack Pointer (user, system) - depending on system status bit
    pub sh: (u16, u16), // Base/segment address (8 bits concatenated with sl/most significant 8 bits ignored)
    pub sl: (u16, u16),
    // Program Counter
    pub ph: u16, // Base/segment address (8 bits concatenated with pl/most significant 8 bits ignored)
    pub pl: u16,
    // Status Register
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
            0 => &self.r1,
            1 => &self.r2,
            2 => &self.r3,
            3 => &self.r4,
            4 => &self.r5,
            5 => &self.r6,
            6 => &self.z,
            7 => &self.lh,
            8 => &self.ll,
            9 => &self.ah,
            10 => &self.al,
            11 => {
                if sr_bit_is_set(StatusRegisterFields::SystemMode, self) {
                    &self.sh.1
                } else {
                    &self.sh.0
                }
            }
            12 => {
                if sr_bit_is_set(StatusRegisterFields::SystemMode, self) {
                    &self.sl.1
                } else {
                    &self.sl.0
                }
            }
            13 => &self.ph,
            14 => &self.pl,
            15 => &self.sr,
            _ => panic!("Fatal: No register mapping for index [{}]", index),
        }
    }
}

impl IndexMut<u8> for Registers {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        match index {
            0 => &mut self.r1,
            1 => &mut self.r2,
            2 => &mut self.r3,
            3 => &mut self.r4,
            4 => &mut self.r5,
            5 => &mut self.r6,
            6 => &mut self.z,
            7 => &mut self.lh,
            8 => &mut self.ll,
            9 => &mut self.ah,
            10 => &mut self.al,
            11 => {
                if sr_bit_is_set(StatusRegisterFields::SystemMode, self) {
                    &mut self.sh.1
                } else {
                    &mut self.sh.0
                }
            }
            12 => {
                if sr_bit_is_set(StatusRegisterFields::SystemMode, self) {
                    &mut self.sl.1
                } else {
                    &mut self.sl.0
                }
            }
            13 => &mut self.ph,
            14 => &mut self.pl,
            15 => &mut self.sr,
            _ => panic!("Fatal: No register mapping for index [{}]", index),
        }
    }
}

// TODO: Deprecate and remove this in favour of just using the Index/IndexMut traits
impl RegisterIndexing for Registers {
    fn get_at_index(&self, index: u8) -> u16 {
        self[index]
    }
    fn set_at_index(&mut self, index: u8, value: u16) {
        self[index] = value;
    }
}

impl AddressRegisterIndexing for Registers {
    fn get_address_register_at_index(&self, index: u8) -> u32 {
        match AddressRegisterName::from_register_index(index) {
            AddressRegisterName::LinkRegister => self.get_full_link_address(),
            AddressRegisterName::Address => self.get_full_address_address(),
            AddressRegisterName::ProgramCounter => self.get_full_pc_address(),
            AddressRegisterName::StackPointer => self.get_full_sp_address(),
        }
    }
    fn set_address_register_at_index(&mut self, index: u8, value: u32) {
        match AddressRegisterName::from_register_index(index) {
            AddressRegisterName::LinkRegister => self.set_full_link_address(value),
            AddressRegisterName::Address => self.set_full_address_address(value),
            AddressRegisterName::ProgramCounter => self.set_full_pc_address(value),
            AddressRegisterName::StackPointer => self.set_full_sp_address(value),
        }
    }
}

impl SegmentedRegisterAccess for Registers {
    fn get_segmented_link(&self) -> (u16, u16) {
        (self.lh, self.ll)
    }

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
        if high > 0x00FF {
            println!("Warning: ah is > 0x0FF. The SIRC CPU only has 24 bit addressing. The top 8 bits of the ah register will be ignored for STOR/LOAD operations.")
        }
        let high_shifted = (high as u32) << (size_of::<u16>() * u8::BITS as usize);
        // Bitwise AND with address mask to ensure that the highest 8 bits are ignored
        // This CPU only supports 24 bit addressing
        (high_shifted | low as u32) & ADDRESS_MASK
    }
}

impl FullAddressRegisterAccess for Registers {
    fn get_full_link_address(&self) -> u32 {
        self.get_segmented_link().to_full_address()
    }

    fn get_full_pc_address(&self) -> u32 {
        self.get_segmented_pc().to_full_address()
    }

    fn get_full_address_address(&self) -> u32 {
        self.get_segmented_address().to_full_address()
    }

    fn get_full_sp_address(&self) -> u32 {
        self.get_segmented_sp().to_full_address()
    }

    fn set_full_link_address(&mut self, address: u32) {
        (self.lh, self.ll) = address.to_segmented_address();
    }

    fn set_full_pc_address(&mut self, address: u32) {
        (self.ph, self.pl) = address.to_segmented_address();
    }

    fn set_full_address_address(&mut self, address: u32) {
        (self.ah, self.al) = address.to_segmented_address();
    }

    fn set_full_sp_address(&mut self, address: u32) {
        self.set_segmented_sp(address.to_segmented_address());
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
/// Returns the value of the interrupt mask in the status register as a 4-bit integer.
///
/// ```
/// use peripheral_cpu::registers::{get_interrupt_mask, Registers};
///
/// let mut registers = Registers::default();
///
/// registers.sr = 0b0000_0000 << u8::BITS;
/// assert_eq!(0, get_interrupt_mask(&registers));
/// registers.sr = 0b0000_0010 << u8::BITS;
/// assert_eq!(1, get_interrupt_mask(&registers));
/// registers.sr = 0b0000_0100 << u8::BITS;
/// assert_eq!(2, get_interrupt_mask(&registers));
/// registers.sr = 0b0000_0110 << u8::BITS;
/// assert_eq!(3, get_interrupt_mask(&registers));
/// registers.sr = 0b0000_1000 << u8::BITS;
/// assert_eq!(4, get_interrupt_mask(&registers));
/// registers.sr = 0b0000_1010 << u8::BITS;
/// assert_eq!(5, get_interrupt_mask(&registers));
/// registers.sr = 0b0000_1100 << u8::BITS;
/// assert_eq!(6, get_interrupt_mask(&registers));
/// registers.sr = 0b0000_1110 << u8::BITS;
/// assert_eq!(7, get_interrupt_mask(&registers));
/// ```
pub fn get_interrupt_mask(registers: &Registers) -> u8 {
    let bit_mask = StatusRegisterFields::InterruptMaskHigh as u16
        | StatusRegisterFields::InterruptMaskMed as u16
        | StatusRegisterFields::InterruptMaskLow as u16;
    let masked_bits = registers.sr & bit_mask;
    // TODO: Can we work out this shift based on the fields position?
    (masked_bits >> 9) as u8
}

///
/// Sets the value of the interrupt mask in the status register as a 4-bit integer.
///
/// ```
/// use peripheral_cpu::registers::{set_interrupt_mask, Registers};
///
/// let mut registers = Registers::default();
/// registers.sr = 0b0001_0001 << u8::BITS;
///
/// set_interrupt_mask(&mut registers, 0);
/// assert_eq!(0b0001_0001 << u8::BITS, registers.sr);
/// set_interrupt_mask(&mut registers, 1);
/// assert_eq!(0b0001_0011 << u8::BITS, registers.sr);
/// set_interrupt_mask(&mut registers, 2);
/// assert_eq!(0b0001_0101 << u8::BITS, registers.sr);
/// set_interrupt_mask(&mut registers, 3);
/// assert_eq!(0b0001_0111 << u8::BITS, registers.sr);
/// set_interrupt_mask(&mut registers, 4);
/// assert_eq!(0b0001_1001 << u8::BITS, registers.sr);
/// set_interrupt_mask(&mut registers, 5);
/// assert_eq!(0b0001_1011 << u8::BITS, registers.sr);
/// set_interrupt_mask(&mut registers, 6);
/// assert_eq!(0b0001_1101 << u8::BITS, registers.sr);
/// set_interrupt_mask(&mut registers, 7);
/// assert_eq!(0b0001_1111 << u8::BITS, registers.sr);
/// set_interrupt_mask(&mut registers, 8);
/// assert_eq!(0b0001_1111 << u8::BITS, registers.sr);
/// ```
pub fn set_interrupt_mask(registers: &mut Registers, interrupt_mask: u8) {
    // TODO: Can we work out this shift based on the fields position?
    let shifted_value = (interrupt_mask.clamp(0, 7) as u16) << 9;
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
        "r1" => 0,
        "r2" => 1,
        "r3" => 2,
        "r4" => 3,
        "r5" => 4,
        "r6" => 5,
        "r7" => 6,
        // [ah, al]
        "lh" => 7, // Link high
        "ll" => 8, // Link low
        // [ah, al]
        "ah" => 9,  // Address high
        "al" => 10, // Address low
        // [sh, sl]
        "sh" => 11, // Stack pointer high
        "sl" => 12, // Stack pointer low
        // [ph, pl]
        "ph" => 13, // Program counter high
        "pl" => 14, // Program counter low
        "sr" => 15,
        _ => panic!("Fatal: No register mapping for name [{}]", name),
    }
}

///
/// Returns true if start_index/end_index form a valid range of registers,
/// otherwise false.
///
/// ```
/// use peripheral_cpu::registers::is_valid_register_range;
///
/// assert!(is_valid_register_range(0, 15));
/// assert!(is_valid_register_range(1, 4));
/// assert!(!is_valid_register_range(0, 16));
/// assert!(!is_valid_register_range(2, 1));
/// assert!(!is_valid_register_range(0, 0));
///
/// ```
pub fn is_valid_register_range(start_index: u8, end_index: u8) -> bool {
    // TODO: Use std::mem::variant_count::<RegisterName>() when it stabilises https://stackoverflow.com/a/73543241/1153203
    let register_count = 16;
    start_index < register_count && end_index < register_count && end_index > start_index
}
