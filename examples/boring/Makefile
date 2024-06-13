# Makes the "boring" example binary file to be executed by the VM

CARGO_ARGS="--manifest-path=../../sirc-vm/Cargo.toml"

all: boring.bin

boring.o: boring.asm
	cargo run ${CARGO_ARGS} --bin assembler -- --input-file boring.asm --output-file boring.o

boring.bin: boring.o
	cargo run ${CARGO_ARGS} --bin linker -- --segment-offset 0 --output-file boring.bin boring.o

run: boring.bin
	cargo run ${CARGO_ARGS} --bin sbrc_vm -- -vv --program-file ./boring.bin --register-dump-file ./boring.register-dump

check: run
	diff -u ./boring.register-dump ./boring.register-dump-expected

clean:
	rm -f boring.bin boring.o boring.register-dump boring.bin.dbg
	cargo clean ${CARGO_ARGS}