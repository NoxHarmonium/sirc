# Builds a small binary used as the source of truth for manual encoding examples.
#
# This intentionally does not run the VM or compare a register dump. The check target
# only verifies that every annotated manual example produced one 32-bit instruction.

CARGO_ARGS=--manifest-path=../../sirc-vm/Cargo.toml

PROGRAM=reference-encodings
SOURCE=$(PROGRAM).sasm
OBJECT=$(PROGRAM).o
BINARY=$(PROGRAM).bin
HEX=$(PROGRAM).hex
MANUAL_TEX_DIR=../../docs/reference/generated
MANUAL_TEX_AWK=manual-tex.awk
EXAMPLES=$(PROGRAM).examples.tmp
WORDS=$(PROGRAM).words.tmp
SECTIONS=$(PROGRAM).sections.tmp

all: $(BINARY) $(HEX)

$(OBJECT): $(SOURCE)
	cargo run $(CARGO_ARGS) --no-default-features --bin assembler -- --input-file $(SOURCE) --output-file $(OBJECT)

$(BINARY): $(OBJECT)
	cargo run $(CARGO_ARGS) --no-default-features --bin linker -- --segment-offset 0 --output-file $(BINARY) $(OBJECT)

$(HEX): $(BINARY) $(SOURCE)
	grep '^; @manual-example ' $(SOURCE) | sed 's/^; @manual-example //' > $(EXAMPLES)
	od -An -tx1 -v $(BINARY) | awk '{ for (i = 1; i <= NF; i += 4) if ((i + 3) <= NF) print "0x" toupper($$i $$(i + 1) $$(i + 2) $$(i + 3)); }' > $(WORDS)
	paste $(EXAMPLES) $(WORDS) | awk -F '\t' '{ printf "%-48s %s\n", $$1, $$2 }' > $(HEX)

manual-tex: $(BINARY) $(SOURCE)
	@mkdir -p $(MANUAL_TEX_DIR)
	rm -f $(MANUAL_TEX_DIR)/*-format-encodings.tex
	grep -E '^; @(manual-section|manual-example) ' $(SOURCE) | sed 's/^; @//' > $(SECTIONS)
	od -An -tx1 -v $(BINARY) | awk '{ for (i = 1; i <= NF; i += 4) if ((i + 3) <= NF) print "0x" toupper($$i $$(i + 1) $$(i + 2) $$(i + 3)); }' > $(WORDS)
	awk -v word_file="$(WORDS)" -v out_dir="$(MANUAL_TEX_DIR)" -f $(MANUAL_TEX_AWK) $(SECTIONS)

check: all
	@examples=$$(grep -c '^; @manual-example ' $(SOURCE)); \
	words=$$(wc -l < $(WORDS)); \
	test "$$examples" -eq "$$words" || { echo "Expected $$examples encoded words, got $$words"; exit 1; }

clean:
	rm -f $(OBJECT) $(BINARY) $(BINARY).dbg $(HEX) $(EXAMPLES) $(WORDS) $(SECTIONS)

clean_all: clean
	cargo clean $(CARGO_ARGS)
	cargo llvm-cov clean $(CARGO_ARGS) --workspace

.PHONY: all manual-tex check clean clean_all
