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
use libsirc::capi::{CPalette, CTileSet, CTilemap, CTilemapExport, free_str, tilemap_to_str};
use std::ffi::CString;

/// Just exists to make sure nothing goes out of scope and invaldates pointers
#[repr(C)]
struct TestHarness {
    tilesets_comment: CString,
    tileset_comments: [CString; 16],
    tileset_labels: [CString; 16],
    tilesets: Vec<CTileSet>,
    tileset_data: Vec<[u16; 16]>,

    tilemaps_comment: CString,
    tilemap_comments: [CString; 16],
    tilemap_labels: [CString; 16],
    tilemaps: Vec<CTilemap>,
    tilemap_data: Vec<[u16; 8]>,

    palettes_comment: CString,
    palette_comments: [CString; 16],
    palette_labels: [CString; 16],
    palettes: Vec<CPalette>,
    palette_data: Vec<[u16; 16]>,

    export: CTilemapExport,
}

fn create_test_export() -> Box<TestHarness> {
    // 8x8 tile
    let tile_pixels = &[
        1u16, 2u16, 3u16, 4u16, 1u16, 2u16, 3u16, 4u16, 1u16, 2u16, 3u16, 4u16, 1u16, 2u16, 3u16,
        4u16,
    ];
    let tilemap_entries = &[
        0x0u16, 0x1u16, 0x2u16, 0x3u16, 0x4u16, 0x5u16, 0x6u16, 0x7u16,
    ];

    let tilesets_comment = CString::new("Tilesets Section").unwrap();
    let tileset_comments: [CString; 16] = (0..16)
        .map(|i| CString::new(format!("tileset {i} comment")).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let tileset_labels: [CString; 16] = (0..16)
        .map(|i| CString::new(format!("tileset_{i}")).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let tileset_data = (0..16).map(|_| *tile_pixels).collect::<Vec<[u16; 16]>>();

    let tilemaps_comment = CString::new("Tilemaps Section").unwrap();
    let tilemap_comments: [CString; 16] = (0..16)
        .map(|i| CString::new(format!("tilemap {i} comment")).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let tilemap_labels: [CString; 16] = (0..16)
        .map(|i| CString::new(format!("tilemap_{i}")).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let tilemap_data = (0..16).map(|_| *tilemap_entries).collect::<Vec<[u16; 8]>>();

    let palettes_comment = CString::new("Palettes Section").unwrap();
    let palette_comments: [CString; 16] = (0..16)
        .map(|i| CString::new(format!("palette {i} comment")).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let palette_labels: [CString; 16] = (0..16)
        .map(|i| CString::new(format!("palette_{i}")).unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let palette_data = (0..16)
        .map(|i: u16| {
            [
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
            ]
        })
        .collect::<Vec<_>>();

    let mut harness = Box::new(TestHarness {
        tilesets_comment,
        tileset_comments,
        tileset_labels,
        tilesets: Vec::new(),
        tileset_data,

        tilemaps_comment,
        tilemap_comments,
        tilemap_labels,
        tilemaps: Vec::new(),
        tilemap_data,

        palettes_comment,
        palette_comments,
        palette_labels,
        palettes: Vec::new(),
        palette_data,

        export: CTilemapExport {
            tilesets_comment: std::ptr::null(),
            tilesets: std::ptr::null(),
            tilesets_len: 0,
            tilemaps_comment: std::ptr::null(),
            tilemaps: std::ptr::null(),
            tilemaps_len: 0,
            palettes_comment: std::ptr::null(),
            palettes: std::ptr::null(),
            palettes_len: 0,
        },
    });

    for (i, tile) in harness.tileset_data.iter().enumerate() {
        harness.tilesets.push(CTileSet {
            label: harness.tileset_labels[i].as_ptr(),
            comment: harness.tileset_comments[i].as_ptr(),
            data: tile.as_ptr(),
            data_len: tile.len(),
        });
    }

    for (i, tilemap_datum) in harness.tilemap_data.iter().enumerate() {
        harness.tilemaps.push(CTilemap {
            label: harness.tilemap_labels[i].as_ptr(),
            comment: harness.tilemap_comments[i].as_ptr(),
            data: tilemap_datum.as_ptr(),
            data_len: tilemap_datum.len(),
        });
    }

    for (i, palette_datum) in harness.palette_data.iter().enumerate() {
        harness.palettes.push(CPalette {
            label: harness.palette_labels[i].as_ptr(),
            comment: harness.palette_comments[i].as_ptr(),
            data: palette_datum.as_ptr(),
            data_len: palette_datum.len(),
        });
    }

    // TODO: What I am doing?
    // I'll also have to update sirc-tiledit so it slices the pixels up into 8x8 or 16x16 tiles and deduplicates them
    // I'm guessing that there won't be many tiles that are exact duplicates, but it will reduce the amount of storage
    // for fully transparent areas by a lot.
    // Then it can export a proper data structure in asm for basic_video asm example and maybe render something
    // TODO: Do we need to duplicate all these types with the device-video types? Can we use them somehow?
    harness.export = CTilemapExport {
        tilesets_comment: harness.tilesets_comment.as_ptr(),
        tilesets: harness.tilesets.as_ptr(),
        tilesets_len: harness.tilesets.len(),
        tilemaps_comment: harness.tilemaps_comment.as_ptr(),
        tilemaps: harness.tilemaps.as_ptr(),
        tilemaps_len: harness.tilemaps.len(),
        palettes_comment: harness.palettes_comment.as_ptr(),
        palettes: harness.palettes.as_ptr(),
        palettes_len: harness.palettes.len(),
    };

    harness
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
