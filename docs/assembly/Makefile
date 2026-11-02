# Makefile for the SIRC-1 Assembly Programmer's Manual
#
# A sibling of ../reference/Makefile: the same build, format and
# reference checks, without the Graphviz diagrams, plus a `listings` target that
# assembles every code listing in the manual with the real assembler. The LaTeX
# preamble is shared with the architecture manual (../reference/preamble.tex).

MAIN = main
OUTPUT = SIRC-1-assembly-programmers-manual

LATEX = pdflatex
LATEX_FLAGS = -interaction=nonstopmode -halt-on-error

TEX := $(MAIN).tex ../reference/preamble.tex $(wildcard chapters/*.tex)

.PHONY: all clean cleanall quick format format-check check-refs listings qa

all: $(OUTPUT).pdf

$(OUTPUT).pdf: $(TEX)
	@echo "Building PDF (pass 1/3)..."
	@$(LATEX) $(LATEX_FLAGS) $(MAIN).tex > /dev/null || (cat $(MAIN).log && exit 1)
	@echo "Building PDF (pass 2/3)..."
	@$(LATEX) $(LATEX_FLAGS) $(MAIN).tex > /dev/null || (cat $(MAIN).log && exit 1)
	@echo "Building PDF (pass 3/3)..."
	@$(LATEX) $(LATEX_FLAGS) $(MAIN).tex > /dev/null || (cat $(MAIN).log && exit 1)
	@mv $(MAIN).pdf $(OUTPUT).pdf
	@echo "✓ PDF generated: $(OUTPUT).pdf"
	@$(MAKE) --no-print-directory check-refs

# Quick build (single pass, for drafts)
quick: $(TEX)
	@echo "Quick build (single pass)..."
	@$(LATEX) $(LATEX_FLAGS) $(MAIN).tex
	@mv $(MAIN).pdf $(OUTPUT).pdf
	@echo "✓ Quick PDF generated: $(OUTPUT).pdf"

clean:
	@echo "Cleaning temporary files..."
	@rm -f *.aux *.log *.toc *.out *.lof *.lot *.fls *.fdb_latexmk *.synctex.gz
	@rm -f chapters/*.aux
	@echo "✓ Cleaned"

cleanall: clean
	@echo "Removing PDF..."
	@rm -f $(OUTPUT).pdf $(MAIN).pdf
	@echo "✓ All clean"

format: .format.stamp

.format.stamp: $(MAIN).tex chapters/*.tex
	latexindent -m -l --cruft=.cruft/ --overwriteIfDifferent --silent $? || (cat .cruft/indent.log && exit 1)

format-check: .format-check.stamp

.format-check.stamp: $(MAIN).tex chapters/*.tex
	latexindent -m -l --cruft=.cruft/ --silent --check $? || (cat .cruft/indent.log && exit 1)

# Fails if the build log has any reference LaTeX couldn't resolve. Run
# automatically as the last step of `all`; needs main.log from a full build.
check-refs:
	@test -f $(MAIN).log || (echo "✗ No $(MAIN).log -- run 'make all' first" && exit 1)
	@if grep -q "LaTeX Warning: There were undefined references." $(MAIN).log; then \
		grep -A1 "LaTeX Warning: Reference \`\|LaTeX Warning: Citation \`" $(MAIN).log; \
		echo "✗ Undefined references/citations found"; \
		exit 1; \
	fi
	@echo "✓ No undefined references or citations"

# Assembles every listing marked `% qa:assemble` in the chapters, using the same
# extractor as the architecture manual's QA. Listings of shell commands and
# diagnostics are not marked and are left alone.
EXTRACTED = extracted
ASSEMBLER = cargo run --manifest-path=../../sirc-vm/Cargo.toml --no-default-features --bin assembler --quiet --

listings:
	@rm -rf $(EXTRACTED) && mkdir -p $(EXTRACTED)
	@awk -v outdir=$(EXTRACTED) -f ../../examples/manual-examples/extract-listings.awk chapters/*.tex
	@count=0; for f in $(EXTRACTED)/*.sasm; do \
		$(ASSEMBLER) --input-file $$f --output-file $${f%.sasm}.o || { echo "✗ $$f does not assemble"; exit 1; }; \
		count=$$((count + 1)); \
	done; echo "✓ $$count listing(s) assemble cleanly"

# Every check this manual has: formatting, a full build with reference check, and the listings.
qa: format-check all listings
