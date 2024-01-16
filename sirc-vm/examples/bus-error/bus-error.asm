
;; Devices
; Debug
.EQU $DEBUG_DEVICE_SEGMENT         #0x000B
.EQU $DEBUG_DEVICE_BUS_ERROR       #0x0000

; Reserved space for 128x32 bit exception vectors
.ORG 0x0000
.DQ @init
.ORG 0x0002
.DQ @bus_fault_handler

.ORG 0x0100
:init
LOAD    r1, #1

LOAD    ah, $DEBUG_DEVICE_SEGMENT
LOAD    al, $DEBUG_DEVICE_BUS_ERROR
STOR    (#0, a), r1

; Halt CPU
COPI    r1, #0x14FF

:bus_fault_handler

LOAD    r1, #2

RETE