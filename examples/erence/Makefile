# Exports an ERENCE song, links it with a small loader and plays it in the VM.
#
#   make play       assemble, link and run; writes demo.wav of what the device played
#   make check      compare the VM's audio with ERENCE's own render

CARGO_ARGS=--manifest-path=../../sirc-vm/Cargo.toml --release --offline
ERENCE=cargo run ${CARGO_ARGS} --bin erence --
ASM=cargo run ${CARGO_ARGS} --bin assembler --
LINK=cargo run ${CARGO_ARGS} --bin linker --
VM=cargo run ${CARGO_ARGS} --bin sirc_vm --

all: play.bin

# The VM waits for every sequence to finish, so the song must not loop.
demo-once.erence: demo.erence
	sed 's/^loop .*/loop none/' demo.erence > demo-once.erence

demo-once.sasm: demo-once.erence
	${ERENCE} --export demo-once.sasm demo-once.erence

demo-once.o: demo-once.sasm
	${ASM} --input-file demo-once.sasm --output-file demo-once.o

play.o: play.sasm
	${ASM} --input-file play.sasm --output-file play.o

play.bin: play.o demo-once.o
	${LINK} --segment-offset 0 --output-file play.bin play.o demo-once.o

play: play.bin
	${VM} --program-file ./play.bin --segment SCRATCH:00010000:FFFF --audio-wav demo.wav

reference.wav: demo-once.erence
	${ERENCE} --render reference.wav demo-once.erence

check: play reference.wav
	python3 compare.py demo.wav reference.wav

clean:
	rm -f demo-once.* play.o play.bin play.bin.dbg demo.wav reference.wav
