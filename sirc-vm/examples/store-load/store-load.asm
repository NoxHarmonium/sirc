:init
LOAD    x1, #0xCAFE
LOAD    z3, #0xBEEF
LOAD    y1, #0x000A

LOAD    ah, #0x0000
LOAD    al, #0xAAF0

STOR    (#0x000F, a), x1
LOAD    x2, (#0x000F, a)
STOR    (y1, a), z3
HALT