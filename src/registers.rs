pub trait RegisterIndexing {
    fn get_at_index(&mut self, index: u8) -> u16;
    fn set_at_index(&mut self, index: u8, value: u16);
}

pub enum StatusRegisterFields {
    LastComparisonResult = 0x01,
    CpuHalted = 0x2,
}

#[derive(Debug, Clone)]
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
    pub a1: u16,
    pub a2: u16,
    pub a3: u16,
    pub pc: u16,
    pub sp: u16,
    // Status Register
    // 0 - Last comparison result (e.g. 1 was success)
    // 1 - Carry bit
    // 2 - Overflow bit
    pub sr: u16,
}

// TODO: Could probably use a macro or something to do this
// E.g. derive(debug) can access struct fields
impl RegisterIndexing for Registers {
    fn get_at_index(&mut self, index: u8) -> u16 {
        match index {
            0 => self.x1,
            1 => self.x2,
            2 => self.x3,
            3 => self.y1,
            4 => self.y2,
            5 => self.y3,
            6 => self.z1,
            7 => self.z2,
            8 => self.z3,
            9 => self.a1,
            10 => self.a2,
            11 => self.a3,
            12 => self.pc,
            13 => self.sp,
            14 => self.sr,
            _ => panic!("Fatal: No register mapping for index [{}]", index),
        }
    }
    fn set_at_index(&mut self, index: u8, value: u16) {
        match index {
            0 => self.x1 = value,
            1 => self.x2 = value,
            2 => self.x3 = value,
            3 => self.y1 = value,
            4 => self.y2 = value,
            5 => self.y3 = value,
            6 => self.z1 = value,
            7 => self.z2 = value,
            8 => self.z3 = value,
            9 => self.a1 = value,
            10 => self.a2 = value,
            11 => self.a3 = value,
            12 => self.pc = value,
            13 => self.sp = value,
            14 => self.sr = value,
            _ => panic!("Fatal: No register mapping for index [{}]", index),
        }
    }
}

pub fn new_registers() -> Registers {
    Registers {
        x1: 0x0000,
        x2: 0x0000,
        x3: 0x0000,
        y1: 0x0000,
        y2: 0x0000,
        y3: 0x0000,
        z1: 0x0000,
        z2: 0x0000,
        z3: 0x0000,
        a1: 0x0000,
        a2: 0x0000,
        a3: 0x0000,
        pc: 0x0000,
        sp: 0x0000,
        sr: 0x0000,
    }
}

pub fn sr_bit_is_set(bit: StatusRegisterFields, registers: &Registers) -> bool {
    let bit_mask = bit as u16;
    registers.sr & bit_mask == bit_mask
}

pub fn set_sr_bit(bit: StatusRegisterFields, registers: &mut Registers) {
    let bit_mask = bit as u16;
    registers.sr |= bit_mask
}

pub fn clear_sr_bit(bit: StatusRegisterFields, registers: &mut Registers) {
    let bit_mask = bit as u16;
    registers.sr &= !bit_mask
}
