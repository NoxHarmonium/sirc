#[derive(Debug, Clone, Default, PartialEq, Eq, Copy)]
pub struct ExceptionUnitRegisters {
    pub cause_register: u16,
    pub exception_level: u8,
    pub link_registers: [u32; 7],
}
