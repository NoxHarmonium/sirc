# Builds the integer maths coprocessor emulation example.

# --no-default-features disables the video device.
CARGO_ARGS=--manifest-path=../../sirc-vm/Cargo.toml
RUN_ARGS=--program-file ./math-coprocessor-emulation.bin --segment SCRATCH:00010000:10000 --register-dump-file ./math-coprocessor-emulation.register-dump

all: math-coprocessor-emulation.bin

math-coprocessor-emulation.o: math-coprocessor-emulation.sasm
	cargo run ${CARGO_ARGS} --no-default-features --bin assembler -- --input-file math-coprocessor-emulation.sasm --output-file math-coprocessor-emulation.o

math-coprocessor-emulation.bin: math-coprocessor-emulation.o
	cargo run ${CARGO_ARGS} --no-default-features --bin linker -- --segment-offset 0 --output-file math-coprocessor-emulation.bin math-coprocessor-emulation.o

run: math-coprocessor-emulation.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS}

debug: math-coprocessor-emulation.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --debug

check: run
	diff -u ./math-coprocessor-emulation.register-dump ./math-coprocessor-emulation.register-dump-expected

clean:
	rm -f math-coprocessor-emulation.bin math-coprocessor-emulation.o math-coprocessor-emulation.register-dump math-coprocessor-emulation.bin.dbg

clean_all: clean
	cargo clean ${CARGO_ARGS}
	cargo llvm-cov clean ${CARGO_ARGS} --workspace
