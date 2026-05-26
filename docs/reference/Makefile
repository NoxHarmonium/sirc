# Makefile for SIRC-1 CPU Reference Manual

# Main document
MAIN = main
OUTPUT = SIRC-1-reference-manual

# LaTeX compiler
LATEX = pdflatex
LATEX_FLAGS = -interaction=nonstopmode -halt-on-error

ROOT_DIR := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))
TEX := $(MAIN).tex preamble.tex $(wildcard chapters/*.tex)
DOT := $(wildcard diagrams/*.dot)
DIAGRAM_PDF := $(DOT:.dot=.pdf)

.PHONY: all clean cleanall quick graphviz view format format-check

# Default target - build the PDF
all: $(OUTPUT).pdf

# Build the PDF (multiple passes for cross-references)
$(OUTPUT).pdf: graphviz $(TEX)
	@echo "Building PDF (pass 1/3)..."
	@$(LATEX) $(LATEX_FLAGS) $(MAIN).tex > /dev/null || (cat $(MAIN).log && exit 1)
	@echo "Building PDF (pass 2/3)..."
	@$(LATEX) $(LATEX_FLAGS) $(MAIN).tex > /dev/null || (cat $(MAIN).log && exit 1)
	@echo "Building PDF (pass 3/3)..."
	@$(LATEX) $(LATEX_FLAGS) $(MAIN).tex > /dev/null || (cat $(MAIN).log && exit 1)
	@mv $(MAIN).pdf $(OUTPUT).pdf
	@echo "✓ PDF generated: $(OUTPUT).pdf"

# Quick build (single pass, for drafts)
quick: graphviz $(TEX)
	@echo "Quick build (single pass)..."
	@$(LATEX) $(LATEX_FLAGS) $(MAIN).tex
	@mv $(MAIN).pdf $(OUTPUT).pdf
	@echo "✓ Quick PDF generated: $(OUTPUT).pdf"

graphviz: $(DIAGRAM_PDF)

diagrams/%.pdf: diagrams/%.dot
	@echo "Rendering diagram: $<"
	@if command -v dot >/dev/null 2>&1; then \
		dot -Tpdf "$<" -o "$@"; \
	else \
		echo "Error: Graphviz 'dot' is required to regenerate $@"; \
		exit 1; \
	fi

# Clean temporary files
clean:
	@echo "Cleaning temporary files..."
	@rm -f *.aux *.log *.toc *.out *.lof *.lot *.fls *.fdb_latexmk *.synctex.gz
	@rm -f chapters/*.aux
	@echo "✓ Cleaned"

# Clean everything including PDF
cleanall: clean
	@echo "Removing PDF..."
	@rm -f $(OUTPUT).pdf $(MAIN).pdf
	@echo "✓ All clean"


format: .format.stamp

.format.stamp: $(TEX)
	latexindent -m -l  --cruft=.cruft/ --overwriteIfDifferent --silent $? || (cat .cruft/indent.log && exit 1)


format-check: .format-check.stamp

.format-check.stamp: $(MAIN).tex preamble.tex chapters/*.tex
	latexindent -m -l --cruft=.cruft/ --silent --check $? || (cat .cruft/indent.log && exit 1)
