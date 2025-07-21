/// This crate isolates FFI functions from the rest of the workspace to keep everything tidy
use std::ffi::{c_char, CStr};
use std::slice;
use toolchain::printers::shared::print_tokens;
use toolchain::types::data::{DataToken, DataType};
use toolchain::types::shared::{LabelToken, NumberToken, NumberType, Token};

#[repr(C)]
pub struct CTilemap {
    pub packed_pixel_data: *const u16,
    pub packed_pixel_data_len: usize,
    pub palette: [u16; 16],
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
pub unsafe extern "C" fn tilemap_to_str(label_name: *const c_char, tilemap: CTilemap) -> *mut c_char {
    assert!(!label_name.is_null());
    let owned_label_name = unsafe { CStr::from_ptr(label_name).to_string_lossy().into_owned() };
    let packed_pixel_data = if tilemap.packed_pixel_data.is_null() {
        &[]
    } else {
        unsafe { slice::from_raw_parts(tilemap.packed_pixel_data, tilemap.packed_pixel_data_len) }
    };

    let pixel_tokens = slice_to_data_tokens(packed_pixel_data);
    let palette_tokens = slice_to_data_tokens(&tilemap.palette);
    let all_tokens = [vec![Token::Label(LabelToken { name: owned_label_name })], pixel_tokens, palette_tokens].concat();
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
