# Builds the protected-mode write-back example.

# --no-default-features disables the video device.
CARGO_ARGS=--manifest-path=../../sirc-vm/Cargo.toml
RUN_ARGS=-vv --program-file ./protected-mode-writeback.bin --segment SCRATCH:00010000:10000 --register-dump-file ./protected-mode-writeback.register-dump

all: protected-mode-writeback.bin

protected-mode-writeback.o: protected-mode-writeback.sasm
	cargo run ${CARGO_ARGS} --no-default-features --bin assembler -- --input-file protected-mode-writeback.sasm --output-file protected-mode-writeback.o

protected-mode-writeback.bin: protected-mode-writeback.o
	cargo run ${CARGO_ARGS} --no-default-features --bin linker -- --segment-offset 0 --output-file protected-mode-writeback.bin protected-mode-writeback.o

run: protected-mode-writeback.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS}

debug: protected-mode-writeback.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --debug

check: run
	diff -u ./protected-mode-writeback.register-dump ./protected-mode-writeback.register-dump-expected

clean:
	rm -f protected-mode-writeback.bin protected-mode-writeback.o protected-mode-writeback.register-dump protected-mode-writeback.bin.dbg

clean_all: clean
	cargo clean ${CARGO_ARGS}
	cargo llvm-cov clean ${CARGO_ARGS} --workspace
