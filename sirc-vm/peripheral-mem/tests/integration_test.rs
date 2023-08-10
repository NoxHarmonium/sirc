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
    clippy::missing_const_for_fn
)]
#![deny(warnings)]

extern crate quickcheck;
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use std::{
    fs::File,
    io::{Read, Seek},
    path::Path,
};

use peripheral_mem::new_memory_peripheral;
use quickcheck::TestResult;
use tempfile::tempdir;

#[test]
fn regular_segment_test() {
    let segment_size: u32 = 0xF;

    let mut mem = new_memory_peripheral();
    mem.map_segment("some_segment", 0xCAFE_BEEF, segment_size, true);

    let out_of_bounds_address = 0xCAFE_FFFF;
    let in_bounds_address = 0xCAFE_BEF2;
    let boundary_address_low = 0xCAFE_BEEF;
    let boundary_address_high = 0xCAFE_BEEF + segment_size - 1;
    let boundary_address_one_over = 0xCAFE_BEEF + segment_size;

    let tests = vec![
        // Writing to address not mapped to segment will cause value to go into black hole
        // Reading from an out of bounds address should always be zero
        (out_of_bounds_address, 0xAFAF, 0x0),
        (in_bounds_address, 0xAFAF, 0xAFAF),
        (boundary_address_low, 0xFAFA, 0xFAFA),
        (boundary_address_high, 0xF00D, 0xF00D),
        (boundary_address_one_over, 0xAFAF, 0x0),
    ];

    for (address, value_to_write, expected_value_to_read) in tests {
        // Should read 0x0 if not written to
        assert_eq!(0x0, mem.read_address(address));
        mem.write_address(address, value_to_write);
        assert_eq!(expected_value_to_read, mem.read_address(address));
    }
}

#[test]
fn memory_mapped_segment_test() {
    let segment_size: u32 = 0xF;
    let base_address = 0xCAFE_BEEF;

    let dir = tempdir().unwrap();
    let file_to_memory_map = dir.path().join("mmap.bin");

    setup_memory_mapped_file(&file_to_memory_map, segment_size);

    let out_of_bounds_address = 0xCAFE_FFFF;
    let in_bounds_address = 0xCAFE_BEF2;
    let boundary_address_low = base_address;
    let boundary_address_high = base_address + segment_size - 1;
    let boundary_address_one_over = base_address + segment_size;

    let tests = vec![
        // Writing to address not mapped to segment will cause value to go into black hole
        // Reading from an out of bounds address should always be zero
        (out_of_bounds_address, 0xAFAF, 0x0),
        (in_bounds_address, 0xAFAF, 0xAFAF),
        (boundary_address_low, 0xFAFA, 0xFAFA),
        (boundary_address_high, 0xF00D, 0xF00D),
        (boundary_address_one_over, 0xAFAF, 0x0),
    ];

    write_data_to_file_segment(base_address, segment_size, &file_to_memory_map, &tests);

    let mut mem_mapped_file = File::open(file_to_memory_map).unwrap();

    for (address, _, expected_value_to_read) in &tests {
        // We don't want the out of bounds tests here
        if *expected_value_to_read == 0x0 {
            continue;
        }
        assert_file_has_expected_value(
            &mut mem_mapped_file,
            *address,
            base_address,
            *expected_value_to_read,
        );
    }
}

#[allow(clippy::cast_possible_truncation, clippy::needless_pass_by_value)]
#[quickcheck()]
fn dump_segment_to_file_test(test_buffer: Vec<u16>) -> TestResult {
    let base_address: u32 = 0xCAFE_BEEF;

    let mut mem = new_memory_peripheral();
    mem.map_segment("some_segment", base_address, test_buffer.len() as u32, true);

    for i in 0..test_buffer.len() {
        mem.write_address(base_address + i as u32, *test_buffer.get(i).unwrap());
    }

    let dump = mem.dump_segment("some_segment");

    let actual: Vec<u16> = dump
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes(chunk.try_into().unwrap()))
        .collect();
    TestResult::from_bool(test_buffer == actual)
}

fn write_data_to_file_segment(
    base_address: u32,
    segment_size: u32,
    file_to_memory_map: &Path,
    tests: &Vec<(u32, u16, u16)>,
) {
    let mut mem = new_memory_peripheral();
    mem.map_segment_to_file(
        "some_segment",
        base_address,
        segment_size,
        true,
        file_to_memory_map,
    );

    for (address, value_to_write, expected_value_to_read) in tests {
        // Should read 0x0 if not written to
        assert_eq!(0x0, mem.read_address(*address));
        mem.write_address(*address, *value_to_write);
        assert_eq!(*expected_value_to_read, mem.read_address(*address));
    }

    // Make sure that the file is not mounted by the MemoryPeripheral anymore
    drop(mem);
}

fn assert_file_has_expected_value(
    mem_mapped_file: &mut File,
    address: u32,
    base_address: u32,
    expected_value_to_read: u16,
) {
    mem_mapped_file
        .seek(std::io::SeekFrom::Start(
            ((address - base_address) * 2).into(),
        ))
        .unwrap();
    let mut bytes = [0; 2];
    mem_mapped_file.read_exact(&mut bytes).unwrap();
    assert_eq!(expected_value_to_read, u16::from_be_bytes(bytes));
}

#[allow(clippy::cast_lossless)]
fn setup_memory_mapped_file(file_to_memory_map: &Path, segment_size: u32) {
    let new_file = File::create(file_to_memory_map).unwrap();
    // Multiply by two because memory is accessed as words
    new_file.set_len((segment_size * 2) as u64).unwrap();
    drop(new_file);
}
