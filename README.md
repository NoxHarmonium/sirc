# SIRC - Super Imaginary Retro Console

The best retro console that never existed

[![codecov](https://codecov.io/gh/NoxHarmonium/sirc/branch/main/graph/badge.svg?token=8VC7KHXLBI)](https://codecov.io/gh/NoxHarmonium/sirc)

# Summary

This is a project to develop a retro video game console from scratch that never existed.

It has been a dream of mine for a long time to build a console from the ground up,
from designing the CPU instruction set and PPU architecture up to even the assembler/linker
toolchain used to build the games.

Using the power of FPGA the hardware components required _should_ be able to be synthesised
so that the final product is a physical game console.

However, there is a long way to go until that point.

I built a basic CPU using an FPGA board in university about ten years ago so it could be possible...

# Target

The console will be a "fourth generation" console and will be roughly targeting the power/graphics quality of a [SNES](https://en.wikipedia.org/wiki/Super_Nintendo_Entertainment_System).

It should have about 128 KB of general-purpose RAM and 64 KB VRAM but this could be slightly flexible to allow differences
in system design.

The target FPGA board at the moment is the [ULX3s](https://www.crowdsupply.com/radiona/ulx3s).
This could change at some point so the design will try to be dev board independent but
it at least provides a target to hit.

## CPU

The CPU will be basic (e.g. no superscalar) but it does not need to be as simple as the 6502,
and can take inspiration from other CPUs available around that time (68k, ARM6, x86).

It will be strictly 16 bit. One exception is that memory addresses can be 24 bit by synthesising the address by from two 16 bit registers,
as a 16 bit address space is too limiting (even the SNES has 24 bit addressing).

Instructions are strictly 32 bit for instruction decoder simplicity reasons.

The current high level design goals are:

### RISC

Prioritise a design that is simple and fast to execute, over complex instructions.

The advantages here are:

- Simpler CPU design (easier for me define in Verilog)
- Allows for performance improvements in future revisions

The disadvantages:

- Poor ergonomics for developers writing straight assembler code without a compiler
- Larger binary sizes (could be an issue with the limited RAM/ROM available at the target generation)
- More instruction fetches (2x16 bit fetches for each instruction)

### Load/Store Architecture

Simplifies the CPU design by restricting the ALU to only operating on registers.
This is taken to the extreme by not even providing any immediate addressing modes for arithmetic/logic instructions.

A special pair of instructions will be provided for loading/storing multiple registers at once.
This increases the speed that data can get in/out of the CPU and will probably avoid the need for a DMA controller.

The CPU will be designed on the assumption that the connected RAM module will support bursting
and that memory reads can be pipelined (e.g. reading n words takes 1+n cycles)

# Project Status

The many stages to building this thing:

1. Design/Simulate the CPU

- [x] Define an instruction set (SIRCIS)
- [x] Build a virtual machine
- [x] Write a basic assembler/linker
- [x] Run some basic programs
- [ ] Write and run an extensive test program to shake out any implementation bugs
- [ ] (Optional) Write an LLVM backend to write programs in C (or even rust???)
- [ ] (Optional) Write a language server and asm regex for IDE support (and debugging??)
- [ ] Write a real world program that interacts with the outside world via memory mapping (a raytracer)

2. Design/Simulate Input

- [ ] Design controller input and map to keyboard buttons
- [ ] (Optional) Add I/O window with status LEDs to aid with debugging

3. Design/Simulate the PPU

- [ ] Design the PPU architecture (based mainly on the SNES)
- [ ] Design the bus that connects the CPU/PPU and the clock ratios
- [ ] Write a simulator for the PPU that renders to a window
- [ ] Write a basic game that tests the PPU (e.g. tetris?)

4. Design/Simulate the APU

- [ ] Design the APU architecture (sample based)
- [ ] Write the simulator for the APU
- [ ] Update the basic game to use the APU

5. Build the FPGA system

- [ ] Implement Verilog based on ULX3S FPGA board
- [ ] Run the basic game on real hardware!
