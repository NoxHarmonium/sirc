# Makes the "faults" example binary file to be executed by the VM

all: faults.bin

faults.o: faults.asm
	cargo run --bin assembler -- --input-file faults.asm --output-file faults.o

faults.bin: faults.o
	cargo run --bin linker -- --segment-offset 0 --output-file faults.bin faults.o

run: faults.bin
	cargo run --bin sbrc_vm -- -vvvv --program-file ./faults.bin --register-dump-file ./faults.register-dump

check: run
	diff -u ./faults.register-dump ./faults.register-dump-expected

clean:
	rm -f faults.bin faults.o faults.register-dump
	cargo clean