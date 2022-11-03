:init
LOAD    x1, 5
LOAD    x2, 3
LOAD    x3, 64

:loop
ADDR    x2, x1
COMP    x2, x3
BRAN|<= @loop
HALT
