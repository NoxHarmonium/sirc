use peripheral_mem::MemoryPeripheral;

pub fn fetch_instruction(mem: &MemoryPeripheral, pc: u32) -> [u8; 4] {
    // TODO: Alignment check?
    // TODO: Do we need to copy here?
    let [b1, b2] = u16::to_be_bytes(mem.read_address(pc).to_owned());
    let [b3, b4] = u16::to_be_bytes(mem.read_address(pc + 1).to_owned());
    [b2, b1, b4, b3]
}
