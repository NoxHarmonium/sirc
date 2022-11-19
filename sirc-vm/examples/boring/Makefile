# Makes the "boring" example binary file to be executed by the VM

all: boring.bin

boring.o: boring.asm
	cargo run --bin assembler -- --input-file boring.asm --output-file boring.o

boring.bin: boring.o
	cargo run --bin linker -- --segment-offset 256 --output-file boring.bin boring.o

run: boring.bin
	cargo run --bin sbrc_vm -- --program-file ./boring.bin

clean:
	rm -f boring.bin boring.o