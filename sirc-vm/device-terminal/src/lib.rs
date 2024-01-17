#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    // I don't like this rule
    clippy::module_name_repetitions,
    // Might be good practice but too much work for now
    clippy::missing_errors_doc,
)]
#![deny(warnings)]

use log::debug;
use log::error;

use peripheral_bus::device::BusAssertions;
use peripheral_bus::device::Device;
use peripheral_bus::memory_mapped_device::MemoryMappedDevice;

use std::collections::VecDeque;
use std::io;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::thread;

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
        let bytes_read = io::stdin().read_line(&mut buffer).unwrap();
        if bytes_read == 0 {
            // read_line always returns zero bytes_read when EOF has been reached (nothing)
            break;
        }
        if let Err(error) = tx.send(buffer) {
            error!("Error reading stdin: {error:?}");
            // If we get an error we probably wan't to clean up rather than try and read anything else from the stream
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
pub struct TerminalDevice {
    reading_finished: bool,
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
        reading_finished: false,
        clock_counter: 0,
        master_clock_freq,
        stdin_channel,
        stdin_buffer: VecDeque::new(),
        control_registers: TerminalDeviceControlRegisters::default(),
    }
}

impl Device for TerminalDevice {
    ///  # Panics
    /// Will panic if there is an unexpected error in the channel that reads from stdin
    fn poll(&mut self) -> BusAssertions {
        // Pull any pending data from stdin into the virtual buffer
        if !self.reading_finished {
            match self.stdin_channel.try_recv() {
                Ok(data) => {
                    // TODO: Write the result somewhere and flag exception
                    debug!("Received: [{:X?}]", data.as_bytes());
                    self.stdin_buffer.extend(data.as_bytes());
                }
                Err(TryRecvError::Empty) => {
                    // Nothing to do
                }
                Err(TryRecvError::Disconnected) => {
                    debug!("Channel to read stdin has closed. This does not necessarily mean an error occurred if EOF was encountered");
                    self.reading_finished = true;
                }
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

        // Drip feed the actual data register that the CPU can access at the rate specified by baud
        if self.control_registers.recv_enabled == REGISTER_TRUE && should_activate {
            if let Some(data) = self.stdin_buffer.pop_front() {
                debug!("Data received: [0x{data:X}]");
                self.control_registers.recv_data = u16::from(data);
                self.control_registers.recv_pending = REGISTER_TRUE;
            }
        }

        if self.control_registers.send_enabled == REGISTER_TRUE
            && should_activate
            && self.control_registers.send_pending == REGISTER_TRUE
        {
            self.control_registers.send_pending = REGISTER_FALSE;
        }

        // TODO: Maybe this should disable when send AND recv are both disabled
        // but it doesn't have to be too accurate for now
        if should_activate {
            self.clock_counter = 0;
        } else {
            self.clock_counter += 1;
        }

        if (self.control_registers.recv_enabled == REGISTER_TRUE
            && self.control_registers.recv_pending == REGISTER_TRUE)
            || (self.control_registers.send_enabled == REGISTER_TRUE
                && self.control_registers.send_pending == REGISTER_FALSE)
        {
            return BusAssertions {
                interrupt_assertion: 0x2,
                ..BusAssertions::default()
            };
        }
        BusAssertions::default()
    }
}

#[allow(clippy::cast_possible_truncation)]
impl MemoryMappedDevice for TerminalDevice {
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
