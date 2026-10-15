# SIRC-1 CPU Reference Manual

## Diagram Tooling

The CPU block diagram is sourced from Graphviz and committed as a rendered PDF under `diagrams/`.
Bus timing waveforms are authored directly in LaTeX with the `tikz-timing` package.

- Building the manual does not require Graphviz when the committed diagram PDF is up to date.
- Graphviz is only required when changing `.dot` sources and regenerating diagrams.
- Regenerate diagrams explicitly with `make graphviz`.
- Building the manual requires `tikz-timing`. This is included in a normal TeX Live/MacTeX installation; if it is
  missing, `make` will fail with a `tikz-timing.sty` error.

On macOS you can install Graphviz with:

```bash
brew install graphviz
```

## Make Commands

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

# Check formatting without changing anything (exits non-zero if `make format` would change a file)
make format-check

# QA: build the PDF, verify every worked-example program and code listing
# against the real assembler/linker (and the VM, for the Appendix D
# programs, via their own `make check`), and check for undefined
# cross-references. Everything runs locally against the sirc-vm toolchain in
# this checkout; nothing is fetched or pushed anywhere.
make qa
```

Run `make format` after every change to the manual, before committing. This is enforced by the repo's pre-commit
hook (`.githooks/pre-commit`), which runs `make format-check` whenever a commit touches `docs/reference` and fails
the commit if any file is unformatted -- it does not format for you, so run `make format` yourself and re-stage.

## QA (`make qa` / `python3 qa.py`)

Runs `make format-check`, a full `make all` build, and then verifies content against the real `sirc-vm` toolchain:

- Every worked-example program in Appendix D (`\lstinputlisting`) is checked via its own `examples/<name>/Makefile`'s
  `check` target -- assemble, link, run under the VM, and diff against a checked-in expected fixture. This also
  correctly handles multi-file programs (e.g. `hardware-exception.sasm` + `data.sasm`) since it just runs the real
  build recipe rather than re-guessing it.
- Every inline `\begin{lstlisting}` block inside a `\begin{example}` float (the short Usage Examples throughout the
  instruction chapters) is fed to the real assembler. Bare listings outside an `example` float -- the `Syntax:` and
  `Operation:` pseudocode blocks, which use placeholder operands like `#opcode` and aren't meant to be literal
  code -- are skipped by default; opt one in by adding a `% qa:assemble` comment on the line before its
  `\begin{lstlisting}`.
- `main.log` is checked for any reference or citation LaTeX couldn't resolve after the full 3-pass build.

Useful flags: `--skip-build` reuses the existing PDF/log instead of rebuilding (faster when iterating on listings
only); `--render 10,50` renders the given 0-indexed PDF pages to PNG (via ImageMagick, if installed) for visual
spot-checks.

Deliberately out of scope for now: deep prose/terminology review (definition-before-use, confusing passages) is a
job for the `manual-cold-reader` agent pass, not something this script tries to approximate with regexes.
