:init
LOAD    x1, #5
LOAD    x2, #3
LOAD    x3, 64

:loop
ADDR    x2, x1
; Remember that COMP has the same argument order as SUBR
COMP    x3, x2
BRAN|>= @loop
HALT
