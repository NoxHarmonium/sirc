;; Segments
.EQU $PROGRAM_SEGMENT               #0x0000
.EQU $SCRATCH_SEGMENT               #0x0001
.EQU $PROGRAM_SEGMENT_HIGH          #0x0002

; Section to test program counter overflow
; without trying to execute the vector table
; that lives at 0x0000 in the main segment

; Note: This is not linked together with the main program
; This is compiled as a separate program and mapped as a separate segment

.ORG 0x0000

LOAD    ah, $PROGRAM_SEGMENT
LOAD    al, #0xFFF0
LJMP    a


; TODO: Work out how to avoid padding this out
; I had to pad it out because even though I said this segment has a length of 0xFFFF
; I think the way files are mapped means that length is ignored
.ORG    0xFFFE
LOAD    r1, r1
