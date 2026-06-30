# Makes the "raytracer" example binary file to be executed by the VM
# Output: render.pgm (PGM P5 grayscale 128x96 image)

CARGO_ARGS=--manifest-path=../../sirc-vm/Cargo.toml

# Segment 0x0002 mapped to render.pgm
# 0xFFFF words = 131070 bytes (header + pixels + stack room)
RUN_ARGS=--program-file ./raytracer.bin --segment file:00020000:FFFF:./render.pgm

all: raytracer.bin

raytracer.o: raytracer.sasm
	cargo run ${CARGO_ARGS} --no-default-features --bin assembler -- --input-file raytracer.sasm --output-file raytracer.o

raytracer.bin: raytracer.o
	cargo run ${CARGO_ARGS} --no-default-features --bin linker -- --segment-offset 0 --output-file raytracer.bin raytracer.o

run: raytracer.bin
	rm -f ./render.pgm
	truncate -s 131070 ./render.pgm
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS}

debug: raytracer.bin
	rm -f ./render.pgm
	truncate -s 131070 ./render.pgm
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --debug

clean:
	rm -f raytracer.bin raytracer.o render.pgm raytracer.bin.dbg

clean_all: clean
	cargo clean ${CARGO_ARGS}
	cargo llvm-cov clean ${CARGO_ARGS} --workspace
