:init
SET x1, 5
SET x2, 3
SET x3, 64

:loop
ADD x1, x2
CLT x2, x3
JUMPIF @loop
HALT
