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

_Note: It is my first serious project in rust so it will be a bit rough, I'm also trying to get a vertical slice going before I optimise everything. Please don't judge me._

I built a basic CPU using an FPGA board in university about ten years ago so it could be possible...

# Target

The console will be a "fourth generation" console and will be roughly targeting the power/graphics quality of a [SNES](https://en.wikipedia.org/wiki/Super_Nintendo_Entertainment_System).

It should have about 128 KB of general-purpose RAM and 64 KB VRAM but this could be slightly flexible to allow differences
in system design.

The target FPGA board at the moment is the [ULX3s](https://www.crowdsupply.com/radiona/ulx3s).
This could change at some point so the design will try to be dev board independent but
it at least provides a target to hit.

# Usage

A good example of all the components working together is the Makefile in the [byte-sieve example project](https://github.com/NoxHarmonium/sirc/blob/main/sirc-vm/examples/byte-sieve/Makefile).

It involves the assembler, linker and the virtual machine.

You can also use the `--help` command line switches for each component.

```bash
$ cargo run --bin assembler -- --help

Usage: assembler --input-file <FILE> --output-file <FILE>

Options:
  -i, --input-file <FILE>
  -o, --output-file <FILE>
  -h, --help                Print help
  -V, --version             Print version
```

```bash
$ cargo run --bin linker -- --help

Usage: linker --output-file <FILE> --segment-offset <SEGMENT_OFFSET> [INPUT FILES]...

Arguments:
  [INPUT FILES]...

Options:
  -o, --output-file <FILE>
  -s, --segment-offset <SEGMENT_OFFSET>
  -h, --help                             Print help
  -V, --version                          Print version
```

```bash
$ cargo run --bin sbrc_vm -- --help

Usage: sbrc_vm [OPTIONS] --program-file <FILE>

Options:
  -p, --program-file <FILE>
  -s, --segment <SEGMENT>
  -r, --register-dump-file <FILE>
  -v, --verbose...                 Increase logging verbosity
  -q, --quiet...                   Decrease logging verbosity
  -e, --enable-video
  -d, --debug
  -h, --help                       Print help
  -V, --version                    Print version

```

## CPU

See the wiki for information on the CPU and PPU design!

https://github.com/NoxHarmonium/sirc/wiki

# Project Status

The many stages to building this thing:

1. Design/Simulate the CPU

- [x] Define an instruction set (SIRCIS)
- [x] Build a virtual machine
- [x] Write a basic assembler/linker
- [x] Run some basic programs
- [x] Add a debugger to allow stepping through programs
- [x] Write and run an extensive 'real world' test program to shake out any implementation bugs
- [x] Write a test unit suite to test each instruction to allow to do some serious refactoring for performance (and also a benchmark)
- [ ] Optimise the simulator code to make it usable (or add a "low accuracy" mode)
- [ ] Document the CPU architecture and instruction set in a proper reference manual
- [ ] (Optional) Write an LLVM backend to write programs in C (or even rust???)
- [ ] (Optional) Write a language server and asm regex for IDE support (and debugging??)

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
