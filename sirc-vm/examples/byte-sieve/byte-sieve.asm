; Ported from BASIC version at https://en.wikipedia.org/wiki/Byte_Sieve

:START

; Reset working registers
LOAD    x1, #0
LOAD    x2, #0
LOAD    x3, #0

; Current Prime
LOAD    y1, #0
; K
LOAD    y2, #0
; COUNT
LOAD    y3, #0

; Constants
; - Zero
LOAD    z1, #0
; - One
LOAD    z2, #1
; - Size
LOAD    z3, #8190

; File I/O segment
LOAD    ah, #0x00F0
; Array Start = 0x00F0_0000
LOAD    al, #0x0000

; FOR I = 0 TO SIZE
:SETUP
; FLAGS (I) = 1
STOR    (x1, a), z2
ADDR    x1, z2
; NEXT I
COMP    z3, x1
BRAN|>> @SETUP

;FOR I = 0 TO SIZE
LOAD    x1, #0
:SIEVE
; IF FLAGS (I) = 0 THEN 18
LOAD    x2, (x1, a)
COMP    x2, z1
BRAN|== @SIEVE
; PRIME = I+I+3
LOAD    y1, x1
ADDR    y1, y1
LOAD    x3, #3
ADDR    y1, x3
; K = I + PRIME
LOAD    y2, x1
ADDR    y2, y1

:UPDATE_K
; IF K > SIZE THEN 17
COMP    y2, z3
BRAN|>> @NEXT
; FLAGS (K) = 0
STOR    (x1, a), z1
; K = K + PRIME
ADDR    y2, y1
BRAN    @UPDATE_K

:NEXT
ADDR    y3, z2
BRAN    @SIEVE

:DONE
; Result is in COUNT (y3)

