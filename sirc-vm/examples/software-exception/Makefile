# Makes the "software-exception" example binary file to be executed by the VM

all: software-exception.bin

software-exception.o: software-exception.asm
	cargo run --bin assembler -- --input-file software-exception.asm --output-file software-exception.o

software-exception.bin: software-exception.o
	cargo run --bin linker -- --segment-offset 0 --output-file software-exception.bin software-exception.o

run: software-exception.bin
	cargo run --bin sbrc_vm -- -vv --program-file ./software-exception.bin --register-dump-file ./software-exception.register-dump

check: run
	diff -u ./software-exception.register-dump ./software-exception.register-dump-expected

clean:
	rm -f software-exception.bin software-exception.o software-exception.register-dump
	cargo clean