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

use insta::assert_snapshot;
use libsirc::capi::{CPalette, CTilemap, CTilemapExport, free_str, tilemap_to_str};
use std::ffi::CString;

/// Just exists to make sure nothing goes out of scope and invaldates pointers
#[repr(C)]
struct TestHarness {
    palette_comments: [CString; 16],
    tilemap_comment: CString,
    tilemap_label_name: CString,
    palette_label_name: CString,
    palettes: [CPalette; 16],
    tilemap: CTilemap,
    tilemaps: Vec<CTilemap>,
    export: CTilemapExport,
}

fn create_test_export() -> TestHarness {
    let pixels = &[1u16, 2u16, 3u16, 4u16];

    let tilemap_comment = CString::new("tilemap 1 comment").unwrap();
    let palette_label_name = CString::new("palettes").unwrap();
    let tilemap_label_name = CString::new("tilemaps").unwrap();

    let palette_comments: [CString; 16] = (0..16)
        .map(|i| CString::new(format!("palette {i} comment")).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let palettes: [CPalette; 16] = (0..16)
        .map(|i: u16| CPalette {
            comment: palette_comments[i as usize].as_ptr(),
            data: [
                i,
                1 + i,
                2 + i,
                3 + i,
                4 + i,
                5 + i,
                6 + i,
                7 + i,
                8 + i,
                9 + i,
                10 + i,
                11 + i,
                12 + i,
                13 + i,
                14 + i,
                15 + i,
            ],
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let tilemap: CTilemap = CTilemap {
        label: tilemap_label_name.as_ptr(),
        comment: tilemap_comment.as_ptr(),
        packed_pixel_data: pixels.as_ptr(),
        packed_pixel_data_len: pixels.len(),
        palette_index: 0,
    };

    let tilemaps = vec![tilemap];

    let export = CTilemapExport {
        tilemaps: tilemaps.as_ptr(),
        tilemaps_len: 1,
        palette_label: palette_label_name.as_ptr(),
        palettes,
    };

    TestHarness {
        palette_comments,
        tilemap_comment,
        tilemap_label_name,
        palette_label_name,
        palettes,
        tilemap,
        tilemaps,
        export,
    }
}

#[test]
fn test_tilemap_to_str() {
    // Setup
    // Note: This is designed to be called from C/C++ via FFI, this setup code is just simulating that
    let test_harness = create_test_export();
    let export = test_harness.export;

    // Run
    let str = unsafe { tilemap_to_str(export) };

    // Assert
    let c_str = unsafe { CString::from_raw(str) };
    let actual_str = c_str.to_str().unwrap();

    assert_snapshot!(actual_str);

    // No need to free the string in a rust test because it will be freed when c_str goes out of bounds
    // (the from_raw retakes ownership in rust's memory management)
}

#[test]
fn test_free_str() {
    // Setup
    // Note: This is designed to be called from C/C++ via FFI, this setup code is just simulating that\
    let test_harness = create_test_export();
    let export = test_harness.export;

    // Run
    let raw_str = unsafe { tilemap_to_str(export) };
    // We don't use CString::from_raw here because we don't want rust to take ownership of the string
    // again because we specifically want to test the free function.
    unsafe { free_str(raw_str) }

    // Not sure how to assert here. It will panic on double frees or anything like that though
}
