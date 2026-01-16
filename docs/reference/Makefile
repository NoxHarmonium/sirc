# Makefile for SIRCULAR CPU Reference Manual

# Main document
MAIN = main
OUTPUT = sircular-reference-manual

# LaTeX compiler
LATEX = pdflatex
LATEX_FLAGS = -interaction=nonstopmode -halt-on-error

# Output directory (optional)
# OUTDIR = build

.PHONY: all clean cleanall quick view

# Default target - build the PDF
all: $(OUTPUT).pdf

# Build the PDF (multiple passes for cross-references)
$(OUTPUT).pdf: $(MAIN).tex preamble.tex chapters/*.tex
	@echo "Building PDF (pass 1/3)..."
	@$(LATEX) $(LATEX_FLAGS) $(MAIN).tex > /dev/null || (cat $(MAIN).log && exit 1)
	@echo "Building PDF (pass 2/3)..."
	@$(LATEX) $(LATEX_FLAGS) $(MAIN).tex > /dev/null || (cat $(MAIN).log && exit 1)
	@echo "Building PDF (pass 3/3)..."
	@$(LATEX) $(LATEX_FLAGS) $(MAIN).tex > /dev/null || (cat $(MAIN).log && exit 1)
	@mv $(MAIN).pdf $(OUTPUT).pdf
	@echo "✓ PDF generated: $(OUTPUT).pdf"

# Quick build (single pass, for drafts)
quick: $(MAIN).tex
	@echo "Quick build (single pass)..."
	@$(LATEX) $(LATEX_FLAGS) $(MAIN).tex
	@mv $(MAIN).pdf $(OUTPUT).pdf
	@echo "✓ Quick PDF generated: $(OUTPUT).pdf"

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

# View the PDF (macOS)
view: $(OUTPUT).pdf
	@open $(OUTPUT).pdf

# View the PDF (Linux)
view-linux: $(OUTPUT).pdf
	@xdg-open $(OUTPUT).pdf

# View the PDF (Windows)
view-windows: $(OUTPUT).pdf
	@start $(OUTPUT).pdf

# Help
help:
	@echo "SIRCULAR CPU Reference Manual - Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  make          - Build the complete PDF (default)"
	@echo "  make quick    - Quick build (single pass, for drafts)"
	@echo "  make clean    - Remove temporary files"
	@echo "  make cleanall - Remove all generated files including PDF"
	@echo "  make view     - Open the PDF (macOS)"
	@echo "  make help     - Show this help message"
	@echo ""
	@echo "The output PDF will be named: $(OUTPUT).pdf"

# Word count (approximate, from .tex files)
wordcount:
	@echo "Approximate word count:"
	@texcount -total -q $(MAIN).tex chapters/*.tex

format:
	latexindent $(MAIN).tex chapters/*.tex