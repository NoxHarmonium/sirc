# Makes the "store-load" example binary file to be executed by the VM

all: store-load.bin

store-load.o: store-load.asm
	cargo run --bin assembler -- --input-file store-load.asm --output-file store-load.o

store-load.bin: store-load.o
	cargo run --bin linker -- --segment-offset 0 --output-file store-load.bin store-load.o

run: store-load.bin
	cargo run --bin sbrc_vm -- --program-file ./store-load.bin

clean:
	rm -f store-load.bin store-load.o
	cargo clean