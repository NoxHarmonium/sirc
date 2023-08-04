; https://rosettacode.org/wiki/Sieve_of_Eratosthenes

:main

; File I/O segment
LOAD    ah, #0x00F0
; Array Start = 0x00F0_0000
LOAD    al, #0x0000

LOAD    r7, #2
LOAD    r1, #2
LOAD    r2, #1

:1

STOR    (r1, a), r2
ADDI    r1, #2
CMPI    r1, #1000
BRAN|<= @1
LOAD    r1, #3
LOAD    r3, #1

:2

LOAD    r2, (r1, a)
CMPI    r2, #1
BRAN|== @4
; THIS NUMBER IS A PRIME!
; TODO: Log this somehow
LOAD    r7, r1
LOAD    r2, r1

:3
STOR    (r2, a), r3
ADDR    r2, r1
CMPI    r2, #1000
BRAN|<= @3

:4
ADDI    r1, #2
CMPI    r1, #1000
BRAN|<= @2

:DONE
; Result is in COUNT (r6)

; Halt CPU
COPI    r1, #0x14FF
