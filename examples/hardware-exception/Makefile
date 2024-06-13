# Makes the "hardware-exception" example binary file to be executed by the VM

CARGO_ARGS="--manifest-path=../../sirc-vm/Cargo.toml"
RUN_ARGS=-vv --program-file ./hardware-exception.bin --register-dump-file ./hardware-exception.register-dump --segment SCRATCH:00010000:FFFF

all: hardware-exception.bin

hardware-exception.o: hardware-exception.asm
	cargo run ${CARGO_ARGS} --bin assembler -- --input-file hardware-exception.asm --output-file hardware-exception.o

data.o: data.asm
	cargo run ${CARGO_ARGS} --bin assembler -- --input-file data.asm --output-file data.o

hardware-exception.bin: hardware-exception.o data.o
	cargo run ${CARGO_ARGS} --bin linker -- --segment-offset 0 --output-file hardware-exception.bin hardware-exception.o data.o

run: hardware-exception.bin
	echo "test input that should end with the letter a" | cargo run ${CARGO_ARGS} --bin sbrc_vm -- ${RUN_ARGS} > output.log

debug: hardware-exception.bin
	echo "test input that should end with the letter a" | cargo run ${CARGO_ARGS} --bin sbrc_vm -- ${RUN_ARGS} --debug > output.log

check: run
	diff -u ./output.log ./output.log.expected
	diff -u ./hardware-exception.register-dump ./hardware-exception.register-dump-expected

clean:
	rm -f hardware-exception.bin hardware-exception.o data.o hardware-exception.register-dump output.log hardware-exception.bin.dbg
	cargo clean ${CARGO_ARGS}