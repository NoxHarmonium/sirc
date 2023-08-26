; Reserved space for 128x32 bit exception vectors

.ORG 0x0000
DQ @init

.ORG 0x0100

:init
LOAD    r1, #0xCAFE
LOAD    r3, #0xBEEF
LOAD    r5, #0x000A

LOAD    ah, #0x0008
LOAD    al, #0xAAF0

STOR    (#0x000F, a), r1
LOAD    r2, (#0x000F, a)
STOR    (r5, a), r3
LOAD    r4, (r5, a)

; Halt CPU
COPI    r1, #0x14FF