; https://rosettacode.org/wiki/Sieve_of_Eratosthenes

; Reserved space for 128x32 bit exception vectors
.ORG 0x0000
DQ @main

.ORG 0x0100
:main

; File I/O segment
LOAD    ah, #0x00F0
; Array Start = 0x00F0_0000
LOAD    al, #0x0000

; Stack segment (other end of mapped file)
LOAD    sh, #0x00F0
; Array Start = 0x00F0_0000
LOAD    sl, #0xFFFF

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
STOR    -(#0, s), r1
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

; Halt CPU
COPI    r1, #0x14FF
