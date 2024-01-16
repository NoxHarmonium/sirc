# Makes the "bus-error" example binary file to be executed by the VM

all: bus-error.bin

bus-error.o: bus-error.asm
	cargo run --bin assembler -- --input-file bus-error.asm --output-file bus-error.o

bus-error.bin: bus-error.o
	cargo run --bin linker -- --segment-offset 0 --output-file bus-error.bin bus-error.o

run: bus-error.bin
	cargo run --bin sbrc_vm -- -vvvv --program-file ./bus-error.bin --register-dump-file ./bus-error.register-dump

check: run
	diff -u ./bus-error.register-dump ./bus-error.register-dump-expected

clean:
	rm -f bus-error.bin bus-error.o bus-error.register-dump
	cargo clean