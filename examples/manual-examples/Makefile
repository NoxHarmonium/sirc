# Assembles every SIRC-1 code listing pulled out of the reference manual
# (docs/reference/chapters/*.tex), to catch syntax the manual claims is
# valid but that the real assembler rejects. See extract-listings.awk for
# exactly which listings get pulled out, and docs/reference/README.md's QA
# section for how this fits into the wider manual QA pass.
#
# Two-step, mirroring the assemble step every other examples/*/Makefile
# already uses:
#   1. `extract` -- awk pulls listings out of the manual into extracted/
#      (gitignored), one .sasm file per listing, only rewriting a file when
#      its content actually changed so unrelated files keep their mtime.
#   2. `objects`  -- an ordinary pattern rule assembles each extracted/*.sasm
#      to extracted/*.o, so `make` only re-assembles listings that actually
#      changed since the last run.

CARGO_ARGS := --manifest-path=../../sirc-vm/Cargo.toml
ASSEMBLE := cargo run $(CARGO_ARGS) --no-default-features --bin assembler --quiet --

CHAPTERS_DIR := ../../docs/reference/chapters
CHAPTER_TEX := $(wildcard $(CHAPTERS_DIR)/*.tex)
OUTDIR := extracted

.PHONY: all extract objects _objects clean

all: objects

extract: .extracted.stamp

.extracted.stamp: extract-listings.awk $(CHAPTER_TEX)
	@mkdir -p $(OUTDIR)
	@awk -v outdir=$(OUTDIR) -f extract-listings.awk $(CHAPTER_TEX) | sort > .extracted.manifest
	@ls $(OUTDIR)/*.sasm 2>/dev/null | sort > .extracted.manifest.prev || true
	@comm -23 .extracted.manifest.prev .extracted.manifest | xargs rm -f
	@echo "✓ extracted $$(wc -l < .extracted.manifest | tr -d ' ') listing(s) from $(words $(CHAPTER_TEX)) chapter file(s)"
	@touch $@

# Ordinary pattern rule: make only re-assembles a listing whose extracted
# .sasm is newer than its .o (or has no .o yet).
$(OUTDIR)/%.o: $(OUTDIR)/%.sasm
	$(ASSEMBLE) --input-file $< --output-file $@

# The .sasm file list doesn't exist until `extract` has actually run, so the
# object list below has to be computed by a fresh make invocation (the
# standard fix for "generated file list needed in the same run").
objects: extract
	@$(MAKE) --no-print-directory _objects

_objects: $(patsubst $(OUTDIR)/%.sasm,$(OUTDIR)/%.o,$(wildcard $(OUTDIR)/*.sasm))
	@echo "✓ $(words $^) example listing(s) assemble cleanly"

clean:
	rm -rf $(OUTDIR) .extracted.stamp .extracted.manifest .extracted.manifest.prev
