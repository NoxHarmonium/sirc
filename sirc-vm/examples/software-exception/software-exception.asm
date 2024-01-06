; Reserved space for 128x32 bit exception vectors

; First vector is the reset vector
.ORG 0x0000
.DQ @start

; 64th vector (*2 words per vector)
.ORG 0x0080
.DQ @exception_handler
.DQ @exception_handler_that_should_be_ignored

.ORG 0x0100
:start

; Reset register
LOAD    r1, #1
; Trigger Software Exception (TODO: Create psuedo instructions for these)
COPI    r1, #0x1140

; r1 should be 0x0FF here after exception handler runs

; Halt CPU
COPI    r1, #0x14FF

.ORG 0x0200
:exception_handler

; Load a value to detect if this code runs
; Final value of r1 should be this value
LOAD    r1, #0xFF
; Try and trigger nested software exception (should be ignored)
COPI    r1, #0x1141
; Return from exception
COPI    r1, #0x1A00

.ORG 0x0400
:exception_handler_that_should_be_ignored

; Load a value to detect if this code runs
; Final value of r1 should _not_ be BB because this vector should never be jumped to
LOAD    r1, #0xBB
; Return from exception
COPI    r1, #0x1A00