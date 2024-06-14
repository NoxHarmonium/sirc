use std::fs::{read, write};
use std::io;
use std::path::PathBuf;

use crate::utils::path::add_extension;

use super::types::ProgramDebugInfo;

pub fn write_debug_map(debug_info_map: &ProgramDebugInfo, path: PathBuf) -> Result<(), io::Error> {
    let mut debug_map_path = path;
    add_extension(&mut debug_map_path, "dbg");
    let bytes_to_write = match postcard::to_allocvec(&debug_info_map) {
        Ok(bytes_to_write) => bytes_to_write,
        Err(error) => panic!("Error encoding debug file: {error}"),
    };
    write(debug_map_path, bytes_to_write)?;
    Ok(())
}

pub fn read_debug_map(path: PathBuf) -> Result<ProgramDebugInfo, io::Error> {
    let mut debug_map_path = path;
    add_extension(&mut debug_map_path, "dbg");
    let bytes = read(debug_map_path)?;
    let debug_map = match postcard::from_bytes(&bytes) {
        Ok(bytes_to_write) => bytes_to_write,
        Err(error) => panic!("Error decoding debug file: {error}"),
    };
    Ok(debug_map)
}
