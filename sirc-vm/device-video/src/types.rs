// Suppress warnings caused by macro code
#![allow(redundant_semicolons, dead_code)]

use modular_bitfield::prelude::*;
use peripheral_bus::memory_mapped_device::MemoryMapped;

const READONLY_STATUS_REGISTER_ADDR: u32 = 0x0013;
// TODO: Use exclusive range in match when rust 1.80 is released
// category=Refactoring
// Then we can get rid of this. See also: https://github.com/rust-lang/rust/issues/37854
const READONLY_STATUS_REGISTER_ADDR_MINUS_ONE: u32 = READONLY_STATUS_REGISTER_ADDR - 1;
#[bitfield(bytes = 2)]
#[derive(Debug, Default)]
pub struct BaseConfigRegister {
    pub graphics_disable: B1,
    pub b1_disable: B1,
    pub b2_disable: B1,
    pub b3_disable: B1,
    pub reserved_1: B1,
    pub s1_disable: B1,
    pub s2_disable: B1,
    pub s3_disable: B1,
    pub reserved_2: B1,
    pub screen_brightness: B4,
    pub reserved_3: B3,
}

#[derive(BitfieldSpecifier, Debug)]
pub enum TileSize {
    EightByEight,
    SixteenBySixteen,
}

#[derive(BitfieldSpecifier, Debug)]
pub enum TilemapSize {
    Single,
    Double,
}

#[bitfield(bytes = 2)]
#[derive(Debug, Default)]
pub struct TileSizeRegister {
    #[bits = 1]
    pub b1_size: TileSize,
    #[bits = 1]
    pub b2_size: TileSize,
    #[bits = 1]
    pub b3_size: TileSize,
    pub reserved_1: B1,
    //
    #[bits = 1]
    pub s1_size: TileSize,
    #[bits = 1]
    pub s2_size: TileSize,
    #[bits = 1]
    pub s3_size: TileSize,
    pub reserved_2: B1,
    //
    #[bits = 1]
    pub b1_tilemap_size_x: TilemapSize,
    #[bits = 1]
    pub b2_tilemap_size_x: TilemapSize,
    #[bits = 1]
    pub b3_tilemap_size_x: TilemapSize,
    pub reserved_3: B1,
    //
    #[bits = 1]
    pub b1_tilemap_size_y: TilemapSize,
    #[bits = 1]
    pub b2_tilemap_size_y: TilemapSize,
    #[bits = 1]
    pub b3_tilemap_size_y: TilemapSize,
    pub reserved_4: B1,
}

#[bitfield(bytes = 2)]
#[derive(Debug, Default)]
pub struct ScrollRegister {
    pub scroll_amount: B10,
    pub reserved: B6,
}

#[derive(BitfieldSpecifier, Debug)]
pub enum OutputMode {
    Ntsc,
    Pal,
}

#[bitfield(bytes = 2)]
#[derive(Debug, Default)]
pub struct StatusRegister {
    pub ppu_version: B4,
    #[bits = 1]
    pub output_mode: OutputMode,
    pub reserved: B11,
}

#[derive(Debug, Default)]
pub struct PpuRegisters {
    pub base_config: BaseConfigRegister,
    pub tile_size: TileSizeRegister,
    pub b1_tilemap_addr: u16,
    pub b2_tilemap_addr: u16,
    pub b3_tilemap_addr: u16,
    pub reserved: u16,
    pub b1_tile_addr: u16,
    pub b2_tile_addr: u16,
    pub b3_tile_addr: u16,
    pub reserved2: u16,
    pub b1_scroll_x: ScrollRegister,
    pub b2_scroll_x: ScrollRegister,
    pub b3_scroll_x: ScrollRegister,
    pub reserved3: u16,
    pub b1_scroll_y: ScrollRegister,
    pub b2_scroll_y: ScrollRegister,
    pub b3_scroll_y: ScrollRegister,
    pub reserved4: u16,
    pub s_tile_addr: u16,
    pub status: StatusRegister,
}

impl MemoryMapped for PpuRegisters {
    fn read_address(&self, address: u32) -> u16 {
        // The modular_bitfield crate only provides bytes for raw output, hopefully the compiler can
        // optimise it so it doesn't matter
        match address {
            0x0000 => u16::from_be_bytes(self.base_config.bytes),
            0x0001 => u16::from_be_bytes(self.tile_size.bytes),
            0x0002 => self.b1_tilemap_addr,
            0x0003 => self.b2_tilemap_addr,
            0x0004 => self.b3_tilemap_addr,
            0x0005 => self.reserved,
            0x0006 => self.b1_tile_addr,
            0x0007 => self.b2_tile_addr,
            0x0008 => self.b3_tile_addr,
            0x0009 => self.reserved2,
            0x000A => u16::from_be_bytes(self.b1_scroll_x.bytes),
            0x000B => u16::from_be_bytes(self.b2_scroll_x.bytes),
            0x000C => u16::from_be_bytes(self.b3_scroll_x.bytes),
            0x000D => self.reserved3,
            0x000E => u16::from_be_bytes(self.b1_scroll_y.bytes),
            0x000F => u16::from_be_bytes(self.b2_scroll_y.bytes),
            0x0010 => u16::from_be_bytes(self.b3_scroll_y.bytes),
            0x0011 => self.reserved4,
            0x0012 => self.s_tile_addr,
            READONLY_STATUS_REGISTER_ADDR => u16::from_be_bytes(self.status.bytes),
            _ => 0x0, // Open bus
        }
    }

    fn write_address(&mut self, address: u32, value: u16) {
        // The modular_bitfield crate only accepts bytes for raw input, hopefully the compiler can
        // optimise it so it doesn't matter
        let bytes = u16::to_be_bytes(value);
        match address {
            0x0000 => self.base_config.bytes = bytes,
            0x0001 => self.tile_size.bytes = bytes,
            0x0002 => self.b1_tilemap_addr = value,
            0x0003 => self.b2_tilemap_addr = value,
            0x0004 => self.b3_tilemap_addr = value,
            0x0005 => self.reserved = value,
            0x0006 => self.b1_tile_addr = value,
            0x0007 => self.b2_tile_addr = value,
            0x0008 => self.b3_tile_addr = value,
            0x0009 => self.reserved2 = value,
            0x000A => self.b1_scroll_x.bytes = bytes,
            0x000B => self.b2_scroll_x.bytes = bytes,
            0x000C => self.b3_scroll_x.bytes = bytes,
            0x000D => self.reserved3 = value,
            0x000E => self.b1_scroll_y.bytes = bytes,
            0x000F => self.b2_scroll_y.bytes = bytes,
            0x0010 => self.b3_scroll_y.bytes = bytes,
            0x0011 => self.reserved4 = value,
            0x0012 => self.s_tile_addr = value,
            // 0x0013 Read Only (status)
            _ => {} // Open bus
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_round_trip() {
        let mut ppu_registers = PpuRegisters::default();
        for addr in 0..=0xFF {
            let sentinel_value = 0xFF - addr as u16;
            ppu_registers.write_address(addr, sentinel_value);
            let stored_value = ppu_registers.read_address(addr);
            match addr {
                0x0..=READONLY_STATUS_REGISTER_ADDR_MINUS_ONE => {
                    assert_eq!(sentinel_value, stored_value)
                }
                _ => assert_eq!(0x0, stored_value),
            }
        }
    }
}
