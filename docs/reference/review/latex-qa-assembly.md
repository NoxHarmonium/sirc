# LaTeX QA: assembly programmer's manual

Build: existing main.log checked (newer than every chapter source). No undefined references or citations, no missing
files, no overfull vbox, no literal LaTeX in the PDF text.

Listings: `make listings` passes; 11 of 11 listings assemble.

Overfull hbox over 10pt (fix needed; text runs into or past the page edge):
- Ch.4, Table 4.2 (p.18): 62.0pt, overhangs the right margin.
- Ch.5, Table 5.2 (p.20): 114.9pt, "displacement f..." clipped at the page edge.
- Ch.6, the `--segment` item (p.22): 58.7pt, "label:offset:length:file" past the margin.
- Ch.8, Table 8.1 (p.29): 109.0pt, text clipped at the page edge.

Below the threshold: Ch.1 7.3pt, Ch.7 5.4pt (ADDR paragraph, p.26). 32 underfull hboxes (4 in Table 4.2, 28 in Table
7.1 cells): loose lines, no overflow.

Notation: `{ }` and `< >` render correctly in the Ch.4 synopsis and the Ch.6 listings and `--segment` items, but Ch.4
Table 4.1 (p.17) prints `<<` and `>>` as guillemets (a T1 typewriter ligature). Needs a fix such as `<{}<` or `\textless`.

Table 7.1 (three columns, verbatim messages) and the Ch.6 fixed-segments table fit within the margins.

Pages for a human to look at: p.17, p.18, p.20, p.22, p.29.
