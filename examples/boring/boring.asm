:init
SETR x1, 5
SETR x2, 3
SETR x3, 64

:loop
ADDR x1, x2
CPLT x2, x3
JPEQ @loop
HALT
