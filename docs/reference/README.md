# SIRC-1 CPU Reference Manual

## Diagram Tooling

The CPU block diagram is sourced from Graphviz and committed as a rendered PDF under `diagrams/`.

- Building the manual does not require Graphviz when the committed diagram PDF is up to date.
- Graphviz is only required when changing `.dot` sources and regenerating diagrams.
- Regenerate diagrams explicitly with `make graphviz`.

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
```
