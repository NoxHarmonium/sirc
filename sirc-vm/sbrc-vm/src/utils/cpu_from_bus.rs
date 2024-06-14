use peripheral_bus::BusPeripheral;
use peripheral_cpu::CpuPeripheral;

/// Since the bus only knows about device interfaces, and not the concrete
/// `CpuPeripheral` type (which would introduce a circular dependency)
/// This is a roundabout way to cast it, since we know the bus master
/// is always the CPU at this point
pub fn cpu_from_bus(bus_peripheral: &mut BusPeripheral) -> &CpuPeripheral {
    // TODO: This is weird. Is there a better way?
    bus_peripheral
        .bus_master
        .as_any()
        .downcast_ref::<CpuPeripheral>()
        .expect("failed to downcast")
}
