/// This crate isolates FFI functions from the rest of the workspace to keep everything tidy
use std::ffi::{CStr, c_char};
use std::slice;
use toolchain::printers::shared::print_tokens;
use toolchain::types::data::{DataToken, DataType};
use toolchain::types::shared::{LabelToken, NumberToken, NumberType, Token};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CTilemap {
    /// The label printed above this tilemap data so it can be referenced in assembly
    pub label: *const c_char,
    /// The comment printed above this tilemap data for humans to read
    pub comment: *const c_char,
    /// The tilemap entries that define how the tiles are laid out (see TilemapEntry type in device-video)
    ///     pub tile_index: B10, // 10 bits
    ///     pub palette_select: B3, // 3 bits
    ///     pub priority: B1, // 1 bit
    ///     pub flip_horizontal: B1, // 1 bit
    ///     pub flip_vertical: B1, // 1 bit
    pub data: *const u16,
    /// The length of the tilemap entries array
    pub data_len: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CPalette {
    /// The label printed above this palette so it can be referenced in assembly
    pub label: *const c_char,
    /// The comment printed above this palette for humans to read
    pub comment: *const c_char,
    /// The entries of the palette (15-bit bgr colour values)
    pub data: *const u16,
    /// The number of entries in the palette,
    pub data_len: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CTileSet {
    /// The label printed above this tileset so it can be referenced in assembly
    pub label: *const c_char,
    /// The comment printed above this tileset for humans to read
    pub comment: *const c_char,
    /// The tile data is laid out sequentially in either 8x8 or 16x16.
    /// Tile pixels are specified in 4-bit palette offsets. Therefore, each u16 value is four pixels.
    /// An 8x8 tile would be 16 x u16 values, and a 16x16 tile would be 64 x u16 values.
    pub data: *const u16,
    /// The length of the tileset data array.
    pub data_len: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CTilemapExport {
    /// The comment printed above all the tileset data, for humans to read
    pub tilesets_comment: *const c_char,
    /// Pointer to the first tileset.
    /// We can have any number of tilesets, but each layer can only have one tile base address that
    /// can only be changed in between frames.
    /// The tilemap entries are offset from the layers base address
    pub tilesets: *const CTileSet,
    /// The number of tilesets that have been passed in
    pub tilesets_len: usize,
    /// Pointer to the first tilemap
    /// We could have any number of tilemaps (whatever fits in the PPU ram), so we use a pointer
    /// and size combo, so it can be dynamic
    pub tilemaps_comment: *const c_char,
    pub tilemaps: *const CTilemap,
    /// The number of tilemaps that have been passed in
    pub tilemaps_len: usize,
    /// The comment printed above all the palette data, for humans to read
    pub palettes_comment: *const c_char,
    /// Pointer to the first palette.
    /// The palettes define the colours that the tile pixels reference.
    /// The palette is selected by a combination of PPU registers and tile map entry palette select values.
    pub palettes: *const CPalette,
    /// The number of palettes that have been passed in
    pub palettes_len: usize,
}

fn c_str_to_comment_token(x: *const c_char) -> Token {
    if x.is_null() {
        return Token::Comment(String::new());
    }
    Token::Comment(String::from_utf8_lossy(unsafe { CStr::from_ptr(x).to_bytes() }).into_owned())
}

fn u16_to_data_token(x: u16) -> Token {
    Token::Data(DataToken {
        size_bytes: 2,
        value: DataType::Value(NumberToken {
            value: u32::from(x),
            number_type: NumberType::Hex,
        }),
    })
}

fn slice_to_data_tokens(x: &[u16]) -> Vec<Token> {
    x.iter()
        .map(|x| u16_to_data_token(*x))
        .collect::<Vec<Token>>()
}

/// Takes tilemap data and returns a string that contains assembly code that will assemble to the
/// same tilemap.
///
/// Useful when you want to embed tilemap data in a program, which is probably the simplest way to
/// do it at the moment (although it might be possible to just link in a raw binary file).
///
/// # Safety
///
/// The arguments passed to this function should be properly initialised and point to valid memory
/// They will not be modified by this function.
/// The string returned by this function is owned by the rust code and must be freed by the free_str function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tilemap_to_str(tilemap_export: CTilemapExport) -> *mut c_char {
    assert!(!tilemap_export.tilesets_comment.is_null());
    assert!(!tilemap_export.tilesets.is_null());
    assert!(!tilemap_export.tilemaps_comment.is_null());
    assert!(!tilemap_export.tilemaps.is_null());
    assert!(!tilemap_export.palettes_comment.is_null());
    assert!(!tilemap_export.palettes.is_null());

    let tilesets =
        unsafe { slice::from_raw_parts(tilemap_export.tilesets, tilemap_export.tilesets_len) };

    // TODO: De-duplicate these functions
    let tileset_tokens = tilesets
        .iter()
        .filter(|tileset| {
            !tileset.label.is_null() && !tileset.comment.is_null() && !tileset.data.is_null()
        })
        .flat_map(|tileset| {
            let tileset_entries = if tileset.data.is_null() {
                &[]
            } else {
                unsafe { slice::from_raw_parts(tileset.data, tileset.data_len) }
            };
            let owned_tileset_label =
                unsafe { CStr::from_ptr(tileset.label).to_string_lossy().into_owned() };

            [
                vec![c_str_to_comment_token(tileset.comment)],
                vec![Token::Label(LabelToken {
                    name: owned_tileset_label,
                })],
                slice_to_data_tokens(tileset_entries),
            ]
            .concat()
        })
        .collect();

    let tilemaps =
        unsafe { slice::from_raw_parts(tilemap_export.tilemaps, tilemap_export.tilemaps_len) };

    let tilemap_tokens = tilemaps
        .iter()
        .filter(|tilemap| {
            !tilemap.label.is_null() && !tilemap.comment.is_null() && !tilemap.data.is_null()
        })
        .flat_map(|tilemap| {
            let tile_map_entries = if tilemap.data.is_null() {
                &[]
            } else {
                unsafe { slice::from_raw_parts(tilemap.data, tilemap.data_len) }
            };
            let owned_tilemap_label =
                unsafe { CStr::from_ptr(tilemap.label).to_string_lossy().into_owned() };

            [
                vec![c_str_to_comment_token(tilemap.comment)],
                vec![Token::Label(LabelToken {
                    name: owned_tilemap_label,
                })],
                slice_to_data_tokens(tile_map_entries),
            ]
            .concat()
        })
        .collect();

    let palettes = if tilemap_export.palettes.is_null() {
        &[]
    } else {
        unsafe { slice::from_raw_parts(tilemap_export.palettes, tilemap_export.palettes_len) }
    };

    let palette_tokens = palettes
        .iter()
        .filter(|palette| {
            !palette.label.is_null() && !palette.comment.is_null() && !palette.data.is_null()
        })
        .flat_map(|palette| {
            let palette_data = unsafe { slice::from_raw_parts(palette.data, palette.data_len) };
            let owned_palette_label =
                unsafe { CStr::from_ptr(palette.label).to_string_lossy().into_owned() };
            [
                vec![c_str_to_comment_token(palette.comment)],
                vec![Token::Label(LabelToken {
                    name: owned_palette_label,
                })],
                slice_to_data_tokens(palette_data),
            ]
            .concat()
        })
        .collect();

    let all_tokens = [
        vec![c_str_to_comment_token(tilemap_export.tilesets_comment)],
        tileset_tokens,
        vec![c_str_to_comment_token(tilemap_export.tilemaps_comment)],
        tilemap_tokens,
        vec![c_str_to_comment_token(tilemap_export.palettes_comment)],
        palette_tokens,
    ]
    .concat();
    let output_string = print_tokens(&all_tokens);

    let c_str = std::ffi::CString::new(output_string).unwrap();
    c_str.into_raw()
}

/// Frees a string returned by one of the rust functions in this file.
///
/// Should be called when the string is no longer needed to avoid memory leaks.
///
/// # Safety
///
/// The parameter passed to this function _must_ have been allocated by one of the rust functions
/// in this file.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_str(str: *mut c_char) {
    unsafe {
        let c_str = std::ffi::CString::from_raw(str);
        drop(c_str);
    }
}
