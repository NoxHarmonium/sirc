# Makes the "hardware-exception" example binary file to be executed by the VM

all: hardware-exception.bin

hardware-exception.o: hardware-exception.asm
	cargo run --bin assembler -- --input-file hardware-exception.asm --output-file hardware-exception.o

hardware-exception.bin: hardware-exception.o
	cargo run --bin linker -- --segment-offset 0 --output-file hardware-exception.bin hardware-exception.o

run: hardware-exception.bin
	echo "test input that should end with the letter a" | cargo run --bin sbrc_vm -- -vvv --program-file ./hardware-exception.bin --register-dump-file ./hardware-exception.register-dump --segment SCRATCH:00010000:FFFF > output.log

check: run
	diff -u ./output.log ./output.log.expected
	diff -u ./hardware-exception.register-dump ./hardware-exception.register-dump-expected

clean:
	rm -f hardware-exception.bin hardware-exception.o hardware-exception.register-dump output.log
	cargo clean