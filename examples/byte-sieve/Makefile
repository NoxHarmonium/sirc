# Makes the "byte-sieve" example binary file to be executed by the VM

# --no-default-features disables the video device
CARGO_ARGS=--manifest-path=../../sirc-vm/Cargo.toml
RUN_ARGS=-vv --program-file ./byte-sieve.bin --segment file:00F00000:FFFF:./mem.bin

all: byte-sieve.bin


byte-sieve.o: byte-sieve.asm
	cargo run ${CARGO_ARGS} --no-default-features --bin assembler -- --input-file byte-sieve.asm --output-file byte-sieve.o

byte-sieve.bin: byte-sieve.o
	cargo run ${CARGO_ARGS} --no-default-features --bin linker -- --segment-offset 0 --output-file byte-sieve.bin byte-sieve.o

run: byte-sieve.bin
	rm -f ./mem.bin
	truncate -s 131070 ./mem.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS}

debug: byte-sieve.bin
	rm -f ./mem.bin
	truncate -s 131070 ./mem.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --debug

check: run expected.hex
	hexdump -C  ./mem.bin > actual.hex
	diff expected.hex actual.hex

clean:
	rm -f byte-sieve.bin byte-sieve.o actual.hex mem.bin byte-sieve.bin.dbg

clean_all: clean
	cargo clean ${CARGO_ARGS}
	cargo llvm-cov clean ${CARGO_ARGS} --workspace