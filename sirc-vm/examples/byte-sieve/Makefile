# Makes the "byte-sieve" example binary file to be executed by the VM

all: byte-sieve.bin

byte-sieve.o: byte-sieve.asm
	cargo run --bin assembler -- --input-file byte-sieve.asm --output-file byte-sieve.o

byte-sieve.bin: byte-sieve.o
	cargo run --bin linker -- --segment-offset 256 --output-file byte-sieve.bin byte-sieve.o

run: byte-sieve.bin
	cargo run --bin sbrc_vm -- --program-file ./byte-sieve.bin

clean:
	rm -f byte-sieve.bin byte-sieve.o