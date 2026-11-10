# Makefile for the SIRCADA Reference Manual
#
# A trimmed sibling of ../hardware/Makefile: the same build, format and
# reference checks, without the Graphviz diagrams or the assembly-listing QA
# (this manual has no code listings to assemble). The LaTeX preamble is shared
# with the architecture manual (../reference/preamble.tex).

MAIN = main
OUTPUT = SIRCADA-reference-manual

LATEX = pdflatex
LATEX_FLAGS = -interaction=nonstopmode -halt-on-error

TEX := $(MAIN).tex ../reference/preamble.tex $(wildcard chapters/*.tex)
FIGURES := $(wildcard figures/*.png) ../logos/cover-simple.jpg ../reference/sirc-logo.png

.PHONY: all clean cleanall quick format format-check check-refs qa

all: $(OUTPUT).pdf

$(OUTPUT).pdf: $(TEX) $(FIGURES)
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
quick: $(TEX) $(FIGURES)
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

# Every check this manual has: formatting, then a full build with reference check.
qa: format-check all
