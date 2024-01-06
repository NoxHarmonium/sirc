# Makes the "store-load" example binary file to be executed by the VM

all: store-load.bin

store-load.o: store-load.asm
	cargo run --bin assembler -- --input-file store-load.asm --output-file store-load.o

store-load.bin: store-load.o
	cargo run --bin linker -- --segment-offset 0 --output-file store-load.bin store-load.o

run: store-load.bin
	cargo run --bin sbrc_vm -- --program-file ./store-load.bin --segment scratch:00080000:FFFF  --register-dump-file ./software-exception.register-dump

check: run
	diff -u ./software-exception.register-dump ./software-exception.register-dump-expected

clean:
	rm -f store-load.bin store-load.o software-exception.register-dump
	cargo clean