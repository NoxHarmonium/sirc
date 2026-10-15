# Changes — appendix-e-quick-reference.tex

## Findings applied
- F-cons-7: Fixed the cross-reference in the BAT Reference section from
  `Chapter~\ref{ch:cpu-architecture}, Section "Pin Descriptions"` to
  `Appendix~\ref{appendix:external-interface}, Section "Pin Descriptions"`. Verified
  the label `appendix:external-interface` exists in appendix-f-external-interface.tex
  and that its "Pin Descriptions" subsection (line 152) has no `\label`, so the
  quoted-name form (as specified in the finding's fix text) is the only option.
- F-cons-20: Replaced the one British "behaviour" (Instruction Mnemonic Index intro)
  with "behavior".

## Findings I could not apply as written
None — both applied exactly as specified.

## Open questions for the user
- None from this file. Note: I did not touch F-cons-16 (rename `\label{ch:quick-reference}`
  to `\label{appendix:quick-reference}`), since it wasn't in my assigned finding list and
  it also touches appendix-d-examples.tex, owned by another editor.

## Copy edit (beyond the two findings)
- Fixed a subject–verb agreement error in the mnemonic-index intro: "N, Z, C, V: uses the
  symbols..." → "...use the symbols..." (plural subject).

Two-line summary: Redirected a misdirected cross-reference to the correct appendix and
standardized on American spelling; fixed one grammar slip found during the copy-edit pass.
