; Reserved space for 128x32 bit exception vectors

.ORG 0x0000
.DQ @init

.ORG 0x0200

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


; Lets test jumping!

; TODO: Add some tests that:
; LJMP to another segment
; LJMP with offset
; LJMP with shifted offset
; Same three things but with LJSR
; Make a maze of jumps so if the jump goes to the wrong place it will land in the forbidden zone where every instruction jumps to the fail label

; r7 will store the test result
LOAD r7, #0
; Use the stack address register pair to store fail address
LOAD sh, #0x0
LOAD sl, @fail
; Address register pair used as part of test
LOAD ah, #0x0
LOAD al, #0x0

BRAN #12            ; Step 1
LJMP s              ; FAIL!
LJMP s              ; FAIL!
BRAN #10            ; Step 3
LJMP s              ; FAIL!
LJMP s              ; FAIL!
BRAN #-6            ; Step 2
LJMP s              ; FAIL!
BRAN @ljmp_register_offset


:ljmp_register_offset
LOAD al, @ljmp_register_offset_anchor
LOAD r1, #-6
LOAD r2, #4
BRAN @ljmp_register_offset_anchor
LJMP s              ; FAIL!
LJMP s              ; FAIL!
LJMP s              ; FAIL!
LJMP a, r2
LJMP s              ; FAIL!
LJMP s              ; FAIL!
:ljmp_register_offset_anchor
LJMP a, r1
LJMP s              ; FAIL!
BRAN @ljmp_immediate_offset
LJMP s              ; FAIL!
LJMP s              ; FAIL!
LJMP s              ; FAIL!
LJMP s              ; FAIL!


:ljmp_immediate_offset
LOAD al, @ljmp_immediate_offset_anchor
BRAN @ljmp_immediate_offset_anchor
LJMP s              ; FAIL!
LJMP s              ; FAIL!
LJMP s              ; FAIL!
LJMP a, #4
LJMP s              ; FAIL!
LJMP s              ; FAIL!
:ljmp_immediate_offset_anchor
LJMP a, #-6
LJMP s              ; FAIL!
BRAN @ljsr_register_offset
LJMP s              ; FAIL!
LJMP s              ; FAIL!
LJMP s              ; FAIL!
LJMP s              ; FAIL!


:ljsr_register_offset
LOAD al, @ljsr_register_offset_anchor
LOAD r1, #-6
LOAD r2, #4
BRAN @ljsr_register_offset_anchor
LJSR s              ; FAIL!
LJSR s              ; FAIL!
LJSR s              ; FAIL!
LJSR a, r2
LJSR s              ; FAIL!
LJSR s              ; FAIL!
:ljsr_register_offset_anchor
LJSR a, r1
LJSR s              ; FAIL!
BRAN @ljsr_immediate_offset
LJSR s              ; FAIL!
LJSR s              ; FAIL!
LJSR s              ; FAIL!
LJSR s              ; FAIL!


:ljsr_immediate_offset
LOAD al, @ljsr_immediate_offset_anchor
BRAN @ljsr_immediate_offset_anchor
LJSR s              ; FAIL!
LJSR s              ; FAIL!
LJSR s              ; FAIL!
LJSR a, #4
LJSR s              ; FAIL!
LJSR s              ; FAIL!
:ljsr_immediate_offset_anchor
LJSR a, #-6
LJSR s              ; FAIL!
BRAN @finish
LJSR s              ; FAIL!
LJSR s              ; FAIL!
LJSR s              ; FAIL!
LJSR s              ; FAIL!


:finish
LOAD r7, #0x0FAB

LOAD sh, #0x0
LOAD sl, @pass
LJMP s

.ORG 0x0400

:fail
LOAD r7, #0xFA11

:pass

; Halt CPU
COPI    r1, #0x14FF
