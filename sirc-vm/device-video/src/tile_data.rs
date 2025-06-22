use crate::types::PpuPixel;

pub const PALETTE_1: [PpuPixel; 16] = [
    // Note that the bytes are reversed from what you would expect
    // The simulator hosts are little endian (or at least the library that provides
    // the from_bytes function is.)
    // Legend of which bits are for which colour: [gggbbbbb, xrrrrrgg]
    PpuPixel::from_bytes([0b0000_0000, 0b0000_0000]), //    (TRANSP_aren
    PpuPixel::from_bytes([0b0000_0000, 0b0000_0000]), // ███ (BLACK__)
    PpuPixel::from_bytes([0b1111_1111, 0b0111_1111]), // ███ (WHITE__)
    PpuPixel::from_bytes([0b0000_0000, 0b0111_1100]), // ███ (RED____)
    PpuPixel::from_bytes([0b1110_0000, 0b0000_0011]), // ███ (GREEN__)
    PpuPixel::from_bytes([0b0001_1111, 0b0000_0000]), // ███ (BLUE___)
    PpuPixel::from_bytes([0b1110_0000, 0b0111_1111]), // ███ (YELLOW_)
    PpuPixel::from_bytes([0b1111_1111, 0b0000_0011]), // ███ (CYAN___)
    PpuPixel::from_bytes([0b0001_1111, 0b0111_1100]), // ███ (magenta)
    PpuPixel::from_bytes([0b1000_0000, 0b0111_1101]), // ███ (ORANGE_)
    PpuPixel::from_bytes([0b0000_1100, 0b0011_0000]), // Purple
    PpuPixel::from_bytes([0b0000_0000, 0b0000_0000]), // Reserved
    PpuPixel::from_bytes([0b0000_0000, 0b0000_0000]), // Reserved
    PpuPixel::from_bytes([0b0000_0000, 0b0000_0000]), // Reserved
    PpuPixel::from_bytes([0b0000_0000, 0b0000_0000]), // Reserved
    PpuPixel::from_bytes([0b0000_0000, 0b0000_0000]), // Reserved
];

// Indexes into PALLETE_1 (should be 0-15)
pub const TRANSP_: u8 = 0;
pub const BLACK__: u8 = 1;
pub const WHITE__: u8 = 2;
pub const RED____: u8 = 3;
pub const GREEN__: u8 = 4;
pub const BLUE___: u8 = 5;
pub const YELLOW_: u8 = 6;
pub const CYAN___: u8 = 7;
pub const MAGENTA: u8 = 8;
pub const ORANGE_: u8 = 9;
pub const PURPLE_: u8 = 10;

/// 8x8 tiles for digits 0-9 (64 pixels each in big-endian 15-bit RGB)
/// TODO: This is wrong! Each pixel is actually 4 bits so it should be a 8x8 array of u4 _OR_
/// a 2x2 array of u16 with 4 pixels packed into each value? Anyway, fix this!
pub const DIGIT_TILES: [[u8; 64]; 10] = [
    // Digit 0 (WHITE__ on BLUE___)
    [
        WHITE__, WHITE__, WHITE__, WHITE__, WHITE__, WHITE__, WHITE__, WHITE__, WHITE__, WHITE__,
        BLUE___, BLUE___, BLUE___, BLUE___, WHITE__, WHITE__, WHITE__, BLUE___, WHITE__, WHITE__,
        WHITE__, WHITE__, BLUE___, WHITE__, WHITE__, BLUE___, WHITE__, WHITE__, WHITE__, WHITE__,
        BLUE___, WHITE__, WHITE__, BLUE___, WHITE__, WHITE__, WHITE__, WHITE__, BLUE___, WHITE__,
        WHITE__, BLUE___, WHITE__, WHITE__, WHITE__, WHITE__, BLUE___, WHITE__, WHITE__, WHITE__,
        BLUE___, BLUE___, BLUE___, BLUE___, WHITE__, WHITE__, WHITE__, WHITE__, WHITE__, WHITE__,
        WHITE__, WHITE__, WHITE__, WHITE__,
    ],
    // Digit 1 (RED____)
    [
        BLACK__, BLACK__, BLACK__, RED____, RED____, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        RED____, RED____, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, RED____, RED____, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, RED____, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, RED____, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, RED____, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, RED____,
        RED____, RED____, RED____, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__,
    ],
    // Digit 2 (GREEN__)
    [
        BLACK__, BLACK__, GREEN__, GREEN__, GREEN__, BLACK__, BLACK__, BLACK__, GREEN__, GREEN__,
        BLACK__, BLACK__, BLACK__, GREEN__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, GREEN__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, GREEN__, GREEN__, BLACK__,
        BLACK__, BLACK__, BLACK__, GREEN__, GREEN__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        GREEN__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, GREEN__, GREEN__,
        GREEN__, GREEN__, GREEN__, GREEN__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__,
    ],
    // Digit 3 (YELLOW_)
    [
        BLACK__, BLACK__, YELLOW_, YELLOW_, YELLOW_, BLACK__, BLACK__, BLACK__, YELLOW_, YELLOW_,
        BLACK__, BLACK__, BLACK__, YELLOW_, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        YELLOW_, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, YELLOW_, YELLOW_, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, YELLOW_, BLACK__, BLACK__, BLACK__,
        YELLOW_, YELLOW_, BLACK__, BLACK__, BLACK__, YELLOW_, BLACK__, BLACK__, BLACK__, BLACK__,
        YELLOW_, YELLOW_, YELLOW_, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__,
    ],
    // Digit 4 (CYAN___)
    [
        BLACK__, BLACK__, BLACK__, BLACK__, CYAN___, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, CYAN___, CYAN___, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, CYAN___, BLACK__,
        CYAN___, BLACK__, BLACK__, BLACK__, CYAN___, CYAN___, CYAN___, CYAN___, CYAN___, CYAN___,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, CYAN___, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, CYAN___, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, CYAN___, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__,
    ],
    // Digit 5 (Magenta)
    [
        BLACK__, BLACK__, MAGENTA, MAGENTA, MAGENTA, BLACK__, BLACK__, BLACK__, MAGENTA, MAGENTA,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, MAGENTA, MAGENTA, MAGENTA, MAGENTA,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, MAGENTA, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, MAGENTA, BLACK__, BLACK__, BLACK__,
        MAGENTA, MAGENTA, BLACK__, BLACK__, MAGENTA, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        MAGENTA, MAGENTA, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__,
    ],
    // Digit 6 (ORANGE_)
    [
        BLACK__, BLACK__, ORANGE_, ORANGE_, ORANGE_, BLACK__, BLACK__, BLACK__, ORANGE_, ORANGE_,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, ORANGE_, ORANGE_, ORANGE_, ORANGE_,
        BLACK__, BLACK__, BLACK__, BLACK__, ORANGE_, ORANGE_, BLACK__, BLACK__, ORANGE_, BLACK__,
        BLACK__, BLACK__, ORANGE_, ORANGE_, BLACK__, BLACK__, ORANGE_, BLACK__, BLACK__, BLACK__,
        ORANGE_, ORANGE_, BLACK__, BLACK__, ORANGE_, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        ORANGE_, ORANGE_, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__,
    ],
    // Digit 7 (PURPLE_)
    [
        BLACK__, BLACK__, PURPLE_, PURPLE_, PURPLE_, PURPLE_, BLACK__, BLACK__, PURPLE_, PURPLE_,
        BLACK__, BLACK__, BLACK__, PURPLE_, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        PURPLE_, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, PURPLE_, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, PURPLE_, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, PURPLE_, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, PURPLE_,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__,
    ],
    // Digit 8 (RED____)
    [
        BLACK__, RED____, RED____, RED____, RED____, RED____, BLACK__, BLACK__, RED____, RED____,
        BLACK__, BLACK__, BLACK__, RED____, RED____, BLACK__, RED____, BLACK__, RED____, RED____,
        RED____, BLACK__, RED____, BLACK__, RED____, RED____, BLACK__, BLACK__, BLACK__, RED____,
        RED____, BLACK__, RED____, RED____, BLACK__, BLACK__, BLACK__, RED____, RED____, BLACK__,
        RED____, BLACK__, RED____, RED____, RED____, BLACK__, RED____, BLACK__, RED____, RED____,
        BLACK__, BLACK__, BLACK__, RED____, RED____, BLACK__, BLACK__, RED____, RED____, RED____,
        RED____, RED____, BLACK__, BLACK__,
    ],
    // Digit 9 (WHITE__ on BLACK__)
    [
        TRANSP_, WHITE__, WHITE__, WHITE__, WHITE__, WHITE__, WHITE__, TRANSP_,
        WHITE__, TRANSP_, TRANSP_, BLACK__, TRANSP_, TRANSP_, WHITE__, TRANSP_,
        WHITE__, WHITE__, BLACK__, BLACK__, TRANSP_, TRANSP_, WHITE__, TRANSP_,
        TRANSP_, WHITE__, WHITE__, WHITE__, WHITE__, WHITE__, BLACK__, TRANSP_,
        TRANSP_, TRANSP_, TRANSP_, TRANSP_, TRANSP_, BLACK__, WHITE__, TRANSP_,
        TRANSP_, TRANSP_, TRANSP_, TRANSP_, BLACK__, WHITE__, TRANSP_, TRANSP_,
        TRANSP_, WHITE__, WHITE__, WHITE__, WHITE__, WHITE__, TRANSP_, TRANSP_,
        TRANSP_, TRANSP_, TRANSP_, TRANSP_, TRANSP_, TRANSP_, TRANSP_, TRANSP_,
    ]
];

/// Test patterns (8x8 tiles)
pub const TEST_PATTERNS: [[u8; 64]; 10] = [
    // Checkerboard (B/W)
    [
        WHITE__, BLACK__, WHITE__, BLACK__, WHITE__, BLACK__, WHITE__, BLACK__, BLACK__, WHITE__,
        BLACK__, WHITE__, BLACK__, WHITE__, BLACK__, WHITE__, WHITE__, BLACK__, WHITE__, BLACK__,
        WHITE__, BLACK__, WHITE__, BLACK__, BLACK__, WHITE__, BLACK__, WHITE__, BLACK__, WHITE__,
        BLACK__, WHITE__, WHITE__, BLACK__, WHITE__, BLACK__, WHITE__, BLACK__, WHITE__, BLACK__,
        BLACK__, WHITE__, BLACK__, WHITE__, BLACK__, WHITE__, BLACK__, WHITE__, WHITE__, BLACK__,
        WHITE__, BLACK__, WHITE__, BLACK__, WHITE__, BLACK__, BLACK__, WHITE__, BLACK__, WHITE__,
        BLACK__, WHITE__, BLACK__, WHITE__,
    ],
    // Color Bars (Rainbow)
    [
        RED____, RED____, RED____, RED____, RED____, RED____, RED____, RED____, ORANGE_, ORANGE_,
        ORANGE_, ORANGE_, ORANGE_, ORANGE_, ORANGE_, ORANGE_, YELLOW_, YELLOW_, YELLOW_, YELLOW_,
        YELLOW_, YELLOW_, YELLOW_, YELLOW_, GREEN__, GREEN__, GREEN__, GREEN__, GREEN__, GREEN__,
        GREEN__, GREEN__, CYAN___, CYAN___, CYAN___, CYAN___, CYAN___, CYAN___, CYAN___, CYAN___,
        BLUE___, BLUE___, BLUE___, BLUE___, BLUE___, BLUE___, BLUE___, BLUE___, PURPLE_, PURPLE_,
        PURPLE_, PURPLE_, PURPLE_, PURPLE_, PURPLE_, PURPLE_, MAGENTA, MAGENTA, MAGENTA, MAGENTA,
        MAGENTA, MAGENTA, MAGENTA, MAGENTA,
    ],
    // Smiley face
    [
        TRANSP_, TRANSP_, YELLOW_, YELLOW_, YELLOW_, YELLOW_, TRANSP_, TRANSP_, TRANSP_, YELLOW_,
        YELLOW_, YELLOW_, YELLOW_, YELLOW_, YELLOW_, TRANSP_, YELLOW_, YELLOW_, BLACK__, YELLOW_,
        YELLOW_, BLACK__, YELLOW_, YELLOW_, YELLOW_, YELLOW_, YELLOW_, YELLOW_, YELLOW_, YELLOW_,
        YELLOW_, YELLOW_, YELLOW_, YELLOW_, YELLOW_, BLACK__, BLACK__, YELLOW_, YELLOW_, YELLOW_,
        YELLOW_, YELLOW_, YELLOW_, YELLOW_, YELLOW_, YELLOW_, YELLOW_, YELLOW_, TRANSP_, YELLOW_,
        YELLOW_, YELLOW_, YELLOW_, YELLOW_, YELLOW_, TRANSP_, TRANSP_, TRANSP_, YELLOW_, YELLOW_,
        YELLOW_, YELLOW_, TRANSP_, TRANSP_,
    ],
    // TRANSP_arency Test
    [
        RED____, TRANSP_, RED____, TRANSP_, RED____, TRANSP_, RED____, TRANSP_, TRANSP_, GREEN__,
        TRANSP_, GREEN__, TRANSP_, GREEN__, TRANSP_, GREEN__, BLUE___, TRANSP_, BLUE___, TRANSP_,
        BLUE___, TRANSP_, BLUE___, TRANSP_, TRANSP_, YELLOW_, TRANSP_, YELLOW_, TRANSP_, YELLOW_,
        TRANSP_, YELLOW_, CYAN___, TRANSP_, CYAN___, TRANSP_, CYAN___, TRANSP_, CYAN___, TRANSP_,
        TRANSP_, MAGENTA, TRANSP_, MAGENTA, TRANSP_, MAGENTA, TRANSP_, MAGENTA, WHITE__, TRANSP_,
        WHITE__, TRANSP_, WHITE__, TRANSP_, WHITE__, TRANSP_, TRANSP_, BLACK__, TRANSP_, BLACK__,
        TRANSP_, BLACK__, TRANSP_, BLACK__,
    ],
    // Border Test
    [
        WHITE__, WHITE__, WHITE__, WHITE__, WHITE__, WHITE__, WHITE__, WHITE__, WHITE__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, WHITE__, WHITE__, BLACK__, RED____, RED____,
        RED____, RED____, BLACK__, WHITE__, WHITE__, BLACK__, RED____, GREEN__, GREEN__, RED____,
        BLACK__, WHITE__, WHITE__, BLACK__, RED____, GREEN__, GREEN__, RED____, BLACK__, WHITE__,
        WHITE__, BLACK__, RED____, RED____, RED____, RED____, BLACK__, WHITE__, WHITE__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, WHITE__, WHITE__, WHITE__, WHITE__, WHITE__,
        WHITE__, WHITE__, WHITE__, WHITE__,
    ],
    // X Pattern
    [
        RED____, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, RED____, BLACK__, GREEN__,
        BLACK__, BLACK__, BLACK__, BLACK__, GREEN__, BLACK__, BLACK__, BLACK__, BLUE___, BLACK__,
        BLACK__, BLUE___, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, YELLOW_, YELLOW_, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, YELLOW_, YELLOW_, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLUE___, BLACK__, BLACK__, BLUE___, BLACK__, BLACK__, BLACK__, GREEN__,
        BLACK__, BLACK__, BLACK__, BLACK__, GREEN__, BLACK__, RED____, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, RED____,
    ],
    // GREEN Checker
    [
        BLACK__, GREEN__, BLACK__, GREEN__, BLACK__, GREEN__, BLACK__, BLACK__, GREEN__, BLACK__,
        GREEN__, BLACK__, GREEN__, BLACK__, GREEN__, BLACK__, BLACK__, GREEN__, BLACK__, GREEN__,
        BLACK__, GREEN__, BLACK__, BLACK__, GREEN__, BLACK__, GREEN__, BLACK__, GREEN__, BLACK__,
        GREEN__, BLACK__, BLACK__, GREEN__, BLACK__, GREEN__, BLACK__, GREEN__, BLACK__, BLACK__,
        GREEN__, BLACK__, GREEN__, BLACK__, GREEN__, BLACK__, GREEN__, BLACK__, BLACK__, GREEN__,
        BLACK__, GREEN__, BLACK__, GREEN__, BLACK__, BLACK__, GREEN__, BLACK__, GREEN__, BLACK__,
        GREEN__, BLACK__, GREEN__, BLACK__,
    ],
    // Unused
    [
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__,
    ],
    // Unused
    [
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__,
    ],
    // Unused
    [
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__, BLACK__,
        BLACK__, BLACK__, BLACK__, BLACK__,
    ],
];

/// Complete 32x32 test tilemap (1024 entries)
/// Tile indices:
/// 0x00-0x09 = Digits 0-8 + 9 = checkerboard
/// 0x0A = Checkerboard  0x0B = Gradient
/// 0x0C = Rainbow bars  0x0D = Transparency
/// 0x0E = Border test   0x0F = X Pattern
pub const TEST_TILEMAP: [u16; 1024] = [
    // Rows 0-3: Digits 0-9 in a 5x2 grid (top-left)
    0x00, 0x01, 0x02, 0x03, 0x04, 0x00, 0x01, 0x02, 0x03, 0x04, 0x00, 0x01, 0x02, 0x03, 0x04, 0x00,
    0x01, 0x02, 0x03, 0x04, 0x00, 0x01, 0x02, 0x03, 0x04, 0x00, 0x01, 0x02, 0x03, 0x04, 0x00, 0x01,
    // -
    0x05, 0x06, 0x07, 0x08, 0x09, 0x05, 0x06, 0x07, 0x08, 0x09, 0x05, 0x06, 0x07, 0x08, 0x09, 0x05,
    0x06, 0x07, 0x08, 0x09, 0x05, 0x06, 0x07, 0x08, 0x09, 0x05, 0x06, 0x07, 0x08, 0x09, 0x05, 0x06,
    // -
    0x00, 0x01, 0x02, 0x03, 0x04, 0x00, 0x01, 0x02, 0x03, 0x04, 0x00, 0x01, 0x02, 0x03, 0x04, 0x00,
    0x01, 0x02, 0x03, 0x04, 0x00, 0x01, 0x02, 0x03, 0x04, 0x00, 0x01, 0x02, 0x03, 0x04, 0x00, 0x01,
    // -
    0x05, 0x06, 0x07, 0x08, 0x09, 0x05, 0x06, 0x07, 0x08, 0x09, 0x05, 0x06, 0x07, 0x08, 0x09, 0x05,
    0x06, 0x07, 0x08, 0x09, 0x05, 0x06, 0x07, 0x08, 0x09, 0x05, 0x06, 0x07, 0x08, 0x09, 0x05, 0x06,
    // Rows 4-7: Test patterns in 4x4 blocks
    0x0A, 0x0A, 0x0B, 0x0B, 0x0A, 0x0A, 0x0B, 0x0B, 0x0A, 0x0A, 0x0B, 0x0B, 0x0A, 0x0A, 0x0B, 0x0B,
    0x0A, 0x0A, 0x0B, 0x0B, 0x0A, 0x0A, 0x0B, 0x0B, 0x0A, 0x0A, 0x0B, 0x0B, 0x0A, 0x0A, 0x0B, 0x0B,
    0x0A, 0x0A, 0x0B, 0x0B, 0x0A, 0x0A, 0x0B, 0x0B, 0x0A, 0x0A, 0x0B, 0x0B, 0x0A, 0x0A, 0x0B, 0x0B,
    0x0A, 0x0A, 0x0B, 0x0B, 0x0A, 0x0A, 0x0B, 0x0B, 0x0A, 0x0A, 0x0B, 0x0B, 0x0A, 0x0A, 0x0B, 0x0B,
    0x0C, 0x0C, 0x0D, 0x0D, 0x0C, 0x0C, 0x0D, 0x0D, 0x0C, 0x0C, 0x0D, 0x0D, 0x0C, 0x0C, 0x0D, 0x0D,
    0x0C, 0x0C, 0x0D, 0x0D, 0x0C, 0x0C, 0x0D, 0x0D, 0x0C, 0x0C, 0x0D, 0x0D, 0x0C, 0x0C, 0x0D, 0x0D,
    0x0C, 0x0C, 0x0D, 0x0D, 0x0C, 0x0C, 0x0D, 0x0D, 0x0C, 0x0C, 0x0D, 0x0D, 0x0C, 0x0C, 0x0D, 0x0D,
    0x0C, 0x0C, 0x0D, 0x0D, 0x0C, 0x0C, 0x0D, 0x0D, 0x0C, 0x0C, 0x0D, 0x0D, 0x0C, 0x0C, 0x0D, 0x0D,
    // Rows 8-15: Mixed diagonal patterns
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00,
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00,
    0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x01,
    0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x01,
    0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x01, 0x02,
    0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x01, 0x02,
    0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x01, 0x02, 0x03,
    0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x01, 0x02, 0x03,
    0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x01, 0x02, 0x03, 0x04,
    0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x01, 0x02, 0x03, 0x04,
    0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05,
    0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05,
    0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
    0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
    // Rows 16-31: Repeating test patterns
    // Vertical stripes (every 4 tiles)
    0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F,
    0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F,
    0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F,
    0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F,
    // ... (repeat last 4 rows until row 31)
    // Vertical stripes (every 4 tiles)
    0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F,
    0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F,
    0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F,
    0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F,
    // Vertical stripes (every 4 tiles)
    0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F,
    0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F,
    0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F,
    0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F,
    // Vertical stripes (every 4 tiles)
    0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D, 0x0A, 0x0B, 0x0C, 0x0D,
    0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F,
    0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F,
    0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F,
    0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F, 0x0E, 0x0F,
];
