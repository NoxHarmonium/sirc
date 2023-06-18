use crate::instructions::definitions::DecodedInstruction;

#[derive(Debug, Default)]
pub struct IntermediateRegisters {
    pub alu_output: u16,
    pub lmd: u16,
    pub npc: u16,
}

pub enum ExecutionStage {
    // TODO: Work out a cleaner way to specify the data in each stage
    Execution(DecodedInstruction),
    MemoryAccessAndBranchCompletion(DecodedInstruction),
    WriteBack(DecodedInstruction),
}

// ///
// /// Transfer the lower word address register to the lower word of the program counter.
// ///
// /// Allows a jump to anywhere within a segment.
// ///
// /// Not a privileged operation because it only allows the program to jump within its
// /// current segment.
// ///
// /// ```
// /// use peripheral_cpu::instructions::definitions::{ImmediateInstructionData, ShortJumpWithImmediateData, ConditionFlags};
// /// use peripheral_cpu::registers::{Registers, FullAddressRegisterAccess};
// /// use peripheral_cpu::executors::Executor;
// /// use peripheral_mem::new_memory_peripheral;
// ///
// ///
// /// let jumpInstruction = ShortJumpWithImmediateData {
// ///   data: ImmediateInstructionData {
// ///     register: 0x0, // unused
// ///     value: 0xCAFE,
// ///     condition_flag: ConditionFlags::Always,
// ///     additional_flags: 0x0
// ///   }
// /// };
// /// let mut registers = Registers::default();
// /// registers.set_full_address_address(0x00CAFECA);
// /// let mem = new_memory_peripheral();
// /// jumpInstruction.execute(&mut registers, &mem);
// ///
// /// assert_eq!((registers.ph, registers.pl), (0x0000, 0xCAFE));
// ///
// /// ```
// ///
// impl Executor for ShortJumpWithImmediateData {
//     // ID: 0x18
//     fn execute(
//         &self,
//         registers: &mut Registers,
//         _mem: &MemoryPeripheral,
//         stage: &ExecutionStage,
//         intermediateRegisters: &IntermediateRegisters,
//     ) {
//         match stage {
//             ExecutionStage::Execution(DecodedInstruction { imm, .. }) => {
//                 intermediateRegisters.AluOutput = *imm;
//             }
//             ExecutionStage::MemoryAccessAndBranchCompletion(_) => {
//                 registers.pl = intermediateRegisters.AluOutput;
//             }
//             _ => {}
//         }
//     }
// }

// impl Executor for ShortJumpToSubroutineWithImmediateData {
//     // ID: 0x19
//     fn execute(
//         &self,
//         registers: &mut Registers,
//         mem: &MemoryPeripheral,
//         stage: &ExecutionStage,
//         intermediateRegisters: &IntermediateRegisters,
//     ) {
//         // Even though it is only a short jump and we only need a partial address
//         // to return, we push the entire register to be consistent and avoid issues
//         // if the system mode bit is flipped during a subroutine
//         // push_address_to_stack(registers, mem, registers.get_full_pc_address());
//         // registers.pl = self.data.value;

//         // TODO: Subroutines
//     }
// }

// impl Executor for BranchToSubroutineData {
//     // ID: 0x1A
//     fn execute(
//         &self,
//         registers: &mut Registers,
//         mem: &MemoryPeripheral,
//         stage: &ExecutionStage,
//         intermediateRegisters: &IntermediateRegisters,
//     ) {
//         // Even though it is only a short jump and we only need a partial address
//         // to return, we push the entire register to be consistent and avoid issues
//         // if the system mode bit is flipped during a subroutine
//         // push_address_to_stack(registers, mem, registers.get_full_pc_address());
//         // let effective_address =
//         //     calculate_effective_address_with_pc_offset(registers, self.data.value);
//         // registers.set_full_pc_address(effective_address);

//         // TODO: Subroutines
//     }
// }

// ///
// /// Moves the CPU program counter by the specified offset, relative
// /// to the current program counter.
// ///
// /// Useful for making position independent programs (and avoiding having to
// /// store in the address register)
// ///
// ///
// impl Executor for BranchInstructionData {
//     // ID: 0x1B
//     fn execute(
//         &self,
//         registers: &mut Registers,
//         _mem: &MemoryPeripheral,
//         stage: &ExecutionStage,
//         intermediateRegisters: &IntermediateRegisters,
//     ) {
//         match stage {
//             ExecutionStage::Execution(DecodedInstruction { imm, .. }) => {
//                 let (_, pl) = calculate_effective_address_with_pc_offset(registers, *imm)
//                     .to_segmented_address();

//                 intermediateRegisters.AluOutput = pl;
//             }
//             ExecutionStage::MemoryAccessAndBranchCompletion(_) => {
//                 registers.pl = intermediateRegisters.AluOutput;
//             }
//             _ => {}
//         }
//     }
// }

// ///
// /// Transfer the address registers to the program counter.
// ///
// /// Allows a full 24-bit address to be loaded into the address registers
// /// and used to specify where execution starts.
// ///
// /// This should be a privileged operation because it allows the program to jump
// /// anywhere it wants (escape its segment).
// ///
// /// ```
// /// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, LongJumpWithAddressRegisterData, ConditionFlags};
// /// use peripheral_cpu::registers::{Registers, FullAddressRegisterAccess};
// /// use peripheral_cpu::executors::Executor;
// /// use peripheral_mem::new_memory_peripheral;
// ///
// ///
// /// let jumpInstruction = LongJumpWithAddressRegisterData {
// ///   data: RegisterInstructionData {
// ///     r1: 0x0,
// ///     r2: 0x0,
// ///     r3: 0x0,
// ///     condition_flag: ConditionFlags::Always,
// ///     additional_flags: 0x0
// ///   }
// /// };
// /// let mut registers = Registers::default();
// /// registers.set_full_address_address(0x00CAFECA);
// /// let mem = new_memory_peripheral();
// /// jumpInstruction.execute(&mut registers, &mem);
// ///
// /// assert_eq!((registers.ph, registers.pl), (0x00CA, 0xFECA));
// ///
// /// ```
// ///
// impl Executor for LongJumpWithAddressRegisterData {
//     // ID: 0x1C
//     fn execute(
//         &self,
//         registers: &mut Registers,
//         _mem: &MemoryPeripheral,
//         stage: &ExecutionStage,
//         intermediateRegisters: &IntermediateRegisters,
//     ) {
//         match stage {
//             ExecutionStage::MemoryAccessAndBranchCompletion(DecodedInstruction {
//                 ad_h_,
//                 ad_l_,
//                 ..
//             }) => {
//                 // TODO: Define this properly in the sequence in notes.md
//                 (registers.ph, registers.pl) = (*ad_h_, *ad_l_);
//             }
//             _ => {}
//         }
//     }
// }

// // Note: Privileged as stack could be overwritten to jump anywhere
// impl Executor for LongJumpToSubroutineWithAddressRegisterData {
//     // ID: 0x1D
//     fn execute(
//         &self,
//         registers: &mut Registers,
//         mem: &MemoryPeripheral,
//         stage: &ExecutionStage,
//         intermediateRegisters: &IntermediateRegisters,
//     ) {
//         // TODO: Subroutines

//         // push_address_to_stack(registers, mem, registers.get_full_pc_address());
//         // (registers.ph, registers.pl) = registers.get_segmented_address();
//         // TODO: Should these long jumps be able to jump to s or p register sets too?
//         // probably not, but maybe that would make it more consistent
//     }
// }

// // Data Access

// impl Executor for LoadEffectiveAddressFromIndirectImmediateData {
//     // ID: 0x1F
//     fn execute(
//         &self,
//         registers: &mut Registers,
//         _mem: &MemoryPeripheral,
//         stage: &ExecutionStage,
//         intermediateRegisters: &IntermediateRegisters,
//     ) {
//         let source_address_register = self.data.additional_flags;
//         let dest_address_register = self.data.register;

//         let new_address = calculate_effective_address_with_immediate(
//             registers,
//             source_address_register,
//             self.data.value,
//         );

//         registers.set_address_register_at_index(dest_address_register, new_address);
//     }
// }

// impl Executor for LoadEffectiveAddressFromIndirectRegisterData {
//     // ID: 0x20
//     fn execute(
//         &self,
//         registers: &mut Registers,
//         _mem: &MemoryPeripheral,
//         stage: &ExecutionStage,
//         intermediateRegisters: &IntermediateRegisters,
//     ) {
//         let dest_address_register = self.data.r1;
//         let displacement_register = self.data.r2;
//         let source_address_register = self.data.r3;

//         let new_address = calculate_effective_address_with_register(
//             registers,
//             source_address_register,
//             displacement_register,
//         );

//         registers.set_address_register_at_index(dest_address_register, new_address);
//     }
// }

// ///
// /// Loads an immediate 16 bit value encoded in an instruction into a register.
// ///
// /// ```
// /// use peripheral_cpu::instructions::definitions::{ImmediateInstructionData, LoadRegisterFromImmediateData, ConditionFlags};
// /// use peripheral_cpu::registers::Registers;
// /// use peripheral_cpu::executors::Executor;
// /// use peripheral_mem::new_memory_peripheral;
// ///
// ///
// /// let loadRegisterFromImmediateInstruction = LoadRegisterFromImmediateData {
// ///   data: ImmediateInstructionData {
// ///     register: 0x02, // z1
// ///     value: 0xCAFE,
// ///     condition_flag: ConditionFlags::Always,
// ///     additional_flags: 0x00,
// ///   }
// /// };
// ///
// /// let mut registers = Registers::default();
// ///
// /// let mut mem = new_memory_peripheral();
// ///
// /// loadRegisterFromImmediateInstruction.execute(&mut registers, &mem);
// ///
// /// assert_eq!(registers.z1, 0xCAFE);
// ///
// /// ```
// ///
// impl Executor for LoadRegisterFromImmediateData {
//     // ID: 0x22
//     fn execute(
//         &self,
//         registers: &mut Registers,
//         _mem: &MemoryPeripheral,
//         stage: &ExecutionStage,
//         intermediateRegisters: &IntermediateRegisters,
//     ) {
//         registers.set_at_index(self.data.register, self.data.value);
//     }
// }

// ///
// /// Loads a 16 bit value from a register into another register.
// ///
// /// ```
// /// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, LoadRegisterFromRegisterData, ConditionFlags};
// /// use peripheral_cpu::registers::Registers;
// /// use peripheral_cpu::executors::Executor;
// /// use peripheral_mem::new_memory_peripheral;
// ///
// ///
// /// let loadRegisterFromRegisterData = LoadRegisterFromRegisterData {
// ///   data: RegisterInstructionData {
// ///     r1: 0x02, // z1
// ///     r2: 0x00, // x1
// ///     r3: 0x00, // Unused
// ///     condition_flag: ConditionFlags::Always,
// ///     additional_flags: 0x00,
// ///   }
// /// };
// /// let mut registers = Registers { x1: 0xCAFE, ..Registers::default() };
// ///
// /// let mut mem = new_memory_peripheral();
// ///
// /// loadRegisterFromRegisterData.execute(&mut registers, &mem);
// ///
// /// assert_eq!(registers.z1, 0xCAFE);
// ///
// /// ```
// ///
// impl Executor for LoadRegisterFromRegisterData {
//     // ID: 0x23
//     fn execute(
//         &self,
//         registers: &mut Registers,
//         _mem: &MemoryPeripheral,
//         stage: &ExecutionStage,
//         intermediateRegisters: &IntermediateRegisters,
//     ) {
//         let source_value = registers.get_at_index(self.data.r2);
//         registers.set_at_index(self.data.r1, source_value)
//     }
// }

// ///
// /// Loads a 16 bit value out of a memory address into a register.
// ///
// /// The base address value is specified by the address registers (ah/al)
// /// the first operand is the destination register and the operand is an
// /// immediate offset to add to the base address value.
// ///
// /// ```
// /// use peripheral_cpu::instructions::definitions::{ImmediateInstructionData, LoadRegisterFromIndirectImmediateData, ConditionFlags};
// /// use peripheral_cpu::registers::Registers;
// /// use peripheral_cpu::executors::Executor;
// /// use peripheral_mem::new_memory_peripheral;
// ///
// ///
// /// let loadRegisterFromIndirectImmediate = LoadRegisterFromIndirectImmediateData {
// ///   data: ImmediateInstructionData {
// ///     register: 0x02, // z1
// ///     value: 0x0001,
// ///     condition_flag: ConditionFlags::Always,
// ///     additional_flags: 0x00,
// ///   }
// /// };
// /// let mut registers = Registers { ah: 0x1011, al: 0x1110, ..Registers::default() };
// ///
// /// let mut mem = new_memory_peripheral();
// /// mem.map_segment("TEST", 0x00110000, 0xFFFF, true);
// /// mem.write_address(0x00111111, 0xCAFE);
// ///
// /// loadRegisterFromIndirectImmediate.execute(&mut registers, &mem);
// ///
// /// assert_eq!(registers.z1, 0xCAFE);
// ///
// /// ```
// ///
// impl Executor for LoadRegisterFromIndirectImmediateData {
//     // ID: 0x24
//     fn execute(
//         &self,
//         registers: &mut Registers,
//         mem: &MemoryPeripheral,
//         stage: &ExecutionStage,
//         intermediateRegisters: &IntermediateRegisters,
//     ) {
//         let address_to_read = calculate_effective_address_with_immediate(
//             registers,
//             self.data.additional_flags,
//             self.data.value,
//         );
//         let read_value = mem.read_address(address_to_read);
//         registers.set_at_index(self.data.register, read_value);
//     }
// }

// ///
// /// Loads a 16 bit value out of a memory address into a register.
// ///
// /// The base address value is specified by the address registers (ah/al)
// /// the first operand is the destination register and the second register
// /// is an offset to add to the base address value.
// ///
// /// ```
// /// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, LoadRegisterFromIndirectRegisterData, ConditionFlags};
// /// use peripheral_cpu::registers::Registers;
// /// use peripheral_cpu::executors::Executor;
// /// use peripheral_mem::new_memory_peripheral;
// ///
// ///
// /// let loadOffsetRegisterInstruction = LoadRegisterFromIndirectRegisterData {
// ///   data: RegisterInstructionData {
// ///     r1: 0x00, // x1
// ///     r2: 0x03, // x2
// ///     r3: 0x00, // unused
// ///     condition_flag: ConditionFlags::Always,
// ///     additional_flags: 0x00,
// ///   }
// /// };
// /// let mut registers = Registers { ah: 0x1011, al: 0x1110, x2: 0x0001, ..Registers::default() };
// ///
// /// let mut mem = new_memory_peripheral();
// /// mem.map_segment("TEST", 0x00110000, 0xFFFF, true);
// /// mem.write_address(0x00111111, 0xCAFE);
// ///
// /// loadOffsetRegisterInstruction.execute(&mut registers, &mem);
// ///
// /// assert_eq!(registers.x1, 0xCAFE);
// ///
// /// ```
// ///
// impl Executor for LoadRegisterFromIndirectRegisterData {
//     // ID: 0x25
//     fn execute(
//         &self,
//         registers: &mut Registers,
//         mem: &MemoryPeripheral,
//         stage: &ExecutionStage,
//         intermediateRegisters: &IntermediateRegisters,
//     ) {
//         let address_to_read =
//             calculate_effective_address_with_register(registers, self.data.r3, self.data.r2);
//         let read_value = mem.read_address(address_to_read);
//         registers.set_at_index(self.data.r1, read_value);
//     }
// }

// ///
// /// Stores a 16 bit value from a register into a memory address.
// ///
// /// The base address value is specified by the address registers (ah/al)
// /// the first operand is the destination register and the operand is an
// /// immediate offset to add to the base address value.
// ///
// /// ```
// /// use peripheral_cpu::instructions::definitions::{ImmediateInstructionData, StoreRegisterToIndirectImmediateData, ConditionFlags};
// /// use peripheral_cpu::registers::Registers;
// /// use peripheral_cpu::executors::Executor;
// /// use peripheral_mem::new_memory_peripheral;
// ///
// ///
// /// let storeRegisterFromIndirectImmediate = StoreRegisterToIndirectImmediateData {
// ///   data: ImmediateInstructionData {
// ///     register: 0x02, // z1
// ///     value: 0x0001,
// ///     condition_flag: ConditionFlags::Always,
// ///     additional_flags: 0x00,
// ///   }
// /// };
// /// let mut registers = Registers { z1: 0xCAFE, ah: 0x1011, al: 0x1110, ..Registers::default() };
// ///
// /// let mut mem = new_memory_peripheral();
// /// mem.map_segment("TEST", 0x00110000, 0xFFFF, true);
// ///
// /// storeRegisterFromIndirectImmediate.execute(&mut registers, &mem);
// /// let stored_value = mem.read_address(0x00111111);
// ///
// /// assert_eq!(stored_value, 0xCAFE);
// ///
// /// ```
// ///
// impl Executor for StoreRegisterToIndirectImmediateData {
//     // ID: 0x26
//     fn execute(
//         &self,
//         registers: &mut Registers,
//         mem: &MemoryPeripheral,
//         stage: &ExecutionStage,
//         intermediateRegisters: &IntermediateRegisters,
//     ) {
//         let address_to_write = calculate_effective_address_with_immediate(
//             registers,
//             self.data.additional_flags,
//             self.data.value,
//         );
//         mem.write_address(address_to_write, registers.get_at_index(self.data.register));
//     }
// }

// ///
// /// Stores a 16 bit value from a register into a memory address.
// ///
// /// The base address value is specified by the address registers (ah/al)
// /// the first operand is the destination register and the second register
// /// is an offset to add to the base address value.
// ///
// /// ```
// /// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, StoreRegisterToIndirectRegisterData, ConditionFlags};
// /// use peripheral_cpu::registers::Registers;
// /// use peripheral_cpu::executors::Executor;
// /// use peripheral_mem::new_memory_peripheral;
// ///
// ///
// /// let storeRegisterToIndirectRegister = StoreRegisterToIndirectRegisterData {
// ///   data: RegisterInstructionData {
// ///     r1: 0x00, // x1
// ///     r2: 0x03, // x2
// ///     r3: 0x00, // unused
// ///     condition_flag: ConditionFlags::Always,
// ///     additional_flags: 0x00,
// ///   }
// /// };
// /// let mut registers = Registers { ah: 0x1011, al: 0x1110, x1: 0xCAFE, x2: 0x0001, ..Registers::default() };
// ///
// /// let mut mem = new_memory_peripheral();
// /// mem.map_segment("TEST", 0x00110000, 0xFFFF, true);
// ///
// /// storeRegisterToIndirectRegister.execute(&mut registers, &mem);
// /// let stored_value = mem.read_address(0x00111111);
// ///
// /// assert_eq!(stored_value, 0xCAFE);
// /// ```
// ///
// impl Executor for StoreRegisterToIndirectRegisterData {
//     // ID: 0x27
//     fn execute(
//         &self,
//         registers: &mut Registers,
//         mem: &MemoryPeripheral,
//         stage: &ExecutionStage,
//         intermediateRegisters: &IntermediateRegisters,
//     ) {
//         let address_to_write =
//             calculate_effective_address_with_register(registers, self.data.r3, self.data.r2);
//         mem.write_address(address_to_write, registers.get_at_index(self.data.r1));
//     }
// }

// ///
// /// Stores multiple 16 bit values out a range of registers into sequential memory addresses.
// ///
// /// The base address value is specified by the given address register.
// /// The only supported source addressing mode is immediate displacement but only an offset of +- 128
// /// due to encoding constraints.
// ///
// /// The address will be decremented for each value that is written into memory.
// ///
// /// E.g. the following instruction:
// ///
// /// STMR (#-1, s), x1->z1
// ///
// /// where s is 0x0A
// ///
// /// Will result in the following writes
// ///
// /// 0x09 <- z1
// /// 0x08 <- y1
// /// 0x07 <- x1
// ///
// /// The given address register will be overwritten with the final address that was written to.
// /// This is to make storing a range of registers into the stack faster and easier.
// /// If you are storing into a structure that isn't a stack, you'll need to take that into account
// /// and possibly use the size of the structure as the immediate displacement.
// ///
// /// E.g.
// ///
// /// STMR (#3, a), x1->z1
// ///
// /// ```
// /// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, StoreManyRegisterFromAddressRegisterData, ConditionFlags};
// /// use peripheral_cpu::registers::{Registers, SegmentedRegisterAccess, SegmentedAddress};
// /// use peripheral_cpu::executors::Executor;
// /// use peripheral_mem::new_memory_peripheral;
// ///
// ///
// /// let storeManyRegisterFromAddressRegister = StoreManyRegisterFromAddressRegisterData {
// ///   data: RegisterInstructionData {
// ///     r1: 0x00, // x1
// ///     r2: 0x02, // z1
// ///     r3: 0x02, // s (sh/sl)
// ///     condition_flag: ConditionFlags::Always,
// ///     additional_flags: !0u8, // -1
// ///   }
// /// };
// /// let start_address_h = 0x1011;
// /// let start_address_l = 0x1110;
// /// let mut registers = Registers::default();
// /// registers.set_segmented_sp((start_address_h, start_address_l));
// ///
// /// registers.x1 = 0xCAFE;
// /// registers.y1 = 0xEFAC;
// /// registers.z1 = 0xBEEF;
// ///
// /// let mut mem = new_memory_peripheral();
// /// mem.map_segment("TEST", 0x00110000, 0xFFFF, true);
// ///
// /// storeManyRegisterFromAddressRegister.execute(&mut registers, &mem);
// ///
// /// // Offsets are -1 because of the immediate displacement of -1 (see additional_flags)
// /// let written_x1 = mem.read_address((start_address_h, start_address_l - 1).to_full_address());
// /// let written_y1 = mem.read_address((start_address_h, start_address_l - 2).to_full_address());
// /// let written_z1 = mem.read_address((start_address_h, start_address_l - 3).to_full_address());
// ///
// /// assert_eq!(written_x1, 0xCAFE);
// /// assert_eq!(written_y1, 0xEFAC);
// /// assert_eq!(written_z1, 0xBEEF);
// /// let (_, final_stack_address_low) = registers.get_segmented_sp();
// /// assert_eq!(final_stack_address_low, start_address_l - 3);
// ///
// /// ```
// ///
// impl Executor for StoreManyRegisterFromAddressRegisterData {
//     // ID: 0x28
//     fn execute(
//         &self,
//         registers: &mut Registers,
//         mem: &MemoryPeripheral,
//         stage: &ExecutionStage,
//         intermediateRegisters: &IntermediateRegisters,
//     ) {
//         let start_index = self.data.r1;
//         let end_index = self.data.r2;
//         let address_register_index = self.data.r3;
//         let immediate_offset = sign_extend_small_offset(self.data.additional_flags);
//         let offset_operator = u16::wrapping_sub;

//         // TODO: Work out what would happen on the hardware here
//         // Probably something undefined, depending on how the logic is written
//         // I doubt it would be worth putting validation in but I'll see
//         assert!(is_valid_register_range(start_index, end_index));

//         let mut write_address = 0x0;
//         for register_index in start_index..=end_index {
//             let incremental_offset = register_index - start_index;
//             write_address = calculate_effective_address_with_immediate(
//                 registers,
//                 address_register_index,
//                 offset_operator(immediate_offset, incremental_offset as u16),
//             );
//             let write_value = registers.get_at_index(register_index);
//             mem.write_address(write_address, write_value);
//         }

//         // Only update the address register at the end, otherwise if you are storing the same
//         // address register the stored value will be undefined.
//         // TODO: Is that too complicated? Should we just make the programmer modify the address register?
//         // TODO: I think we probably want to ban doing a many load/store on the address registers as it could lead to all sorts of complexities
//         registers.set_address_register_at_index(address_register_index, write_address);
//     }
// }
