/// This crate isolates FFI functions from the rest of the workspace to keep everything tidy
use std::ffi::{CStr, c_char};
use std::slice;
use toolchain::printers::shared::print_tokens;
use toolchain::types::data::{DataToken, DataType};
use toolchain::types::shared::{LabelToken, NumberToken, NumberType, Token};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CTilemap {
    /// The label printed above this tilemap data for humans to read
    pub label: *const c_char,
    /// The comment printed above this tilemap data, so it can be referenced in assembly routines
    pub comment: *const c_char,
    /// Which palette this tilemap uses
    pub palette_index: u16, // Only for reference, doesn't affect the tilemap
    /// The tilemap pixel data packed so there are four 4bpp pixels per word
    pub packed_pixel_data: *const u16,
    /// The length of the packed pixel data in words
    pub packed_pixel_data_len: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CPalette {
    /// The comment printed above this palette for humans to read
    pub comment: *const c_char,
    /// The 16 entries of the palette.
    pub data: [u16; 16],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CTilemapExport {
    /// Pointer to the first tilemap
    /// We could have any number of tilemaps (whatever fits in the PPU ram), so we use a pointer
    /// and size combo, so it can be dynamic
    pub tilemaps: *const CTilemap,
    /// The number of tilemaps that have been passed in
    pub tilemaps_len: usize,
    /// The label printed above the palette data, so it can be referenced in assembly routines
    pub palette_label: *const c_char,
    /// The palettes that the tilemaps refer to.
    /// The palette size in the PPU is 16x16 colour palettes (only 16 colours can be addressable at once with 4bpp)
    /// We may as well represent this as a fixed size array to keep it simple
    pub palettes: [CPalette; 16],
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
    assert!(!tilemap_export.tilemaps.is_null());
    assert!(!tilemap_export.palette_label.is_null());
    let tilemaps =
        unsafe { slice::from_raw_parts(tilemap_export.tilemaps, tilemap_export.tilemaps_len) };
    let tilemap_tokens = tilemaps
        .iter()
        .flat_map(|tilemap| {
            let packed_pixel_data = if tilemap.packed_pixel_data.is_null() {
                &[]
            } else {
                unsafe {
                    slice::from_raw_parts(tilemap.packed_pixel_data, tilemap.packed_pixel_data_len)
                }
            };
            let owned_tilemap_label =
                unsafe { CStr::from_ptr(tilemap.label).to_string_lossy().into_owned() };

            [
                vec![c_str_to_comment_token(tilemap.comment)],
                vec![Token::Label(LabelToken {
                    name: owned_tilemap_label,
                })],
                slice_to_data_tokens(packed_pixel_data),
            ]
            .concat()
        })
        .collect();

    let palette_tokens = tilemap_export
        .palettes
        .iter()
        .filter(|palette| !palette.comment.is_null())
        .flat_map(|palette| {
            [
                vec![c_str_to_comment_token(palette.comment)],
                slice_to_data_tokens(palette.data.as_slice()),
            ]
            .concat()
        })
        .collect();

    let owned_palette_label = unsafe {
        CStr::from_ptr(tilemap_export.palette_label)
            .to_string_lossy()
            .into_owned()
    };
    let all_tokens = [
        tilemap_tokens,
        vec![Token::Label(LabelToken {
            name: owned_palette_label,
        })],
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
