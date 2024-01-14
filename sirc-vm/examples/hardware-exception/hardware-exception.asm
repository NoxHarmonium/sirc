; Reserved space for 128x32 bit exception vectors


.EQU $FALSE                         #0x0
.EQU $TRUE                          #0x1

; When this character is typed, the program should exit
.EQU $EXPECTED_CHAR                 #0x61

;; Segments
.EQU $PROGRAM_SEGMENT               #0x0000

;; Devices
; Serial
.EQU $SERIAL_DEVICE_SEGMENT         #0x000A
.EQU $SERIAL_DEVICE_BAUD            #0x0000
.EQU $SERIAL_DEVICE_RECV_ENABLED    #0x0001
.EQU $SERIAL_DEVICE_RECV_PENDING    #0x0002
.EQU $SERIAL_DEVICE_RECV_DATA       #0x0003
.EQU $SERIAL_DEVICE_SEND_ENABLED    #0x0004
.EQU $SERIAL_DEVICE_SEND_PENDING    #0x0005
.EQU $SERIAL_DEVICE_SEND_DATA       #0x0006


;; Exception Vectors
; First vector is the reset vector
.ORG 0x0000
.DQ @start

.ORG 0x0080
.DQ @exception_handler_p2

.ORG 0x0100
:start

; Setup routines
BRSR @setup_serial
BRSR @print_help

:wait_for_char
; Wait for exception (will spin until interrupted)
COPI    r1, #0x1900

; Pending byte should be in r7
CMPI    r7, $EXPECTED_CHAR
BRAN|== @finish

BRAN @wait_for_char

:setup_serial

LOAD    r1, #9600
LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_BAUD
; TODO: Allow omitting the #0 offset when not using an offset (infer the #0)
STOR    (#0, a), r1

LOAD    r1, #0x1
LOAD    al, $SERIAL_DEVICE_RECV_ENABLED
STOR    (#0, a), r1
LOAD    al, $SERIAL_DEVICE_SEND_ENABLED
STOR    (#0, a), r1

RETS

:print_help
; Loop counter
LOAD r4, #1
; String size (*2 due to words getting padded to dw)
LOAD r5, #34

:print_loop_start
LOAD    ah, $PROGRAM_SEGMENT
LOAD    al, @help_message
LOAD    r3, (r4, a)

LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_SEND_DATA
STOR    (#0, a), r3

LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_SEND_PENDING
LOAD    r3, $TRUE
STOR    (#0, a), r3

; TODO: Should we use an interrupt for this instead of spinning?
:wait_for_print_ready
LOAD    r3, (#0, a)
CMPI    r3, $TRUE
BRAN|== @wait_for_print_ready

; Increment by two because of lack of packing (a word actually is padded to a DW)
ADDI    r4, #2
CMPR    r4, r5
BRAN|<< @print_loop_start
RETS

:finish

; r1 should be 0x0FF here after exception handler runs

; Halt CPU
COPI    r1, #0x14FF

.ORG 0x0400
:exception_handler_p2

; Check if there is something we need to read
LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_RECV_PENDING
LOAD    r1, (#0, a)

; Return early if not
CMPI    r1, $FALSE
RETE|==

; Read pending byte
LOAD    al, $SERIAL_DEVICE_RECV_DATA
LOAD    r7, (#0, a)

LOAD    al, $SERIAL_DEVICE_RECV_PENDING
LOAD    r1, $FALSE
STOR    (#0, a), r1

RETE

.ORG 0x0800
; Message to print in ASCII codes (length 17)
:help_message
.DW #84
.DW #121
.DW #112
.DW #101
.DW #32
.DW #39
.DW #97
.DW #39
.DW #32
.DW #116
.DW #111
.DW #32
.DW #101
.DW #120
.DW #105
.DW #116
.DW #10

; TODO: Why do I need this here to make this file parse???
NOOP