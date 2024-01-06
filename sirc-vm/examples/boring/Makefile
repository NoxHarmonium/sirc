# Makes the "boring" example binary file to be executed by the VM

all: boring.bin

boring.o: boring.asm
	cargo run --bin assembler -- --input-file boring.asm --output-file boring.o

boring.bin: boring.o
	cargo run --bin linker -- --segment-offset 0 --output-file boring.bin boring.o

run: boring.bin
	cargo run --bin sbrc_vm -- --program-file ./boring.bin --register-dump-file ./software-exception.register-dump

check: run
	diff -u ./software-exception.register-dump ./software-exception.register-dump-expected

clean:
	rm -f boring.bin boring.o software-exception.register-dump
	cargo clean