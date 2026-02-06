# Makes the "faults" example binary file to be executed by the VM

# --no-default-features disables the video device
CARGO_ARGS=--manifest-path=../../sirc-vm/Cargo.toml
RUN_ARGS=-vv --program-file ./faults.bin --segment SCRATCH:00010000:FFFF --segment FAULTS_HIGH:00020000:FFFF:./faults-high.bin --register-dump-file ./faults.register-dump

all: faults.bin

faults.o: faults.asm
	cargo run ${CARGO_ARGS} --no-default-features --bin assembler -- --input-file faults.asm --output-file faults.o

faults-high.o: faults-high.asm
	cargo run ${CARGO_ARGS} --no-default-features --bin assembler -- --input-file faults-high.asm --output-file faults-high.o

faults.bin: faults.o
	cargo run ${CARGO_ARGS} --no-default-features --bin linker -- --segment-offset 0 --output-file faults.bin faults.o

# TODO: Maybe this would be a good oportunity to test linking different binaries, but segments can only be 65535 words long
faults-high.bin: faults-high.o
	cargo run ${CARGO_ARGS} --no-default-features --bin linker -- --segment-offset 0 --output-file faults-high.bin faults-high.o

run: faults.bin faults-high.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS}

debug: faults.bin faults-high.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --debug

check: run
	diff -u ./faults.register-dump ./faults.register-dump-expected

clean:
	rm -f faults.bin faults.o faults-high.bin faults-high.o faults.register-dump faults.bin.dbg faults-high.bin.dbg

clean_all: clean
	cargo clean ${CARGO_ARGS}
	cargo llvm-cov clean ${CARGO_ARGS} --workspace