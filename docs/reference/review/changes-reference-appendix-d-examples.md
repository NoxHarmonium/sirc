# Changes: reference/chapters/appendix-d-examples.tex

## Findings applied
F-sum-8, F-sum-9, F-sum-12, F-con-31, F-con-38 (appendix D location: the Assembly Programmer's Manual title now uses `\asmmanual{}`, which `preamble.tex` provides).

## Not applied as written
- F-sum-10 (simulator, left alone), F-sum-11 (status: merged into F-con-1, which names only assembly/chapters/06-toolchain.tex; no change here).
- Line numbers quoted in the prose (158, 197--242, 137, 26, 37, 8--19, 35--50) were not rechecked against the example files, which are outside this task.

## Open questions
- F-sum-11 suggested optionally noting in Program Conventions that the final `COPI #0x14FF` is a simulator exit hook. The author's decision on F-con-1 places that note in the assembly manual, so none was added here. Confirm that is intended.
