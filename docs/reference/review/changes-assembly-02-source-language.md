# Changes: assembly/chapters/02-source-language.tex

## Findings applied
F-asm-10, F-asm-11, F-asm-13, F-asm-15, F-asm-22, F-con-23.

## Not applied as written
- F-asm-15 and F-con-23 conflict (one operand versus two). Applied F-con-23's two-operand wording (the .EQU name is also unhashed) with F-asm-15's "binary is not accepted".
- F-asm-9 is marked simulator; left alone.

## Copy edit
Serial commas (heading, Case, Registers), "Number Forms" caption, `\reg{}` for registers, comma after "for example".

## Open questions
None.

## Improvements

- F-cra-3, F-cra-5, F-cra-6, F-cra-7: added Table "Notation Key" at the start of Chapter 2 (sigils #, @, $, :, braces, angle brackets, rO, immediate vs short immediate) with pointers to defining sections, plus a note that [ ] and | are literal. Added labels sec:names, sec:numbers, sec:named-constants. Pointers into Chapters 3 and 4 use chapter refs plus section titles because those files have no section labels and are not owned here.
- No TODO(author) needed.
