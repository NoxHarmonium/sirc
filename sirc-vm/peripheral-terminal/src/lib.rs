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

use peripheral_bus::memory_mapped_device::MemoryMappedDevice;

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

// TODO: Rename to TerminalDevice
pub struct TerminalPeripheral {
    stdin_channel: Receiver<String>,
    stdin_buffer: VecDeque<u8>,
}

#[must_use]
pub fn new_terminal_peripheral() -> TerminalPeripheral {
    let stdin_channel = spawn_stdin_channel();
    TerminalPeripheral {
        stdin_channel,
        stdin_buffer: VecDeque::new(),
    }
}

impl MemoryMappedDevice for TerminalPeripheral {
    ///  # Panics
    /// Will panic if there is an unexpected error in the channel that reads from stdin
    fn poll(&mut self) {
        match self.stdin_channel.try_recv() {
            Ok(data) => {
                // TODO: Write the result somewhere and flag exception
                println!("Received: {data}");
                self.stdin_buffer.extend(data.as_bytes());
                // Placeholder
                //
                // self.memory_peripheral.write_address(0x0, 0x1);
            }
            Err(TryRecvError::Empty) => {
                // Nothing to do
            }
            Err(TryRecvError::Disconnected) => {
                panic!("TP: stdin channel disconnected unexpectedly")
            }
        }
    }

    fn read_address(&self, _address: u32) -> u16 {
        todo!()
    }

    fn write_address(&mut self, _address: u32, _value: u16) {
        todo!()
    }

    fn read_raw_bytes(&self, _limit: u32) -> Vec<u8> {
        todo!()
    }

    fn write_raw_bytes(&mut self, _binary_data: &[u8]) {
        todo!()
    }
}
