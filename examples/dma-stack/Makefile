CARGO_ARGS=--manifest-path=../../sirc-vm/Cargo.toml
RUN_ARGS=--program-file ./dma-stack.bin --segment SCRATCH:00010000:10000

.PHONY: all run check clean clean_all
all: dma-stack.bin

dma-stack.o: dma-stack.sasm
	cargo run ${CARGO_ARGS} --no-default-features --bin assembler -- --input-file $< --output-file $@

dma-stack.bin: dma-stack.o
	cargo run ${CARGO_ARGS} --no-default-features --bin linker -- --segment-offset 0 --output-file $@ $<

run: dma-stack.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --optional-coprocessors dma --register-dump-file dma-stack.hardware.register-dump
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --optional-coprocessors none --register-dump-file dma-stack.software.register-dump
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --register-dump-file dma-stack.default.register-dump

check: run
	diff -u dma-stack.hardware.register-dump-expected dma-stack.hardware.register-dump
	diff -u dma-stack.software.register-dump-expected dma-stack.software.register-dump
	diff -u dma-stack.software.register-dump-expected dma-stack.default.register-dump

clean:
	rm -f dma-stack.o dma-stack.bin dma-stack.bin.dbg *.register-dump

clean_all: clean
	cargo clean ${CARGO_ARGS}
	cargo llvm-cov clean ${CARGO_ARGS} --workspace
