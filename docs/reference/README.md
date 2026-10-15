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

# Fails if the last build left any reference/citation unresolved. Runs
# automatically as the last step of `make all`; standalone use needs an
# existing main.log (i.e. run after `make all`).
make check-refs

# QA: every check above plus content verification against the real sirc-vm
# toolchain (see "QA" below). Everything runs locally; nothing is fetched or
# pushed anywhere.
make qa
```

Run `make format` after every change to the manual, before committing. This is enforced by the repo's pre-commit
hook (`.githooks/pre-commit`), which runs `make format-check` whenever a commit touches `docs/reference` and fails
the commit if any file is unformatted -- it does not format for you, so run `make format` yourself and re-stage.

## QA (`make qa` / `./qa.sh`)

`qa.sh` is a thin runner -- every check is a plain `make` target, in this Makefile or one of the
`examples/*/Makefile`s, so a CI pipeline can call those targets directly instead of going through the script; the
script exists purely for a single local command. In order:

1. `make format-check`.
2. `make all` -- a full build, which also runs `check-refs` as its last step (see above).
3. `make -C ../../examples/manual-examples objects` -- assembles every code listing extracted from the manual (see
   below) with the real assembler. This is an ordinary `make` pattern rule against a directory of `.sasm` files, so
   re-running it only re-assembles listings that actually changed -- no separate skip-build flag needed.
4. `make -C <dir> clean check` for every `examples/<name>/` directory an Appendix D `\lstinputlisting` points at
   (discovered from the manual source, not hardcoded) -- assemble, link, run under the VM, and diff against each
   example's own checked-in expected fixture. This also correctly handles multi-file programs (e.g.
   `hardware-exception.sasm` + `data.sasm`), since it just runs each example's real build recipe rather than
   re-guessing it.

### Extracting listings from the manual (`examples/manual-examples/`)

The short Usage Examples scattered through the instruction chapters aren't real files on disk -- they're inline
`\begin{lstlisting}` blocks inside the `.tex` source. `examples/manual-examples/Makefile` pulls them out into
`extracted/` (gitignored) so they can be assembled like any other example:

- `make extract` runs `extract-listings.awk` over every chapter, but only rewrites a `.sasm` file when its content
  actually changed, so editing one chapter doesn't touch every other listing's mtime.
- `make objects` (the default target) re-extracts if needed, then assembles every `extracted/*.sasm` to a matching
  `.o` via an ordinary pattern rule -- so `make`'s own dependency tracking decides what actually needs reassembling,
  the same as any other C-like build.
- A listing is only extracted if it's inside a `\begin{example}` float (the Usage Examples), or opted in with a
  `% qa:assemble` comment on the line before its `\begin{lstlisting}`. Bare `Syntax:`/`Operation:` blocks use
  placeholder operands like `#opcode` or pseudocode like `if condition_true then` and are left alone -- they aren't
  real SIRC-1 assembly.
- Each extracted file is named `<chapter-file>__<nearest-heading-or-mnemonic>__<n>.sasm`, so a failure points
  straight at roughly where in the manual to look.

Deliberately out of scope for now: deep prose/terminology review (definition-before-use, confusing passages) is a
job for the `manual-cold-reader` agent pass, not something a Makefile can meaningfully approximate.

## Tracking Open Questions

Don't leave a TODO as rendered prose in the manual unless it's meant to be visible to readers (e.g. a genuinely
provisional figure, called out as such). For anything else -- a gap noticed while editing, a decision deferred, a
diagram that needs rework -- use the same comment convention already used across the Rust codebase, just with `%`
instead of `//`:

```latex
% TODO: Update the diagram so the memory component is visually separate from the other components on the die
% category=Manual
```

The `% TODO:`/`% category=` pair is invisible in the rendered PDF (LaTeX comments never render), and
`.github/workflows/todo.yml` already scans the whole repository on every push to `main` and files a GitHub issue per
TODO comment it finds -- `.tex` files aren't excluded, so this gets issue tracking for free without any new tooling.
Use `category=Manual` to distinguish these from the Rust codebase's existing categories (`Toolchain`, `Refactoring`,
`Bugs`, etc.).
