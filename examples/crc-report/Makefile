# Makes the "crc-report" example binary file to be executed by the VM.
#
# Computes a CRC-16/XMODEM checksum over a fixed buffer, converts the result
# to decimal ASCII, and writes the digits into a separate memory-mapped
# "file" segment via a software-exception trap into a supervisor monitor
# routine (the main program runs in protected mode and cannot reach that
# segment directly). See PLAN.md in this directory for the full spec.

# --no-default-features disables the video device
CARGO_ARGS=--manifest-path=../../sirc-vm/Cargo.toml
RUN_ARGS=-vv --program-file ./crc-report.bin --segment report_file:00E00000:FFFF:./report_file.bin --register-dump-file ./crc-report.register-dump

all: crc-report.bin

crc-report.o: crc-report.sasm
	cargo run ${CARGO_ARGS} --no-default-features --bin assembler -- --input-file crc-report.sasm --output-file crc-report.o

monitor.o: monitor.sasm
	cargo run ${CARGO_ARGS} --no-default-features --bin assembler -- --input-file monitor.sasm --output-file monitor.o

crc-report.bin: crc-report.o monitor.o
	cargo run ${CARGO_ARGS} --no-default-features --bin linker -- --segment-offset 0 --output-file crc-report.bin crc-report.o monitor.o

run: crc-report.bin
	rm -f ./report_file.bin
	truncate -s 131070 ./report_file.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS}

debug: crc-report.bin
	rm -f ./report_file.bin
	truncate -s 131070 ./report_file.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --debug

# Prints the first bytes of the report file so a human (or the agent) can eyeball
# the result. This deliberately does NOT diff against a committed expected.hex --
# see PLAN.md for why.
inspect: run
	hexdump -C ./report_file.bin | head -4

clean:
	rm -f crc-report.bin crc-report.o monitor.o crc-report.bin.dbg report_file.bin crc-report.register-dump

clean_all: clean
	cargo clean ${CARGO_ARGS}
	cargo llvm-cov clean ${CARGO_ARGS} --workspace

.PHONY: all run debug inspect clean clean_all
