# Makes the "software-exception" example binary file to be executed by the VM

CARGO_ARGS="--manifest-path=../../sirc-vm/Cargo.toml"
RUN_ARGS=--program-file ./software-exception.bin --register-dump-file ./software-exception.register-dump

all: software-exception.bin

software-exception.o: software-exception.asm
	cargo run ${CARGO_ARGS} --bin assembler -- --input-file software-exception.asm --output-file software-exception.o

software-exception.bin: software-exception.o
	cargo run ${CARGO_ARGS} --bin linker -- --segment-offset 0 --output-file software-exception.bin software-exception.o

run: software-exception.bin
	cargo run ${CARGO_ARGS} --bin sbrc_vm -- ${RUN_ARGS}

debug: software-exception.bin
	cargo run ${CARGO_ARGS} --bin sbrc_vm -- -vv ${RUN_ARGS} --debug

check: run
	diff -u ./software-exception.register-dump ./software-exception.register-dump-expected

clean:
	rm -f software-exception.bin software-exception.o software-exception.register-dump software-exception.bin.dbg
	cargo clean ${CARGO_ARGS}
	cargo llvm-cov clean ${CARGO_ARGS} --workspace