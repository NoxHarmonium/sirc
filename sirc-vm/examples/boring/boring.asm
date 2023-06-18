:init
LOAD    r1, #5
LOAD    r2, #3
LOAD    r3, #64

:loop
ADDR    r2, r1
; Remember that COMP has the same argument order as SUBR
CMPR    r3, r2
BRAN|>= @loop
