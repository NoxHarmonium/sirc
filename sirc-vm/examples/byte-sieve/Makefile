# Makes the "byte-sieve" example binary file to be executed by the VM



all: byte-sieve.bin

run: mem.bin
check: actual.hex

byte-sieve.o: byte-sieve.asm
	cargo run --bin assembler -- --input-file byte-sieve.asm --output-file byte-sieve.o

byte-sieve.bin: byte-sieve.o
	cargo run --bin linker -- --segment-offset 256 --output-file byte-sieve.bin byte-sieve.o

mem.bin: byte-sieve.bin
	rm -f ./mem.bin
	truncate -s 131070 ./mem.bin
	cargo run --bin sbrc_vm -- --program-file ./byte-sieve.bin --segment file:00F00000:FFFF:./mem.bin

actual.hex: mem.bin expected.hex
	hexdump -C  ./mem.bin > actual.hex
	diff expected.hex actual.hex

clean:
	rm -f byte-sieve.bin byte-sieve.o actual.hex mem.bin