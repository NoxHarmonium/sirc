pub use super::super::optional_machine::Machine;
use peripheral_bus::device::{BusAccessType, BusAssertions};
use peripheral_cpu::{
    coprocessors::{dma::DmaCoprocessor, processing_unit::definitions::ConditionFlags},
    CpuConfig,
};

// Raw ISA command encodings: default pair is a; magnitude is ORed in by tests.
// Keep these independent of the implementation to catch encoding regressions.
pub const DMAR: u16 = 0x2800;
pub const DMAW: u16 = 0x2900;
pub const DMAT: u16 = 0x2a00;
pub const BACKWARD: u16 = 0x0020;
pub const ADDRESS_PAIR_SHIFT: u16 = 6;
pub const RESERVED_PAIR: u16 = 0x00c0;
pub const RESERVED_DMA_OPCODE: u16 = 0x2b00;
pub const FAULT_LINK: usize = 6;
pub const FAULT_METADATA_LINK: usize = 7;
pub const LEVEL_FIVE_PIN: u8 = 0b1_0000;
pub const RETE: u16 = 0x1a00;
pub const TRACE_VECTOR: u32 = 0x0c;
pub const L5_CONFLICT_VECTOR: u32 = 0x0e;
pub const L5_VECTOR: u32 = 0x20;
pub const L1_VECTOR: u32 = 0xa0;

impl Machine {
    pub fn new(command: u16, register_call: bool) -> Self {
        Self::conditional(command, register_call, ConditionFlags::Always)
    }
    pub fn conditional(command: u16, register_call: bool, condition_flag: ConditionFlags) -> Self {
        Self::with_config(
            command,
            register_call,
            condition_flag,
            CpuConfig {
                optional_coprocessors: vec![Box::new(DmaCoprocessor::default())],
            },
        )
    }
    pub fn dma_accesses(&self) -> Vec<BusAssertions> {
        self.accesses
            .iter()
            .copied()
            .filter(|r| {
                matches!(
                    r.bus_access_type,
                    BusAccessType::DmaReadBurst | BusAccessType::DmaWriteBurst
                )
            })
            .collect()
    }
}
