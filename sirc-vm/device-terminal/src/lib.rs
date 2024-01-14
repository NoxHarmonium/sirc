#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    // I don't like this rule
    clippy::module_name_repetitions,
    // Might be good practice but too much work for now
    clippy::missing_errors_doc,
)]
#![deny(warnings)]

use std::collections::VecDeque;
use std::io;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::thread;

use peripheral_bus::memory_mapped_device::BusAssertions;
use peripheral_bus::memory_mapped_device::MemoryMappedDevice;

// TODO: Could use a trait on u16 for this?
const REGISTER_FALSE: u16 = 0x0;
const REGISTER_TRUE: u16 = 0x1;

fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        // Note: ideally we would pass the bytes through as they are input and not wait for a newline
        // However, due to line buffering we probably wouldn't get the input until a new line anyway
        // If it is an issue we could use a crate or maybe use raw mode but it's not too important
        // at this point.
        io::stdin().read_line(&mut buffer).unwrap();
        if tx.send(buffer).is_err() {
            // Stop looping if there is an error
            break;
        }
    });
    rx
}

#[derive(Default, Debug)]
pub struct TerminalDeviceControlRegisters {
    baud: u16,
    recv_enabled: u16,
    recv_pending: u16,
    recv_data: u16,
    send_enabled: u16,
    send_pending: u16,
    send_data: u16,
}
// TODO: Rename to TerminalDevice
pub struct TerminalDevice {
    master_clock_freq: u32,
    clock_counter: usize,
    stdin_channel: Receiver<String>,
    stdin_buffer: VecDeque<u8>,
    control_registers: TerminalDeviceControlRegisters,
}

#[must_use]
pub fn new_terminal_device(master_clock_freq: u32) -> TerminalDevice {
    let stdin_channel = spawn_stdin_channel();
    TerminalDevice {
        clock_counter: 0,
        master_clock_freq,
        stdin_channel,
        stdin_buffer: VecDeque::new(),
        control_registers: TerminalDeviceControlRegisters::default(),
    }
}

#[allow(clippy::cast_possible_truncation)]
impl MemoryMappedDevice for TerminalDevice {
    ///  # Panics
    /// Will panic if there is an unexpected error in the channel that reads from stdin
    fn poll(&mut self) -> BusAssertions {
        // Pull any pending data from stdin into the virtual buffer
        match self.stdin_channel.try_recv() {
            Ok(data) => {
                // TODO: Write the result somewhere and flag exception
                println!("Received: {data}");
                self.stdin_buffer.extend(data.as_bytes());
            }
            Err(TryRecvError::Empty) => {
                // Nothing to do
            }
            Err(TryRecvError::Disconnected) => {
                panic!("TP: stdin channel disconnected unexpectedly")
            }
        }

        let clocks_per_recv: Option<usize> = if self.control_registers.baud > 0 {
            Some(self.master_clock_freq as usize / self.control_registers.baud as usize)
        } else {
            None
        };

        let should_activate = if let Some(clocks_per_recv) = clocks_per_recv {
            self.clock_counter >= clocks_per_recv
        } else {
            false
        };

        // println!(
        //     "clock_counter: {} should_activate: {should_activate} clocks_per_recv: {clocks_per_recv:?} struct {:?}",
        //     self.clock_counter, self.control_registers
        // );

        // Drip feed the actual data register that the CPU can access at the rate specified by baud
        if self.control_registers.recv_enabled == REGISTER_TRUE && should_activate {
            if let Some(data) = self.stdin_buffer.pop_front() {
                println!("READ OUT OF stdin_buffer {data:X}");
                self.control_registers.recv_data = u16::from(data);
                self.control_registers.recv_pending = REGISTER_TRUE;
            }
        }

        if self.control_registers.send_enabled == REGISTER_TRUE
            && should_activate
            && self.control_registers.send_pending == REGISTER_TRUE
        {
            print!("{}", char::from(self.control_registers.send_data as u8));
            self.control_registers.send_pending = REGISTER_FALSE;
        }

        // TODO: Maybe this should disable when send AND recv are both disabled
        // but it doesn't have to be too accurate for now
        if should_activate {
            self.clock_counter = 0;
        } else {
            self.clock_counter += 1;
        }

        if self.control_registers.recv_pending == REGISTER_TRUE {
            return BusAssertions {
                interrupt_assertion: 0x2,
            };
        }
        BusAssertions::default()
    }

    fn read_address(&self, address: u32) -> u16 {
        match address {
            0x0 => self.control_registers.baud,
            0x1 => self.control_registers.recv_enabled,
            0x2 => self.control_registers.recv_pending,
            0x3 => self.control_registers.recv_data,
            0x4 => self.control_registers.send_enabled,
            0x5 => self.control_registers.send_pending,
            0x6 => self.control_registers.send_data,
            _ => 0x0,
        }
    }

    fn write_address(&mut self, address: u32, value: u16) {
        match address {
            0x0 => self.control_registers.baud = value,
            0x1 => self.control_registers.recv_enabled = value,
            0x2 => {
                self.control_registers.recv_pending = value;
            }
            0x4 => self.control_registers.send_enabled = value,
            0x5 => self.control_registers.send_pending = value,
            0x6 => self.control_registers.send_data = value,
            _ => {}
        }
    }
}
