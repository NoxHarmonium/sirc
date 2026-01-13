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

:setup_serial

LOAD    r1, #9600
LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_BAUD
; TODO: Allow omitting the #0 offset when not using an offset (infer the #0)
STOR    (#0, a), r1

RETS

:print

LOAD    ah, $SCRATCH_SEGMENT
LOAD    al, $MESSAGE_SEND_BASE
STOR    (#0, a), r1

; Align pointer
LOAD    r1, #1

LOAD    ah, $SCRATCH_SEGMENT
LOAD    al, $MESSAGE_SEND_POINTER
STOR    (#0, a), r1

LOAD    r1, $TRUE
LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_SEND_ENABLED
STOR    (#0, a), r1

:wait_for_print_finish

; Wait for exception (will spin until interrupted)
COPI    r1, #0x1900

LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_SEND_ENABLED
LOAD    r1, (#0, a)

CMPI    r1, $TRUE
BRAN|== @wait_for_print_finish

RETS

:exception_handler_p3

; Save the used registers
STOR -(#0, s), lh
STOR -(#0, s), ll
STOR -(#0, s), ah
STOR -(#0, s), al
STOR -(#0, s), r1
STOR -(#0, s), r2
STOR -(#0, s), r3

BRSR @write_pending_byte

; Restore the used registers
LOAD r3, (#0, s)+
LOAD r2, (#0, s)+
LOAD r1, (#0, s)+
LOAD al, (#0, s)+
LOAD ah, (#0, s)+
LOAD ll, (#0, s)+
LOAD lh, (#0, s)+

RETE

:write_pending_byte

; Check if the device is ready to write
LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_SEND_PENDING
LOAD    r1, (#0, a)

; Return early if not (there is already a send pending)
CMPI    r1, $TRUE
RETS|==

LOAD    ah, $SCRATCH_SEGMENT
LOAD    al, $MESSAGE_SEND_POINTER
LOAD    r2, (#0, a)
LOAD    al, $MESSAGE_SEND_BASE
LOAD    al, (#0, a)

LOAD    ah, $PROGRAM_SEGMENT
LOAD    r3, (r2, a)

; If the message buffer (zero terminated) has been sent. Stop sending
CMPI    r3, #0
BRAN|== @stop_send

LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_SEND_DATA
STOR    (#0, a), r3

; Increment by two because of lack of packing (a word actually is padded to a DW)
ADDI    r2, #2
LOAD    ah, $SCRATCH_SEGMENT
LOAD    al, $MESSAGE_SEND_POINTER
STOR    (#0, a), r2

LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_SEND_PENDING
LOAD    r1, $TRUE
STOR    (#0, a), r1

RETS

:stop_send
LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_SEND_ENABLED
LOAD    r1, $FALSE
STOR    (#0, a), r1

RETS
