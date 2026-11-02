use super::numeric::{Constants, NumericField, NumericOperand, NumericPlaceholder};
use crate::data::object::AssemblyError;
use crate::types::data::RefToken;
use peripheral_cpu::coprocessors::processing_unit::definitions::InstructionData;

#[derive(Debug, Clone)]
pub struct InstructionToken {
    /// The length of the parser input at the time of parsing, used to work out where the parser is in the file
    pub input_length: usize,
    pub instruction: InstructionData,
    pub symbol_ref: Option<RefToken>,
    pub numeric_placeholders: Vec<NumericPlaceholder>,
}

impl InstructionToken {
    pub fn resolve_numeric_operands(
        &self,
        constants: &Constants,
        source_length: usize,
    ) -> Result<InstructionData, AssemblyError> {
        let mut instruction = self.instruction.clone();
        for placeholder in &self.numeric_placeholders {
            let value = NumericOperand::Constant(placeholder.reference.clone())
                .resolve(constants)
                .and_then(|number| placeholder.field.encode(&number))
                .map_err(|message| AssemblyError {
                    input_offset: source_length - placeholder.reference.input_length,
                    message,
                })?;
            match (&mut instruction, placeholder.field) {
                (InstructionData::Immediate(data), NumericField::Immediate16) => data.value = value,
                (InstructionData::ShortImmediate(data), NumericField::Immediate8) => {
                    data.value = u8::try_from(value).expect("validated short immediate");
                }
                (InstructionData::ShortImmediate(data), NumericField::ShiftCount) => {
                    data.shift_count = u8::try_from(value).expect("validated shift count");
                }
                (InstructionData::Register(data), NumericField::ShiftCount) => {
                    data.shift_count = u8::try_from(value).expect("validated shift count");
                }
                (InstructionData::Immediate(data), field) => {
                    let mask = match field {
                        NumericField::ExceptionVector | NumericField::DmaTransferCount => 0xff,
                        NumericField::ExceptionLinkIndex => 0xf,
                        NumericField::DmaRegisterCount => 0x27,
                        _ => unreachable!("numeric field does not belong to immediate format"),
                    };
                    data.value = (data.value & !mask) | value;
                }
                _ => unreachable!("numeric placeholder does not match instruction format"),
            }
        }
        Ok(instruction)
    }
}
