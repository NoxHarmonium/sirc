# Comprehensive Test Suite for SIRC-1 CPU
# Tests all major instruction categories and edge cases

CARGO_ARGS=--manifest-path=../../sirc-vm/Cargo.toml
RUN_ARGS=-vv --program-file ./comprehensive-test.bin --segment scratch:00010000:FFFF  --segment test_runner_storage:00020000:FFFF --segment stack:00030000:FFFF --register-dump-file ./comprehensive-test.register-dump

all: comprehensive-test.bin

comprehensive-test.o: comprehensive-test.asm
	cargo run ${CARGO_ARGS} --no-default-features --bin assembler -- --input-file comprehensive-test.asm --output-file comprehensive-test.o

serial-handler.o: serial-handler.asm
	cargo run ${CARGO_ARGS} --no-default-features --bin assembler -- --input-file serial-handler.asm --output-file serial-handler.o

comprehensive-test.bin: comprehensive-test.o serial-handler.o
	cargo run ${CARGO_ARGS} --no-default-features --bin linker -- --segment-offset 0 --output-file comprehensive-test.bin comprehensive-test.o serial-handler.o

run: comprehensive-test.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS}

debug: comprehensive-test.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --debug

check: run
	diff -u ./comprehensive-test.register-dump ./comprehensive-test.register-dump-expected

clean:
	rm -f comprehensive-test.bin comprehensive-test.o serial-handler.o comprehensive-test.register-dump comprehensive-test.bin.dbg
	cargo clean ${CARGO_ARGS}
	cargo llvm-cov clean ${CARGO_ARGS} --workspace

.PHONY: all run debug check clean
