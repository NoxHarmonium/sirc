use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, AddressingMode, InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    AddInstructionData, Instruction, RegisterInstructionData,
};

///
/// Parses the ADDR opcode
///
/// ```
/// use toolchain::parsers::opcodes::addr;
/// use toolchain::parsers::instruction::InstructionToken;
/// use peripheral_cpu::instructions::definitions::{ConditionFlags, Instruction, AddInstructionData, RegisterInstructionData};
///
/// let (_, parsed_instruction) = addr::addr("ADDR|!= y2, z1").unwrap();
/// let (r1, r2, condition_flag) = match parsed_instruction.instruction {
///     Instruction::Add(inner) => (inner.data.r1, inner.data.r2, inner.data.condition_flag),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// // TODO: Make a helper function or something to make these asserts smaller
/// assert_eq!(r1, 4);
/// assert_eq!(r2, 6);
/// assert_eq!(condition_flag, ConditionFlags::NotEqual);
/// ```
pub fn addr(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("ADDR"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [AddressingMode::DirectRegister(dest_register), AddressingMode::DirectRegister(src_register)] => {
                InstructionToken {
                    instruction: Instruction::Add(AddInstructionData {
                        data: RegisterInstructionData {
                            r1: dest_register.to_register_index(),
                            r2: src_register.to_register_index(),
                            r3: 0x00,
                            condition_flag,
                            additional_flags: 0x00,
                        },
                    }),
                    symbol_ref: None,
                }
            }
            _ => panic!(
                "ADDR opcode only supports direct register addressing mode (e.g. ADDR y1, z3)"
            ),
        },
    )(i)
}
