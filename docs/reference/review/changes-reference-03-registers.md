# Changes: reference/chapters/03-registers.tex

## Findings applied
F-con-13, F-arch-10 (merged; same sentence, figure boxes relabeled), F-arch-11, F-con-17 (duplicate of F-arch-11), F-arch-12, F-arch-13, F-arch-14, F-con-9 (Usage Conventions paragraph, per the author's decision). F-arch-3 is `simulator` and was left alone.

## Not applied as written
- F-arch-13: added as a third bullet of the top-level list in High Address Register Restrictions.
- F-arch-10: the 8-bit figure box now holds "High register [15:8] (not used for addressing)". It may be too wide to fit; check the rendering.

## Open questions
- Should the figure box text be shortened or split over two lines if it overflows?

Copy edit: "privilege exception" changed to "privilege violation fault" (two places) to match the rest of the chapter; "User programs" changed to "Programs".

## Layout

Figure 3.2 (bytefield): the gray "High register [15:8] (not used for addressing)" label overflowed its 8-bit cell. Set the label as a three-line \shortstack in \footnotesize (words unchanged), widened the figure from bitwidth=0.8em to 1.1em (as in Figure 3.1) and set bitheight=3.2\baselineskip so the cell holds the label. Label no longer overlaps the ruler or captions (checked PDF p.39).
