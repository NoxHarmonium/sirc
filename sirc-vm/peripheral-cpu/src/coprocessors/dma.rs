//! Supervisor DMA hardware. Architectural side effects commit only on successful ACKs.
use crate::{
    coprocessors::{
        exception_unit::definitions::Faults,
        optional::{CoprocessorFault, OptionalCoprocessor},
    },
    registers::{sr_bit_is_set, Registers, SegmentedAddress, StatusRegisterFields},
};
use peripheral_bus::device::{BusAccessType, BusAssertions, BusOperation};

#[derive(Clone, Copy)]
enum Pair {
    Address,
    Link,
    Stack,
}
impl Pair {
    fn read(self, r: &Registers) -> (u16, u16) {
        match self {
            Self::Address => (r.ah, r.al),
            Self::Link => (r.lh, r.ll),
            Self::Stack => (r.sh, r.sl),
        }
    }
    fn set_low(self, r: &mut Registers, low: u16) {
        match self {
            Self::Address => r.al = low,
            Self::Link => r.ll = low,
            Self::Stack => r.sl = low,
        }
    }
}
struct Burst {
    pair: Pair,
    write: bool,
    backward: bool,
    base: (u16, u16),
    count: u8,
    index: u8,
    trap: bool,
}
impl Burst {
    fn bat(&self) -> BusAccessType {
        if self.write {
            BusAccessType::DmaWriteBurst
        } else {
            BusAccessType::DmaReadBurst
        }
    }
    fn overflow(&self, low: u16) -> CoprocessorFault {
        CoprocessorFault {
            fault: Faults::SegmentOverflow,
            address: (self.base.0, low).to_full_address(),
            bus_access_type: self.bat(),
        }
    }
}

/// Optional coprocessor 2; no unit instance exists in a no-DMA configuration.
#[derive(Default)]
pub struct DmaCoprocessor {
    burst: Option<Burst>,
}
impl OptionalCoprocessor for DmaCoprocessor {
    fn id(&self) -> u8 {
        2
    }
    fn start(&mut self, command: u16, registers: &mut Registers) -> Result<(), CoprocessorFault> {
        self.burst = None;
        let operation = (command >> 8) & 0xf;
        if !matches!(operation, 8 | 9) {
            return Err(CoprocessorFault::invalid_command(command));
        }
        let count = u8::try_from(command & 7).unwrap();
        if count == 0 {
            return Ok(());
        }
        let pair = match (command >> 6) & 3 {
            0 => Pair::Address,
            1 => Pair::Link,
            2 => Pair::Stack,
            // Reserved operands are undefined architecturally; reject safely.
            _ => return Err(CoprocessorFault::invalid_command(command)),
        };
        let mut burst = Burst {
            pair,
            write: operation == 9,
            backward: command & 0x20 != 0,
            base: pair.read(registers),
            count,
            index: 0,
            trap: sr_bit_is_set(StatusRegisterFields::TrapOnAddressOverflow, registers),
        };
        if burst.backward {
            let (low, wrapped) = burst.base.1.overflowing_sub(u16::from(count));
            if wrapped && burst.trap {
                return Err(burst.overflow(low));
            }
            burst.base.1 = low;
            pair.set_low(registers, low);
        }
        self.burst = Some(burst);
        Ok(())
    }
    fn next_request(&mut self, registers: &Registers) -> Result<BusAssertions, CoprocessorFault> {
        let Some(burst) = &self.burst else {
            return Ok(BusAssertions::default());
        };
        if burst.index == burst.count {
            return Ok(BusAssertions::default());
        }
        let (low, wrapped) = burst.base.1.overflowing_add(u16::from(burst.index));
        if wrapped && burst.trap {
            return Err(burst.overflow(low));
        }
        Ok(BusAssertions {
            address: (burst.base.0, low).to_full_address(),
            data: if burst.write {
                registers[burst.index]
            } else {
                0
            },
            op: if burst.write {
                BusOperation::Write
            } else {
                BusOperation::Read
            },
            bus_access_strobe: true,
            bus_access_type: burst.bat(),
            ..BusAssertions::default()
        })
    }
    fn complete_access(
        &mut self,
        _: BusAssertions,
        response: BusAssertions,
        registers: &mut Registers,
    ) -> Result<(), CoprocessorFault> {
        let burst = self
            .burst
            .as_mut()
            .expect("DMA ACK must belong to an active burst");
        if !burst.write {
            registers[burst.index] = response.data;
        }
        burst.index += 1;
        if burst.index == burst.count && !burst.backward {
            let (low, wrapped) = burst.base.1.overflowing_add(u16::from(burst.count));
            if wrapped && burst.trap {
                return Err(burst.overflow(low));
            }
            burst.pair.set_low(registers, low);
        }
        Ok(())
    }
    fn is_finished(&self) -> bool {
        self.burst
            .as_ref()
            .is_none_or(|burst| burst.index == burst.count)
    }
    fn reset(&mut self) {
        self.burst = None;
    }
}
