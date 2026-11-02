# Changes: assembly/chapters/06-toolchain.tex

## Findings applied
F-con-1, F-asm-12, F-asm-20, F-asm-26, F-asm-27, F-con-28.

## Not applied as written
- F-con-1: the example keeps `COPI #0x14FF`, as the decision directs. A paragraph after the example documents it as a simulator-only test hook that raises an invalid-opcode fault on a conforming CPU. The matching Chapter 8 text belongs to that chapter's editor.
- F-asm-27: the fixed-segment table gives PROGRAM, TERMINAL, DEBUG and VIDEO, taken from sirc-vm/sirc-vm/src/main.rs. VIDEO is mapped only with `--enable-video`.
- F-con-9 names this chapter only as evidence. No change here.

## Copy edit
- The `--segment` synopsis now uses `{ }` and `< >`; the linker synopsis is `<OBJECT FILE> ...`.
- Changed `\texttt{r1}` to `\reg{r1}`, "the architecture manual" to `\archmanual{}`, and split or recast a few sentences.
- Added a caption in Title Case and the label `tab:fixed-segments`.

## Open questions
- The 64 K scratch example now uses length 10000 (hexadecimal) per F-asm-26.
- The table states PROGRAM as read-only, from `writeable: false` in main.rs. Please confirm the intent.
