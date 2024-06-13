.EQU $FALSE                         #0x0
.EQU $TRUE                          #0x1
.EQU $MAX_FRAMES                    #60

;; Segments
.EQU $PROGRAM_SEGMENT               #0x0000
.EQU $SCRATCH_SEGMENT               #0x0001

;; Devices

; Video
.EQU $VIDEO_DEVICE_SEGMENT         #0x000C

;; Scratch Variables
; TODO: Add way to share global constants between asm files
.EQU $FRAME_COUNTER                 #0x0002

;; Exception Vectors
; First vector is the reset vector
.ORG 0x0000
.DQ @start

; v_sync interrupt (l4/p5)
.ORG 0x0040
.DQ @exception_handler_p5

; Serial interrupt
.ORG 0x0080
.DQ @exception_handler_p3

;;;;;;; MAIN CODE SECTION

.ORG 0x0200
:start

; Setup routines
BRSR @setup_serial

:wait_for_interrupt

; Wait for exception (will spin until interrupted)
COPI    r1, #0x1900

; Exit when 60 frames have been rendered so it doesn't loop forever
LOAD    ah, $SCRATCH_SEGMENT
LOAD    al, $FRAME_COUNTER
LOAD    r1, (#0, a)
CMPI    r1, $MAX_FRAMES
BRAN|>= @finish

; TODO: Uncomment when program will actually terminate
BRAN @wait_for_interrupt

:finish

BRSR    @print_exit

; Halt CPU
COPI    r1, #0x14FF

:print_exit

LOAD    r1, @exit_message
BRAN    @print

;;; EXCEPTIONS

:exception_handler_p5

; Save the link register
LDEA s, (#0, l)

; Increment frame count
LOAD    ah, $SCRATCH_SEGMENT
LOAD    al, $FRAME_COUNTER
LOAD    r1, (#0, a)
ADDI    r1, #1
STOR    (#0, a), r1

; Restore the link register
LDEA l, (#0, s)

RETE

;;;;;;; DATA

; TODO: A string data type that does this for us
:exit_message
.DW #71
.DW #111
.DW #111
.DW #100
.DW #98
.DW #121
.DW #101
.DW #33
.DW #10
.DW #0

; TODO: Why do I need this here to make this file parse???
NOOP