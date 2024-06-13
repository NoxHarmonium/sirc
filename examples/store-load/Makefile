# Makes the "store-load" example binary file to be executed by the VM

CARGO_ARGS="--manifest-path=../../sirc-vm/Cargo.toml"
RUN_ARGS=-vv --program-file ./store-load.bin --segment scratch:00080000:FFFF  --register-dump-file ./store-load.register-dump

all: store-load.bin

store-load.o: store-load.asm
	cargo run ${CARGO_ARGS} --bin assembler -- --input-file store-load.asm --output-file store-load.o

store-load.bin: store-load.o
	cargo run ${CARGO_ARGS} --bin linker -- --segment-offset 0 --output-file store-load.bin store-load.o

run: store-load.bin
	cargo run ${CARGO_ARGS} --bin sbrc_vm -- ${RUN_ARGS}

debug: store-load.bin
	cargo run ${CARGO_ARGS} --bin sbrc_vm -- ${RUN_ARGS} --debug

check: run
	diff -u ./store-load.register-dump ./store-load.register-dump-expected

clean:
	rm -f store-load.bin store-load.o store-load.register-dump store-load.bin.dbg
	cargo clean ${CARGO_ARGS}