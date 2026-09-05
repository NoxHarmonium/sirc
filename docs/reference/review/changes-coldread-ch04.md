# Cold-read follow-up: Chapter 4 (Data Representation)

## Change made

At the first use of `SR.A` in Section "Arithmetic and Address Wraparound", added a footnote
glossing the `SR.X` dot-notation convention:

> `SR.X` denotes bit or field `X` of the status register; see Chapter~\ref{ch:status-register}
> for the bit layout and Chapter~\ref{ch:reading-instructions} for the full notation convention.

This is attached to the existing `\texttt{SR.A} (Trap on Address Overflow)` occurrence via
`\footnote{...}`, with forward cross-references to Chapter 5 (status register bit layout) and
Chapter 11 (notation table). No other text was changed; no content was restated from either
target chapter.

## Verification

`\begin{...}` / `\end{...}` counts and overall brace counts were confirmed balanced after the
edit.
