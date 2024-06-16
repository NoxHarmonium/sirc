use std::collections::BTreeMap;

use line_col::LineColLookup;

use crate::debug_adapter::types::{ProgramDebugInfo, ProgramPosition};

fn flip_map(
    program_to_input_offset_mapping: &BTreeMap<ProgramPosition, usize>,
) -> BTreeMap<usize, ProgramPosition> {
    program_to_input_offset_mapping
        .iter()
        .map(|(k, v)| (*v, *k))
        .collect::<BTreeMap<_, _>>()
}

#[must_use]
pub fn translate_pc_to_line_column(
    program_debug_info: &ProgramDebugInfo,
    pc: u32,
) -> Option<(i64, i64, String)> {
    // Find the matching file
    let debug_info = program_debug_info
        .debug_info_map
        .range(..=pc)
        .next_back()
        .map(|(_, f)| f)?;

    // Then the matching line
    // TODO: Improve the performance of the PC <-> Line/Column lookups in the debugger
    // category=Debugging
    // - Should probably cache the lookup more
    // - See also the function that translates the other direction
    let lookup = LineColLookup::new(debug_info.original_input.as_str());
    debug_info
        .program_to_input_offset_mapping
        .get(&pc)
        .map(|pos| lookup.get(*pos))
        .map(|(line, col)| {
            (
                line.try_into().unwrap(),
                col.try_into().unwrap(),
                debug_info.original_filename.clone(),
            )
        })
}

#[must_use]
pub fn translate_line_column_to_pc(
    program_debug_info: &ProgramDebugInfo,
    original_filename: &str,
    (line, column): (i64, i64),
) -> Option<u32> {
    // Find the matching file
    let debug_info = program_debug_info
        .debug_info_map
        .values()
        .find(|d| d.original_filename == original_filename)?;
    // Then the matching line
    let input_offset_to_program_mapping = flip_map(&debug_info.program_to_input_offset_mapping);
    let mut lines_to_find = line - 1;
    let mut offset: usize = 0;
    for char in debug_info.original_input.bytes() {
        if lines_to_find == 0 {
            break;
        }
        offset += 1;
        // TODO: Is this right?
        if char == b'\n' {
            lines_to_find -= 1;
        }
    }

    let column_value: usize = column.try_into().unwrap();
    offset += column_value - 1;

    if offset >= debug_info.original_input.len() {
        return None;
    }

    input_offset_to_program_mapping
        .range(..=offset)
        .next_back()
        .map(|(_, pc)| *pc)
}
