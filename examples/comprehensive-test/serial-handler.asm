; TODO: Copy this version that supports segments to the other examples
.EQU $FALSE                         #0x0
.EQU $TRUE                          #0x1


;; Segments
.EQU $PROGRAM_SEGMENT               #0x0000
.EQU $SCRATCH_SEGMENT               #0x0001

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

;; Scratch Variables
.EQU $MESSAGE_SEND_BASE             #0x0000
.EQU $MESSAGE_SEND_POINTER          #0x0001
.EQU $MESSAGE_SEND_SEGMENT          #0x0002

:setup_serial

; Save the used registers
STOR -(s), ah
STOR -(s), al
STOR -(s), r1

LOAD    r1, #9600
LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_BAUD
STOR    (a), r1

; Restore the used registers
LOAD r1, (s)+
LOAD al, (s)+
LOAD ah, (s)+

RETS

:print

; Save the used registers
STOR -(s), ah
STOR -(s), al
STOR -(s), r1
STOR -(s), r2

LOAD    ah, $SCRATCH_SEGMENT
LOAD    al, $MESSAGE_SEND_BASE
STOR    (a), r1

; Store segment
LOAD    ah, $SCRATCH_SEGMENT
LOAD    al, $MESSAGE_SEND_SEGMENT
STOR    (a), r2

; Align pointer
LOAD    r1, #1

LOAD    ah, $SCRATCH_SEGMENT
LOAD    al, $MESSAGE_SEND_POINTER
STOR    (a), r1

LOAD    r1, $TRUE
LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_SEND_ENABLED
STOR    (a), r1

:wait_for_print_finish

; Wait for exception (will spin until interrupted)
COPI    r1, #0x1900

LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_SEND_ENABLED
LOAD    r1, (a)

CMPI    r1, $TRUE
BRAN|== @wait_for_print_finish

; Restore the used registers
LOAD r2, (s)+
LOAD r1, (s)+
LOAD al, (s)+
LOAD ah, (s)+

RETS

:exception_handler_p3

; Save the used registers
STOR -(s), lh
STOR -(s), ll
STOR -(s), ah
STOR -(s), al
STOR -(s), r1
STOR -(s), r2
STOR -(s), r3
STOR -(s), r4

BRSR @write_pending_byte

; Restore the used registers
LOAD r4, (s)+
LOAD r3, (s)+
LOAD r2, (s)+
LOAD r1, (s)+
LOAD al, (s)+
LOAD ah, (s)+
LOAD ll, (s)+
LOAD lh, (s)+

RETE

:write_pending_byte

; Check if the device is ready to write
LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_SEND_PENDING
LOAD    r1, (a)

; Return early if not (there is already a send pending)
CMPI    r1, $TRUE
RETS|==

LOAD    ah, $SCRATCH_SEGMENT
LOAD    al, $MESSAGE_SEND_POINTER
LOAD    r2, (a)
LOAD    al, $MESSAGE_SEND_SEGMENT
LOAD    r4, (a)
LOAD    al, $MESSAGE_SEND_BASE
LOAD    al, (a)
LOAD    ah, r4

LOAD    r3, (r2, a)

; If the message buffer (zero terminated) has been sent. Stop sending
CMPI    r3, #0
BRAN|== @stop_send

LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_SEND_DATA
STOR    (a), r3

; Increment by two because of lack of packing (a word actually is padded to a DW)
ADDI    r2, #2
LOAD    ah, $SCRATCH_SEGMENT
LOAD    al, $MESSAGE_SEND_POINTER
STOR    (a), r2

LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_SEND_PENDING
LOAD    r1, $TRUE
STOR    (a), r1

RETS

:stop_send
LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_SEND_ENABLED
LOAD    r1, $FALSE
STOR    (a), r1

RETS
