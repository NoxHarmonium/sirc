; Reserved space for 128x32 bit exception vectors
.ORG 0x0000
DQ @start

; 64th vector (*2 words per vector)
.ORG 0x0080
DQ @exception_handler

.ORG 0x0100

:start

LOAD    r1, #1

; Trigger Software Exception
COPI    r1, #0x1440

; r1 should be 0x0FF here after exception handler runs

; Halt CPU
COPI    r1, #0x14FF

.ORG 0x0200

:exception_handler

LOAD r1, #0xFF

COPI    r1, #0x1200
