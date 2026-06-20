# Reference Encodings

This fixture assembles the instruction examples used by the reference manual.

The generated `reference-encodings.bin` is the raw binary source of truth, and
`reference-encodings.hex` is a review-friendly listing that pairs each
`@manual-example` annotation with the assembled 32-bit instruction word.

Run:

```bash
make check
```

To refresh the checked-in LaTeX fragments used by the reference manual:

```bash
make manual-tex
```

This writes one checked-in LaTeX fragment per instruction format under
`../../docs/reference/generated/`.

This example is build-only. It does not run in the VM or compare a register
dump, so `examples/check-all.sh` can include it without extra expected-output
files.
