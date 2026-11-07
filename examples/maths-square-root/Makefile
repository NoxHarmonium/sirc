CARGO_ARGS=--manifest-path=../../sirc-vm/Cargo.toml
RUN_ARGS=--program-file ./maths-square-root.bin --segment SCRATCH:00010000:10000

.PHONY: all run check clean clean_all
all: maths-square-root.bin

maths-square-root.o: maths-square-root.sasm
	cargo run ${CARGO_ARGS} --no-default-features --bin assembler -- --input-file $< --output-file $@

maths-square-root.bin: maths-square-root.o
	cargo run ${CARGO_ARGS} --no-default-features --bin linker -- --segment-offset 0 --output-file $@ $<

run: maths-square-root.bin
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --optional-coprocessors math --register-dump-file maths-square-root.hardware.register-dump
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --optional-coprocessors none --register-dump-file maths-square-root.software.register-dump
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --register-dump-file maths-square-root.default.register-dump
	cargo run ${CARGO_ARGS} --no-default-features --bin sirc_vm -- ${RUN_ARGS} --optional-coprocessors dma,math --register-dump-file maths-square-root.combined.register-dump

check: run
	diff -u maths-square-root.hardware.register-dump-expected maths-square-root.hardware.register-dump
	diff -u maths-square-root.software.register-dump-expected maths-square-root.software.register-dump
	diff -u maths-square-root.software.register-dump-expected maths-square-root.default.register-dump
	diff -u maths-square-root.hardware.register-dump-expected maths-square-root.combined.register-dump

clean:
	rm -f maths-square-root.o maths-square-root.bin maths-square-root.bin.dbg *.register-dump

clean_all: clean
	cargo clean ${CARGO_ARGS}
	cargo llvm-cov clean ${CARGO_ARGS} --workspace
