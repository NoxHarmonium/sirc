# SIRC-1 CPU Reference Manual

This directory contains the LaTeX source for the SIRC-1 CPU Reference Manual, documenting the SIRCIS (SIRC Instruction Set) architecture.

## Contents

- `main.tex` - Main document file
- `preamble.tex` - Package imports and formatting definitions
- `chapters/` - Individual chapter files

## Building the PDF

### Prerequisites

You need a LaTeX distribution installed:

- **macOS**: MacTeX (`brew install --cask mactex`)
- **Linux**: TeX Live (`sudo apt-get install texlive-full`)
- **Windows**: MiKTeX or TeX Live

### Build Commands

```bash
# Build the PDF
make

# Clean temporary files
make clean

# Clean everything including PDF
make cleanall

# Quick build (single pass, for drafts)
make quick

# Indent the tex files automatically to make everything consistent
# On MacOS you might need to install this with `brew install latexindent`. The version that comes with maxtex seems broken.
make format
```

### Manual Build

If you don't have `make`, you can build manually:

```bash
pdflatex main.tex
pdflatex main.tex  # Run twice for TOC and cross-references
pdflatex main.tex  # Run three times to resolve all references
```

## Document Structure

### Part I: Architecture Overview

- Chapter 1: Introduction
- Chapter 2: Register Model
- Chapter 3: Status Register

### Part II: Instruction Set Architecture

- Chapter 4: Instruction Formats
- Chapter 5: Addressing Modes
- Chapter 6: Shift Operations
- Chapter 7: Condition Codes

### Part III: SIRCIS Instruction Reference

- Chapter 8: Instruction Summary
- Chapter 9: ALU Instructions
- Chapter 10: Memory Access Instructions
- Chapter 11: Control Flow Instructions
- Chapter 12: Coprocessor Instructions
- Chapter 13: Meta-Instructions

### Appendices

- Appendix A: Complete Opcode Map
- Appendix B: Instruction Timing
- Appendix C: Undocumented Instructions

## Customization

### Colors

Edit `preamble.tex` to customize colors:

```latex
\definecolor{sircblue}{RGB}{0,51,102}
\definecolor{sircgray}{RGB}{128,128,128}
```

### Fonts

The document uses Latin Modern fonts by default. To change:

```latex
\usepackage{times}  % Times Roman
\usepackage{palatino}  % Palatino
```

### Page Layout

Adjust geometry in `preamble.tex`:

```latex
\geometry{
    letterpaper,
    margin=1in,
}
```

## Adding Content

### New Chapters

1. Create a new `.tex` file in `chapters/`
2. Add `\input{chapters/your-chapter}` to `main.tex`

### Assembly Code Examples

Use the predefined SIRC language:

```latex
\begin{lstlisting}
ADDI r1, #100
ADDR r2, r1, r3
\end{lstlisting}
```

### Instruction Documentation

Use the `instructionbox` environment:

```latex
\begin{instructionbox}{ADDI -- Add Immediate}
\textbf{Opcode:} 0x00

\textbf{Syntax:}
\begin{lstlisting}
ADDI rD, #imm16
\end{lstlisting}

\textbf{Description:}
Adds an immediate value to a register.

\end{instructionbox}
```

## Style Guide

### Register Names

Use `\reg{r1}` for register names: \reg{r1}, \reg{sr}

### Opcodes

Use `\opcode{00}` for opcodes: \opcode{00}

### Mnemonics

Use `\mnemonic{ADDI}` for instruction names: \mnemonic{ADDI}

### Immediate Values

Use `\imm{100}` for immediate values: \imm{100}

## Troubleshooting

### "Command not found" errors

Install missing LaTeX packages via your distribution's package manager.

### Cross-references not working

Run pdflatex multiple times (usually 3 times) to resolve all references.

### Compilation errors

Check the log file (`main.log`) for detailed error messages.

## License

This documentation is part of the SIRC project. See the main repository LICENSE for details.

## Contributing

When adding or modifying content:

1. Keep the style consistent with existing chapters
2. Include assembly examples where appropriate
3. Update the table of contents if adding new chapters
4. Test compilation before committing

## Contact

For questions or contributions, see the main SIRC repository.

## Acknowledgments

These Latex files were created with the help of the Claude Sonnet 4.5 LLM model.
