use peripheral_bus::device::{BusAccessType, BusAssertions, BusOperation};

use crate::{
    coprocessors::{exception_unit::definitions::Faults, shared::ExecutionPhase},
    raise_fault,
    registers::{
        sr_bit_is_set, AddressRegisterName, ExceptionUnitRegisters, FullAddressRegisterAccess,
        Registers, StatusRegisterFields,
    },
};

const OPERATION_MASK: u16 = 0x0F00;
const OPERATION_SHIFT: u16 = 8;
const OPERAND_MASK: u16 = 0x00FF;
const REGISTER_SELECT_MASK: u16 = 0b1100_0000;
const REGISTER_SELECT_SHIFT: u16 = 6;
const DIRECTION_BACKWARD_MASK: u16 = 0b0010_0000;
const RESERVED_REGISTER_OPERAND_MASK: u16 = 0b0001_1000;
const COUNT_MAGNITUDE_MASK: u16 = 0b0000_0111;

#[derive(Debug, PartialEq, Eq)]
enum DmaCommand {
    ReadRegisters(RegisterWindowCommand),
    WriteRegisters(RegisterWindowCommand),
    Transfer { count: u8 },
}

#[derive(Debug, PartialEq, Eq)]
struct RegisterWindowCommand {
    address_register: AddressRegisterName,
    direction: DmaDirection,
    count: u8,
}

#[derive(Debug, PartialEq, Eq)]
enum DmaDirection {
    Forward,
    Backward,
}

#[derive(Debug, PartialEq, Eq)]
enum DmaActiveCommand {
    NoOp,
    ReadRegisters(RegisterWindowState),
    WriteRegisters(RegisterWindowState),
    Transfer(TransferState),
}

#[derive(Debug, PartialEq, Eq)]
struct RegisterWindowState {
    address_register: AddressRegisterName,
    current_address: u32,
    next_register: u8,
    remaining: u8,
}

#[derive(Debug, PartialEq, Eq)]
enum TransferPhase {
    Read,
    Write,
}

#[derive(Debug, PartialEq, Eq)]
struct TransferState {
    source_address: u32,
    destination_address: u32,
    remaining: u8,
    next_register: u8,
    window_count: u8,
    phase: TransferPhase,
}

impl Default for DmaActiveCommand {
    fn default() -> Self {
        Self::NoOp
    }
}

#[derive(Default)]
pub struct DmaUnitExecutor {
    active_command: Option<DmaActiveCommand>,
}

impl DmaUnitExecutor {
    pub const COPROCESSOR_ID: u8 = 0x2;

    pub fn step(
        &mut self,
        phase: &ExecutionPhase,
        cause_register_value: u16,
        registers: &mut Registers,
        eu_registers: &mut ExceptionUnitRegisters,
        bus_assertions: BusAssertions,
    ) -> BusAssertions {
        if phase != &ExecutionPhase::InstructionFetchLow || eu_registers.pending_fault.is_some() {
            return BusAssertions::default();
        }

        if sr_bit_is_set(StatusRegisterFields::ProtectedMode, registers) {
            eu_registers.pending_fault =
                raise_fault(eu_registers, Faults::PrivilegeViolation, &bus_assertions);
            registers.pending_coprocessor_command = 0;
            self.active_command = None;
            return BusAssertions::default();
        }

        if self.active_command.is_none() {
            match decode_command(cause_register_value) {
                Ok(command) => {
                    self.active_command = Some(initial_command_state(command, registers));
                }
                Err(()) => {
                    eu_registers.pending_fault =
                        raise_fault(eu_registers, Faults::InvalidOpCode, &bus_assertions);
                    registers.pending_coprocessor_command = 0;
                    return BusAssertions::default();
                }
            }
        }

        let Some(active_command) = self.active_command.as_mut() else {
            return BusAssertions::default();
        };

        match active_command {
            DmaActiveCommand::NoOp => {
                self.finish(registers);
                BusAssertions::default()
            }
            DmaActiveCommand::ReadRegisters(state) => execute_register_read(state),
            DmaActiveCommand::WriteRegisters(state) => execute_register_write(state, registers),
            DmaActiveCommand::Transfer(state) => execute_transfer(state, registers),
        }
    }

    pub fn complete_bus_access(&mut self, registers: &mut Registers, data: u16) {
        let Some(active_command) = self.active_command.as_mut() else {
            return;
        };

        match active_command {
            DmaActiveCommand::NoOp => self.finish(registers),
            DmaActiveCommand::ReadRegisters(state) => {
                registers[state.next_register] = data;
                complete_register_window_access(state);
                if state.remaining == 0 {
                    update_address_register(
                        registers,
                        state.address_register,
                        state.current_address,
                    );
                    self.finish(registers);
                }
            }
            DmaActiveCommand::WriteRegisters(state) => {
                complete_register_window_access(state);
                if state.remaining == 0 {
                    update_address_register(
                        registers,
                        state.address_register,
                        state.current_address,
                    );
                    self.finish(registers);
                }
            }
            DmaActiveCommand::Transfer(state) => match state.phase {
                TransferPhase::Read => {
                    registers[state.next_register] = data;
                    state.source_address = state.source_address.wrapping_add(1);
                    state.next_register += 1;
                    state.window_count += 1;

                    if state.window_count == 7 || state.window_count == state.remaining {
                        state.phase = TransferPhase::Write;
                        state.next_register = 1;
                    }
                }
                TransferPhase::Write => {
                    state.destination_address = state.destination_address.wrapping_add(1);
                    state.next_register += 1;
                    state.remaining -= 1;
                    state.window_count -= 1;

                    if state.remaining == 0 {
                        registers.set_full_address_address(state.source_address);
                        registers.set_full_link_address(state.destination_address);
                        self.finish(registers);
                    } else if state.window_count == 0 {
                        state.phase = TransferPhase::Read;
                        state.next_register = 1;
                    }
                }
            },
        }
    }

    pub fn abort(&mut self, registers: &mut Registers) {
        registers.pending_coprocessor_command = 0;
        self.active_command = None;
    }

    pub fn is_active(&self) -> bool {
        self.active_command.is_some()
    }

    fn finish(&mut self, registers: &mut Registers) {
        registers.pending_coprocessor_command = 0;
        self.active_command = None;
    }
}

fn decode_command(cause_register_value: u16) -> Result<DmaCommand, ()> {
    let operation = (cause_register_value & OPERATION_MASK) >> OPERATION_SHIFT;
    let operand = cause_register_value & OPERAND_MASK;

    match operation {
        0x8 => Ok(DmaCommand::ReadRegisters(decode_register_window_command(
            operand,
        )?)),
        0x9 => Ok(DmaCommand::WriteRegisters(decode_register_window_command(
            operand,
        )?)),
        0xA => Ok(DmaCommand::Transfer {
            count: operand.try_into().expect("operand is masked to 8 bits"),
        }),
        _ => Err(()),
    }
}

fn decode_register_window_command(operand: u16) -> Result<RegisterWindowCommand, ()> {
    if operand & RESERVED_REGISTER_OPERAND_MASK != 0 {
        return Err(());
    }

    let address_register = match (operand & REGISTER_SELECT_MASK) >> REGISTER_SELECT_SHIFT {
        0b00 => AddressRegisterName::Address,
        0b01 => AddressRegisterName::LinkRegister,
        0b10 => AddressRegisterName::StackPointer,
        0b11 => return Err(()),
        _ => unreachable!("register select is masked to 2 bits"),
    };
    let direction = if operand & DIRECTION_BACKWARD_MASK == DIRECTION_BACKWARD_MASK {
        DmaDirection::Backward
    } else {
        DmaDirection::Forward
    };
    let count = (operand & COUNT_MAGNITUDE_MASK)
        .try_into()
        .expect("count magnitude is masked to 3 bits");

    Ok(RegisterWindowCommand {
        address_register,
        direction,
        count,
    })
}

fn initial_command_state(command: DmaCommand, registers: &Registers) -> DmaActiveCommand {
    match command {
        DmaCommand::ReadRegisters(command) => {
            if command.count == 0 {
                return DmaActiveCommand::NoOp;
            }
            DmaActiveCommand::ReadRegisters(initial_register_window_state(command, registers))
        }
        DmaCommand::WriteRegisters(command) => {
            if command.count == 0 {
                return DmaActiveCommand::NoOp;
            }
            DmaActiveCommand::WriteRegisters(initial_register_window_state(command, registers))
        }
        DmaCommand::Transfer { count } => {
            if count == 0 {
                return DmaActiveCommand::NoOp;
            }
            DmaActiveCommand::Transfer(TransferState {
                source_address: registers.get_full_address_address(),
                destination_address: registers.get_full_link_address(),
                remaining: count,
                next_register: 1,
                window_count: 0,
                phase: TransferPhase::Read,
            })
        }
    }
}

fn initial_register_window_state(
    command: RegisterWindowCommand,
    registers: &Registers,
) -> RegisterWindowState {
    let original_address = get_address_register(registers, command.address_register);
    let current_address = match command.direction {
        DmaDirection::Forward => original_address,
        DmaDirection::Backward => original_address.wrapping_sub(u32::from(command.count)),
    };

    RegisterWindowState {
        address_register: command.address_register,
        current_address,
        next_register: 1,
        remaining: command.count,
    }
}

fn execute_register_read(state: &RegisterWindowState) -> BusAssertions {
    BusAssertions {
        address: state.current_address,
        op: BusOperation::Read,
        bus_access_strobe: true,
        bus_access_type: BusAccessType::DmaReadBurst,
        ..BusAssertions::default()
    }
}

fn execute_register_write(state: &RegisterWindowState, registers: &Registers) -> BusAssertions {
    BusAssertions {
        address: state.current_address,
        data: registers[state.next_register],
        op: BusOperation::Write,
        bus_access_strobe: true,
        bus_access_type: BusAccessType::DmaWriteBurst,
        ..BusAssertions::default()
    }
}

fn execute_transfer(state: &TransferState, registers: &Registers) -> BusAssertions {
    match state.phase {
        TransferPhase::Read => BusAssertions {
            address: state.source_address,
            op: BusOperation::Read,
            bus_access_strobe: true,
            bus_access_type: BusAccessType::DmaReadBurst,
            ..BusAssertions::default()
        },
        TransferPhase::Write => BusAssertions {
            address: state.destination_address,
            data: registers[state.next_register],
            op: BusOperation::Write,
            bus_access_strobe: true,
            bus_access_type: BusAccessType::DmaWriteBurst,
            ..BusAssertions::default()
        },
    }
}

fn complete_register_window_access(state: &mut RegisterWindowState) {
    state.current_address = state.current_address.wrapping_add(1);
    state.next_register += 1;
    state.remaining -= 1;
}

fn get_address_register(registers: &Registers, address_register: AddressRegisterName) -> u32 {
    match address_register {
        AddressRegisterName::Address => registers.get_full_address_address(),
        AddressRegisterName::LinkRegister => registers.get_full_link_address(),
        AddressRegisterName::StackPointer => registers.get_full_sp_address(),
        AddressRegisterName::ProgramCounter => registers.get_full_pc_address(),
    }
}

fn update_address_register(
    registers: &mut Registers,
    address_register: AddressRegisterName,
    address: u32,
) {
    match address_register {
        AddressRegisterName::Address => registers.set_full_address_address(address),
        AddressRegisterName::LinkRegister => registers.set_full_link_address(address),
        AddressRegisterName::StackPointer => registers.set_full_sp_address(address),
        AddressRegisterName::ProgramCounter => registers.set_full_pc_address(address),
    }
}
