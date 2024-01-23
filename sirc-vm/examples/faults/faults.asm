;; Segments
.EQU $PROGRAM_SEGMENT               #0x0000
.EQU $SCRATCH_SEGMENT               #0x1000

;; Devices
; Debug
.EQU $DEBUG_DEVICE_SEGMENT         #0x000B
.EQU $DEBUG_DEVICE_BUS_ERROR       #0x0000

; Reserved space for 128x32 bit exception vectors
.ORG 0x0000
.DQ @init
.ORG 0x0002
.DQ @bus_fault_handler
.ORG 0x0004
.DQ @alignment_fault_handler
.ORG 0x0006
.DQ @segment_overflow_fault_handler
.ORG 0x0008
.DQ @invalid_opcode_fault_handler
.ORG 0x000A
.DQ @privilege_violation_fault_handler

.ORG 0x0100
:init
LOAD    r1, #1
LOAD    r2, #1
LOAD    r3, #1
LOAD    r4, #1
LOAD    r5, #1

; Get the debug device to raise a fault on the bus
LOAD    ah, $DEBUG_DEVICE_SEGMENT
LOAD    al, $DEBUG_DEVICE_BUS_ERROR
STOR    (#0, a), r1

; Jumping to an odd address triggers an alignment fault
LOAD    pl, #0x0101

:after_odd_address

; Loading/storing from an odd address should not trigger an alignment fault
LOAD    ah, $SCRATCH_SEGMENT
LOAD    al, #0x0
LOAD    r6, #0xCAFE
STOR    (#1, a), r6
LOAD    r6, (#1, a)

; Calculating an effective address that overflows triggers a segment overflow fault
; (but only if the SR is set up the right way)
LOAD    ah, $PROGRAM_SEGMENT
LOAD    al, #0xFFFE

; This should not trigger fault
LOAD    r6, (#2, a)
; Switch on trap on overflow bit
ORRI     sr, #0x4000
; This _should_ trigger a fault
LOAD    r6, (#2, a)

; There is no coprocessor at ID 4, so should trigger an invalid opcode fault
COPI    r1, #0x4000

; Switch to non privillaged mode
ORRI     sr, #0x0100
; Try to escape the current segment
LOAD     ph, #0xFEFE

; Halt CPU
COPI    r1, #0x14FF

:bus_fault_handler
ADDI    r1, #1
RETE

:alignment_fault_handler
ADDI    r2, #1

; Fix up the return address because the original PC causes an alignment fault
; Returning witout fixing up the return address would cause an endless loop

; Transfer current ELR into 'a' register
COPI    r1, #0x1C16
; Correct the lower word
LOAD    al, @after_odd_address
; Transfer corrected address back to ELR - TODO Why 5? Wouldn't it be six?
COPI    r1, #0x1D16

RETE

:segment_overflow_fault_handler
ADDI    r3, #1
RETE

:invalid_opcode_fault_handler
ADDI    r4, #1
RETE

:privilege_violation_fault_handler
ADDI    r5, #1
RETE
