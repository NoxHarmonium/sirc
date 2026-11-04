CARGO_ARGS=--manifest-path=../../sirc-vm/Cargo.toml
RUN_ARGS=--program-file ./dma-copy.bin --segment SCRATCH:00010000:10000

.PHONY: all run check clean clean_all
all: dma-copy.bin

dma-copy.o: dma-copy.sasm
	cargo run ${CARGO_ARGS} --no-default-features --bin assembler -- --input-file $< --output-file $@

dma-copy.bin: dma-copy.o
	cargo run ${CARGO_ARGS} --no-default-features --bin linker -- --segment-offset 0 --output-file $@ $<

run: dma-copy.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --optional-coprocessors dma --register-dump-file dma-copy.hardware.register-dump
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --optional-coprocessors none --register-dump-file dma-copy.software.register-dump
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --register-dump-file dma-copy.default.register-dump

check: run
	diff -u dma-copy.hardware.register-dump-expected dma-copy.hardware.register-dump
	diff -u dma-copy.software.register-dump-expected dma-copy.software.register-dump
	diff -u dma-copy.software.register-dump-expected dma-copy.default.register-dump

clean:
	rm -f dma-copy.o dma-copy.bin dma-copy.bin.dbg *.register-dump

clean_all: clean
	cargo clean ${CARGO_ARGS}
	cargo llvm-cov clean ${CARGO_ARGS} --workspace
