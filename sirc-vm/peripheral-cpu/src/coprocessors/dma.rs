//! Supervisor DMA hardware. Architectural side effects commit only on successful ACKs.
use crate::{
    coprocessors::{
        exception_unit::definitions::Faults,
        optional::{CoprocessorFault, OptionalCoprocessor},
    },
    registers::{sr_bit_is_set, Registers, SegmentedAddress, StatusRegisterFields},
    COPROCESSOR_OPCODE_ID_MASK,
};
use peripheral_bus::device::{BusAccessType, BusAssertions, BusOperation};

// Opcode fields are kept in their command-word positions for direct masking.
const DMAR_OPCODE: u16 = 0x0800;
const DMAW_OPCODE: u16 = 0x0900;
const DMAT_OPCODE: u16 = 0x0a00;
const REGISTER_BURST_COUNT_MASK: u16 = 0x0007;
const COPY_COUNT_MASK: u16 = 0x00ff;
const BACKWARD_DIRECTION_BIT: u16 = 0x0020;
const ADDRESS_PAIR_MASK: u16 = 0x00c0;
const ADDRESS_PAIR_SHIFT: u16 = 6;
// DMAT buffers each chunk in the architectural r1..r7 register window.
const TRANSFER_WINDOW_WORDS: u8 = 7;

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
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Forward,
    Backward,
    Copy,
}

struct Burst {
    pair: Pair,
    write: bool,
    mode: Mode,
    base: (u16, u16),
    destination: (u16, u16),
    remaining: u8,
    count: u8,
    index: u8,
    trap: bool,
}
fn overflow(base: (u16, u16), low: u16, bat: BusAccessType) -> CoprocessorFault {
    CoprocessorFault {
        fault: Faults::SegmentOverflow,
        address: (base.0, low).to_full_address(),
        bus_access_type: bat,
    }
}
impl Burst {
    fn bat(&self) -> BusAccessType {
        if self.write {
            BusAccessType::DmaWriteBurst
        } else {
            BusAccessType::DmaReadBurst
        }
    }
    fn access_base(&self) -> (u16, u16) {
        if self.mode == Mode::Copy && self.write {
            self.destination
        } else {
            self.base
        }
    }
    fn finish_chunk(&mut self, registers: &mut Registers) -> Result<(), CoprocessorFault> {
        if self.mode == Mode::Copy {
            let (source, source_wrap) = self.base.1.overflowing_add(u16::from(self.count));
            let (destination, destination_wrap) =
                self.destination.1.overflowing_add(u16::from(self.count));
            // Validate both before committing either. Completed words/window
            // writes survive a fault; a/l still describe the unadvanced chunk.
            if self.trap && source_wrap {
                return Err(overflow(self.base, source, BusAccessType::DmaReadBurst));
            }
            if self.trap && destination_wrap {
                return Err(overflow(
                    self.destination,
                    destination,
                    BusAccessType::DmaWriteBurst,
                ));
            }
            registers.al = source;
            registers.ll = destination;
            self.base.1 = source;
            self.destination.1 = destination;
        } else if self.mode != Mode::Backward {
            let (low, wrapped) = self.base.1.overflowing_add(u16::from(self.count));
            if wrapped && self.trap {
                return Err(overflow(self.base, low, self.bat()));
            }
            self.pair.set_low(registers, low);
        }
        self.remaining -= self.count;
        if self.remaining != 0 {
            self.count = self.remaining.min(TRANSFER_WINDOW_WORDS);
            self.index = 0;
            self.write = false;
        }
        Ok(())
    }
}

/// Optional coprocessor 2; no unit instance exists in a no-DMA configuration.
#[derive(Default)]
pub struct DmaCoprocessor {
    burst: Option<Burst>,
}
impl DmaCoprocessor {
    pub const COPROCESSOR_ID: u8 = 2;
}
impl OptionalCoprocessor for DmaCoprocessor {
    fn id(&self) -> u8 {
        Self::COPROCESSOR_ID
    }
    fn start(&mut self, command: u16, registers: &mut Registers) -> Result<(), CoprocessorFault> {
        self.burst = None;
        let operation = command & COPROCESSOR_OPCODE_ID_MASK;
        if !matches!(operation, DMAR_OPCODE | DMAW_OPCODE | DMAT_OPCODE) {
            return Err(CoprocessorFault::invalid_command(command));
        }
        let copy = operation == DMAT_OPCODE;
        let count_mask = if copy {
            COPY_COUNT_MASK
        } else {
            REGISTER_BURST_COUNT_MASK
        };
        let count = u8::try_from(command & count_mask).unwrap();
        if count == 0 {
            return Ok(());
        }
        // DMA selector 00=a, 01=l, 10=s differs from the instruction AF layout.
        let pair = if copy {
            Pair::Address
        } else {
            match (command & ADDRESS_PAIR_MASK) >> ADDRESS_PAIR_SHIFT {
                0 => Pair::Address,
                1 => Pair::Link,
                2 => Pair::Stack,
                // Reserved operands are undefined architecturally; reject safely.
                _ => return Err(CoprocessorFault::invalid_command(command)),
            }
        };
        let mut burst = Burst {
            pair,
            write: operation == DMAW_OPCODE,
            mode: if copy {
                Mode::Copy
            } else if command & BACKWARD_DIRECTION_BIT != 0 {
                Mode::Backward
            } else {
                Mode::Forward
            },
            base: pair.read(registers),
            destination: (registers.lh, registers.ll),
            remaining: count,
            count: count.min(TRANSFER_WINDOW_WORDS),
            index: 0,
            trap: sr_bit_is_set(StatusRegisterFields::TrapOnAddressOverflow, registers),
        };
        if burst.mode == Mode::Backward {
            let (low, wrapped) = burst.base.1.overflowing_sub(u16::from(count));
            if wrapped && burst.trap {
                return Err(overflow(burst.base, low, burst.bat()));
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
        if burst.remaining == 0 {
            return Ok(BusAssertions::default());
        }
        let base = burst.access_base();
        let (low, wrapped) = base.1.overflowing_add(u16::from(burst.index));
        if wrapped && burst.trap {
            return Err(overflow(base, low, burst.bat()));
        }
        Ok(BusAssertions {
            address: (base.0, low).to_full_address(),
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
        if burst.index == burst.count {
            if burst.mode == Mode::Copy && !burst.write {
                // The actual r1..r7 window is the buffer. No hidden host copy
                // can conceal its partial reads or short-final-chunk clobbers.
                burst.write = true;
                burst.index = 0;
            } else {
                burst.finish_chunk(registers)?;
            }
        }
        Ok(())
    }
    fn is_finished(&self) -> bool {
        self.burst.as_ref().is_none_or(|burst| burst.remaining == 0)
    }
    fn reset(&mut self) {
        self.burst = None;
    }
}
