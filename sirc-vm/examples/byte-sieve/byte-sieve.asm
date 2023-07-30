; Ported from BASIC version at https://en.wikipedia.org/wiki/Byte_Sieve

:START

; Reset working registers
LOAD    r1, #0
LOAD    r2, #0
LOAD    r3, #0

; Current Prime
LOAD    r4, #0
; K
LOAD    r5, #0
; COUNT
LOAD    r6, #0

LOAD    r7, #1

; File I/O segment
LOAD    ah, #0x00F0
; Array Start = 0x00F0_0000
LOAD    al, #0x0000

; FOR I = 0 TO SIZE
:SETUP
; FLAGS (I) = 1
STOR    (r1, a), r7
ADDI    r1, #1

LOAD    r7, #0

; NEXT I
CMPI    r1,#8190
BRAN|<= @SETUP

;FOR I = 0 TO SIZE
LOAD    r1, #0
:SIEVE
; IF FLAGS (I) = 0 THEN 18
LOAD    r2, (r1, a)
CMPR    r2, r7
BRAN|== @SIEVE
; PRIME = I+I+3
LOAD    r4, r1
ADDR    r4, r4
LOAD    r3, #3
ADDR    r4, r3
; K = I + PRIME
LOAD    r5, r1
ADDR    r5, r4

:UPDATE_K
; IF K > SIZE THEN 17
CMPI    r5, #8190
BRAN|>> @NEXT
; FLAGS (K) = 0
STOR    (r1, a), r7
; K = K + PRIME
ADDR    r5, r4
BRAN    @UPDATE_K

:NEXT
ADDI    r6, #1
BRAN    @SIEVE

:DONE
; Result is in COUNT (r6)

; Halt CPU
COPI    r1, #0x14FF
