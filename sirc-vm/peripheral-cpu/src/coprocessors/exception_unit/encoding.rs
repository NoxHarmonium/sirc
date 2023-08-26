const OP_CODE_LENGTH: u16 = 8; // bits
const OP_CODE_MASK: u16 = 0x0F00;
const VALUE_MASK: u16 = 0x00FF;

pub struct ExceptionUnitInstruction {
    pub op_code: u8,
    pub value: u8,
}

pub fn decode_exception_unit_instruction(cause_register_value: u16) -> ExceptionUnitInstruction {
    let op_code = ((cause_register_value & OP_CODE_MASK) >> OP_CODE_LENGTH) as u8;
    let value = (cause_register_value & VALUE_MASK) as u8;
    ExceptionUnitInstruction { op_code, value }
}
