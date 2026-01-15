// Suppress warnings caused by macro code
#![allow(redundant_semicolons, dead_code, unused_parens)]

use modular_bitfield::prelude::*;
use peripheral_bus::memory_mapped_device::MemoryMapped;
use std::ops::{Index, IndexMut};

const READONLY_STATUS_REGISTER_ADDR: u32 = 0x0017;

// NOTE: The structs are laid out backwards because of the way bitfield works
// The memory layout in the actual system will be the reverse (see the wiki)

/// The PPU supports 15 bit colour (5 bbp)
#[bitfield(bits = 16)]
#[repr(u16)]
#[derive(Debug, Default, Clone, Copy)]
pub struct PpuPixel {
    pub b: B5,
    pub g: B5,
    pub r: B5,
    pub f: B1, // f=flag - reserved
}

#[bitfield(bits = 16)]
#[repr(u16)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TilemapEntry {
    pub tile_index: B10,     // Bits 0-9 (LSB)
    pub palette_select: B3,  // Bits 10-12
    pub priority: B1,        // Bit 13
    pub flip_horizontal: B1, // Bit 14
    pub flip_vertical: B1,   // Bit 15 (MSB)
}

#[bitfield(bits = 16)]
#[repr(u16)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TileLine {
    // TODO: Rename these to pixel because p could mean palette or pixel
    pub p4: B4,
    pub p3: B4,
    pub p2: B4,
    pub p1: B4,
}

pub const PIXEL_BUFFER_SIZE: u16 = 4;
#[derive(Debug, Default)]
pub struct PixelBuffer {
    pub p1: PpuPixel,
    pub p2: PpuPixel,
    pub p3: PpuPixel,
    pub p4: PpuPixel,
}

impl Index<u8> for PixelBuffer {
    type Output = PpuPixel;

    fn index(&self, index: u8) -> &Self::Output {
        match index {
            0 => &self.p1,
            1 => &self.p2,
            2 => &self.p3,
            3 => &self.p4,
            _ => panic!("Fatal: No register mapping for index [{index}]"),
        }
    }
}

impl IndexMut<u8> for PixelBuffer {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        match index {
            0 => &mut self.p1,
            1 => &mut self.p2,
            2 => &mut self.p3,
            3 => &mut self.p4,
            _ => panic!("Fatal: No pixel buffer mapping for index [{index}]"),
        }
    }
}

// 0
// 1
// 2
// 3-5
// 6-F
// Flip Vertical
// Flip Horizontal
// Priority
// Palette Selection
// Tile index

#[derive(Debug, Eq, PartialEq)]
pub enum Backgrounds {
    Bg1,
    Bg2,
    Bg3,
}

#[bitfield(bits = 16)]
#[repr(u16)]
#[derive(Debug, Default, Clone, Copy)]
pub struct BaseConfigRegister {
    // Note: Fields declared in reverse order from documentation!
    pub reserved_3: B3,        // Bits 0-2
    pub screen_brightness: B4, // Bits 3-6
    pub reserved_2: B1,        // Bit 7
    pub s3_disable: B1,        // Bit 8
    pub s2_disable: B1,        // Bit 9
    pub s1_disable: B1,        // Bit 10
    pub reserved_1: B1,        // Bit 11
    pub b3_disable: B1,        // Bit 12
    pub b2_disable: B1,        // Bit 13
    pub b1_disable: B1,        // Bit 14
    pub graphics_disable: B1,  // Bit 15 (MSB)
}

#[derive(Specifier, Debug)]
pub enum TileSize {
    EightByEight,
    SixteenBySixteen,
}

#[derive(Specifier, Debug)]
pub enum TilemapSize {
    Single,
    Double,
}

#[derive(Specifier, Debug)]
pub enum PaletteSize {
    Four,
    Eight,
    Sixteen,
    ThirtyTwo,
}

#[bitfield(bits = 16)]
#[repr(u16)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TileSizeRegister {
    // Note: Fields declared in reverse order from documentation!
    pub reserved_4: B1,
    pub b3_tilemap_size_y: TilemapSize,
    pub b2_tilemap_size_y: TilemapSize,
    pub b1_tilemap_size_y: TilemapSize,
    pub reserved_3: B1,
    pub b3_tilemap_size_x: TilemapSize,
    pub b2_tilemap_size_x: TilemapSize,
    pub b1_tilemap_size_x: TilemapSize,
    pub reserved_2: B1,
    pub s3_size: TileSize,
    pub s2_size: TileSize,
    pub s1_size: TileSize,
    pub reserved_1: B1,
    pub b3_size: TileSize,
    pub b2_size: TileSize,
    pub b1_size: TileSize,
}

#[bitfield(bits = 16)]
#[repr(u16)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ScrollRegister {
    // Note: Fields declared in reverse order from documentation!
    pub scroll_amount: B10,
    pub reserved: B6,
}

#[bitfield(bits = 16)]
#[repr(u16)]
#[derive(Debug, Default, Clone, Copy)]
pub struct PaletteRegister {
    // Note: Fields declared in reverse order from documentation!
    pub palette_size: PaletteSize,
    pub reserved: B6,
    pub palette_offset: u8,
}

#[derive(Specifier, Debug)]
pub enum OutputMode {
    Ntsc,
    Pal,
}

#[bitfield(bits = 16)]
#[repr(u16)]
#[derive(Debug, Default)]
pub struct StatusRegister {
    // Note: Fields declared in reverse order from documentation!
    pub ppu_version: B4,
    pub output_mode: OutputMode,
    pub reserved: B11,
}

#[derive(Debug, Default)]
pub struct PpuRegisters {
    pub base_config: BaseConfigRegister,
    pub tile_size: TileSizeRegister,
    pub b1_tilemap_addr: u16, // Base address to read the tilemap from (the tile definitions)
    pub b2_tilemap_addr: u16,
    pub b3_tilemap_addr: u16,
    pub reserved: u16,
    pub b1_tile_addr: u16, // Base address to read the actual tile data from
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
    pub b1_palette_config: PaletteRegister,
    pub b2_palette_config: PaletteRegister,
    pub b3_palette_config: PaletteRegister,
    pub reserved5: u16,
    pub s_tile_addr: u16,
    pub status: StatusRegister,
}

impl MemoryMapped for PpuRegisters {
    fn read_address(&self, address: u32) -> u16 {
        // The modular_bitfield crate only provides bytes for raw output, hopefully the compiler can
        // optimise it so it doesn't matter
        match address {
            0x0000 => self.base_config.into(),
            0x0001 => self.tile_size.into(),
            0x0002 => self.b1_tilemap_addr,
            0x0003 => self.b2_tilemap_addr,
            0x0004 => self.b3_tilemap_addr,
            0x0005 => self.reserved,
            0x0006 => self.b1_tile_addr,
            0x0007 => self.b2_tile_addr,
            0x0008 => self.b3_tile_addr,
            0x0009 => self.reserved2,
            0x000A => self.b1_scroll_x.into(),
            0x000B => self.b2_scroll_x.into(),
            0x000C => self.b3_scroll_x.into(),
            0x000D => self.reserved3,
            0x000E => self.b1_scroll_y.into(),
            0x000F => self.b2_scroll_y.into(),
            0x0010 => self.b3_scroll_y.into(),
            0x0011 => self.reserved4,
            0x0012 => self.b1_palette_config.into(),
            0x0013 => self.b2_palette_config.into(),
            0x0014 => self.b3_palette_config.into(),
            0x0015 => self.reserved5,
            0x0016 => self.s_tile_addr,
            READONLY_STATUS_REGISTER_ADDR => u16::from_be_bytes(self.status.bytes),
            _ => 0x0, // Open bus
        }
    }

    fn write_address(&mut self, address: u32, value: u16) {
        match address {
            0x0000 => self.base_config = value.into(),
            0x0001 => self.tile_size = value.into(),
            0x0002 => self.b1_tilemap_addr = value,
            0x0003 => self.b2_tilemap_addr = value,
            0x0004 => self.b3_tilemap_addr = value,
            0x0005 => self.reserved = value,
            0x0006 => self.b1_tile_addr = value,
            0x0007 => self.b2_tile_addr = value,
            0x0008 => self.b3_tile_addr = value,
            0x0009 => self.reserved2 = value,
            0x000A => self.b1_scroll_x = value.into(),
            0x000B => self.b2_scroll_x = value.into(),
            0x000C => self.b3_scroll_x = value.into(),
            0x000D => self.reserved3 = value,
            0x000E => self.b1_scroll_y = value.into(),
            0x000F => self.b2_scroll_y = value.into(),
            0x0010 => self.b3_scroll_y = value.into(),
            0x0011 => self.reserved4 = value,
            0x0012 => self.b1_palette_config = value.into(),
            0x0013 => self.b2_palette_config = value.into(),
            0x0014 => self.b3_palette_config = value.into(),
            0x0015 => self.reserved5 = value,
            0x0016 => self.s_tile_addr = value,
            // 0x0017 Read Only (status)
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
            let sentinel_value =
                u16::try_from(0xFF - addr).expect("sentinel_value should fit into 16 bits");
            ppu_registers.write_address(addr, sentinel_value);
            let stored_value = ppu_registers.read_address(addr);
            match addr {
                0x0..READONLY_STATUS_REGISTER_ADDR => {
                    assert_eq!(sentinel_value, stored_value);
                }
                _ => assert_eq!(0x0, stored_value),
            }
        }
    }
}
