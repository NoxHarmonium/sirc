use peripheral_bus::BusPeripheral;
use peripheral_cpu::CpuPeripheral;

/// Gets a reference to the CPU from the Bus
///
/// Since the bus only knows about device interfaces, and not the concrete
/// `CpuPeripheral` type (which would introduce a circular dependency)
/// This is a roundabout way to cast it, since we know the bus master
/// is always the CPU at this point
pub fn cpu_from_bus(bus_peripheral: &mut BusPeripheral) -> &CpuPeripheral {
    // TODO: Investigate if there is an alternate way to casting to get a handle to the CPU
    // category=Refactoring
    // - Maybe the data that needs to be accessed can be part of the public interface
    // - Maybe we could use the bus itself to access special memory locations to fetch the data
    // - Maybe could use a debug CoP that can actually be implemented in HW
    bus_peripheral
        .bus_master
        .as_any()
        .downcast_ref::<CpuPeripheral>()
        .expect("failed to downcast")
}
