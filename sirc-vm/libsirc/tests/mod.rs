#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    // I don't like this rule
    clippy::module_name_repetitions,
    // Not sure what this is, will have to revisit
    clippy::must_use_candidate,
    // Will tackle this at the next clean up
    clippy::too_many_lines,
    // Might be good practice but too much work for now
    clippy::missing_errors_doc,
    // Not stable yet - try again later
    clippy::missing_const_for_fn,
    // I have a lot of temporary panics for debugging that will probably be cleaned up
    clippy::missing_panics_doc
)]
#![deny(warnings)]

use libsirc::{CTilemap, free_str, tilemap_to_str};

#[test]
fn test_tilemap_to_str() {
    // Setup
    // Note: This is designed to be called from C/C++ via FFI, this setup code is just simulating that
    let pixels = &[1u16, 2u16, 3u16, 4u16];
    let palette = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

    let tilemap: CTilemap = CTilemap {
        packed_pixel_data: pixels.as_ptr(),
        packed_pixel_data_len: pixels.len(),
        palette: *palette,
    };

    // Run
    let str = tilemap_to_str(tilemap);
    let c_str = unsafe { std::ffi::CString::from_raw(str) };

    // Assert
    let actual_str = c_str.to_str().unwrap();
    let expected_str = "\
        .DW #0x0001\n\
        .DW #0x0002\n\
        .DW #0x0003\n\
        .DW #0x0004\n\
        .DW #0x0000\n\
        .DW #0x0001\n\
        .DW #0x0002\n\
        .DW #0x0003\n\
        .DW #0x0004\n\
        .DW #0x0005\n\
        .DW #0x0006\n\
        .DW #0x0007\n\
        .DW #0x0008\n\
        .DW #0x0009\n\
        .DW #0x000A\n\
        .DW #0x000B\n\
        .DW #0x000C\n\
        .DW #0x000D\n\
        .DW #0x000E\n\
        .DW #0x000F\n\
    ";
    assert_eq!(expected_str, actual_str);

    // No need to free the string in a rust test because it will be freed when c_str goes out of bounds
}

#[test]
fn test_free_str() {
    // Setup
    // Note: This is designed to be called from C/C++ via FFI, this setup code is just simulating that
    let pixels = &[1u16, 2u16, 3u16, 4u16];
    let palette = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

    let tilemap: CTilemap = CTilemap {
        packed_pixel_data: pixels.as_ptr(),
        packed_pixel_data_len: pixels.len() * 2,
        palette: *palette,
    };

    // Run
    let raw_str = tilemap_to_str(tilemap);
    unsafe { free_str(raw_str) }

    // Not sure how to assert here. It will panic on double frees or anything like that though
}
