# Makes the "boring" example binary file to be executed by the VM

# --no-default-features disables the video device
CARGO_ARGS=--manifest-path=../../sirc-vm/Cargo.toml
RUN_ARGS=-vv --program-file ./boring.bin --register-dump-file ./boring.register-dump

all: boring.bin

boring.o: boring.asm
	cargo run ${CARGO_ARGS} --no-default-features --bin assembler -- --input-file boring.asm --output-file boring.o

boring.bin: boring.o
	cargo run ${CARGO_ARGS} --no-default-features --bin linker -- --segment-offset 0 --output-file boring.bin boring.o

run: boring.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS}

debug: boring.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --debug

check: run
	diff -u ./boring.register-dump ./boring.register-dump-expected

clean:
	rm -f boring.bin boring.o boring.register-dump boring.bin.dbg

clean_all: clean
	cargo clean ${CARGO_ARGS}
	cargo llvm-cov clean ${CARGO_ARGS} --workspace