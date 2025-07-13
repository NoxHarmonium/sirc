use crate::types::data::RefToken;
use peripheral_cpu::coprocessors::processing_unit::definitions::InstructionData;

#[derive(Debug, Clone)]
pub struct InstructionToken {
    /// The length of the parser input at the time of parsing, used to work out where the parser is in the file
    pub input_length: usize,
    pub instruction: InstructionData,
    pub symbol_ref: Option<RefToken>,
    pub placeholder_name: Option<String>,
}
