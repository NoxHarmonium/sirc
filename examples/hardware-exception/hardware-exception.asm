; Reserved space for 128x32 bit exception vectors


.EQU $FALSE                         #0x0
.EQU $TRUE                          #0x1

; When this character is typed, the program should exit
.EQU $EXPECTED_CHAR                 #0x61

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


;; Exception Vectors
; First vector is the reset vector
.ORG 0x0000
.DQ @start

.ORG 0x0080
.DQ @exception_handler_p3

;;;;;;; MAIN CODE SECTION

.ORG 0x0200
:start

; Enable all hardware interrupts (set bits 9-13 of SR)
ORRI sr, #0b0001_1110_0000_0000

; Setup routines
BRSR @setup_serial
BRSR @print_help
; Make sure this is after the initial help message print (see the subroutine for more info)
BRSR @enable_serial_recv

:wait_for_char
; Wait for exception (will spin until interrupted)
WAIT

; Pending byte should be in r7 after exeception handler runs
CMPI    r7, $EXPECTED_CHAR
BRAN|== @finish

BRAN @wait_for_char

:setup_serial

LOAD    r1, #9600
LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_BAUD
STOR    (a), r1

RETS

:enable_serial_recv

; When piping data to stdin, an EOF character gets sent which closes stdin
; This means that we can't trigger any more interrupts manually after the data is piped
; (stdin is currently the only way to externally trigger an interrupt)
; If the data is coming in at the same type we are sending data out,
; it ends up with the program gets stuck waiting for an interrupt and never finishes
; TODO: Actually that shouldn't be a problem - better investigate this further (
;   maybe the exception handler routines are conflicting with each other)

LOAD    r1, #0x1
LOAD    al, $SERIAL_DEVICE_RECV_ENABLED
STOR    (a), r1

RETS

:print_help

LOAD    r1, @help_message
BRAN    @print

:print_exit

LOAD    r1, @exit_message
BRAN    @print

:print

LOAD    ah, $SCRATCH_SEGMENT
LOAD    al, $MESSAGE_SEND_BASE
STOR    (a), r1

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

LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_SEND_ENABLED
LOAD    r1, (a)

CMPI    r1, $TRUE
BRAN|== @wait_for_print_finish

RETS

:finish

BRSR    @print_exit

; Halt CPU
COPI    r1, #0b0001_0100_1111_1111

;;;;;;; EXCEPTION HANDLERS

.ORG 0x0400
:exception_handler_p3

; Save the link register
LDEA s, (l)

BRSR @read_pending_byte
BRSR @write_pending_byte

; Restore the link register
LDEA l, (s)

RETE

:read_pending_byte
; Check if there is something we need to read
LOAD    ah, $SERIAL_DEVICE_SEGMENT
LOAD    al, $SERIAL_DEVICE_RECV_PENDING
LOAD    r1, (a)

; Return early if not
CMPI    r1, $FALSE
RETS|==

; Read pending byte
LOAD    al, $SERIAL_DEVICE_RECV_DATA
LOAD    r7, (a)

LOAD    al, $SERIAL_DEVICE_RECV_PENDING
LOAD    r1, $FALSE
STOR    (a), r1
RETS

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
LOAD    al, $MESSAGE_SEND_BASE
LOAD    al, (a)

LOAD    ah, $PROGRAM_SEGMENT
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
