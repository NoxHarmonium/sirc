# Makes the "basic-video" example binary file to be executed by the VM

CARGO_ARGS="--manifest-path=../../sirc-vm/Cargo.toml"
RUN_ARGS=-vv --program-file ./basic-video.bin --register-dump-file ./basic-video.register-dump --segment SCRATCH:00010000:FFFF --enable-video

all: basic-video.bin

serial-handler.o: serial-handler.asm
	cargo run ${CARGO_ARGS} --bin assembler -- --input-file serial-handler.asm --output-file serial-handler.o

basic-video.o: basic-video.asm
	cargo run ${CARGO_ARGS} --bin assembler -- --input-file basic-video.asm --output-file basic-video.o

basic-video.bin: basic-video.o serial-handler.o
	cargo run ${CARGO_ARGS} --bin linker -- --segment-offset 0 --output-file basic-video.bin basic-video.o serial-handler.o

run: basic-video.bin
	cargo run ${CARGO_ARGS} --bin sbrc_vm --release -- ${RUN_ARGS}

debug: basic-video.bin
	cargo run ${CARGO_ARGS} --bin sbrc_vm -- ${RUN_ARGS} --debug

check: run
	diff -u ./basic-video.register-dump ./basic-video.register-dump-expected

clean:
	rm -f basic-video.bin basic-video.o serial-handler.o basic-video.register-dump basic-video.bin.dbg
	cargo clean ${CARGO_ARGS}