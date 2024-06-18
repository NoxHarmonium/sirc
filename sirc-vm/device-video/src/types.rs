pub enum TileSize {
    EightByEight,
    SixteenBySixteen,
}

pub struct BaseConfig {
    pub disable_graphics: bool,
    pub disable_b1: bool,
    pub disable_b2: bool,
    pub disable_b3: bool,
    pub disable_s1: bool,
    pub disable_s2: bool,
    pub disable_s3: bool,
    /// 16 values (4 bits)
    pub screen_brightness: u8,
}

pub struct TileSizes {
    pub b1_size: TileSize,
    pub b2_size: TileSize,
    pub b3_size: TileSize,
    pub s1_size: TileSize,
    pub s2_size: TileSize,
    pub s3_size: TileSize,
}

pub struct BgRegisters {
    pub b1: u16,
    pub b2: u16,
    pub b3: u16,
}

pub struct VideoRegisters {
    /// 0x0000
    pub base_config: BaseConfig,
    /// 0x0001
    pub tile_sizes: TileSizes,
    /// 0x0002-0x0005
    pub bg_tilemap_base_addresses: BgRegisters,
    /// 0x0006-0x0009
    pub bg_tile_base_addresses: BgRegisters,
    /// 0x000A-0x000D (10 bit)
    pub bg_scroll_x: BgRegisters,
    /// 0x000E-0x0011 (10 bit)
    pub bg_scroll_y: BgRegisters,
    /// 0x0012
    pub sprite_tile_base_address: u16,
}
