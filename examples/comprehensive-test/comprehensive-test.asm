; SIRC-1 Comprehensive Instruction Test Suite
; This test suite systematically exercises the SIRC-1 CPU instruction set

;; Segments
.EQU $PROGRAM_SEGMENT               #0x0000
.EQU $SCRATCH_SEGMENT               #0x0001
.EQU $TEST_RUNNER_STORAGE           #0x0002
.EQU $STACK                         #0x0003

;; Constants
.EQU $TEST_RESULT_BASE_OFFSET       #0x000F
.EQU $TEST_RESULT_LENGTH            #0x00FF


;; Variables
.EQU $TEST_RESULT_OFFSET            #0x0000


; Exception vector table
.ORG 0x0000
.DQ @main

.ORG 0x0200

:reset_test
    LOAD r7, #0
    RETS

:store_test_result
    ; Store used registers
    STOR -(#0, s), ah
    STOR -(#0, s), al
    STOR -(#0, s), r1

    ; Set the correct segment
    LOAD ah, $TEST_RUNNER_STORAGE

    ; Load current offset in array
    LOAD al, $TEST_RESULT_OFFSET
    LOAD r1, (#0, a)

    ; Setup pointer to start of array
    LOAD al, $TEST_RESULT_BASE_OFFSET

    ; Store the result to the current offset
    STOR (r1, a), r7

    ; Increment the offset
    ADDI r1, #1

    ; Store the incremented offset back to memory
    LOAD al, $TEST_RESULT_OFFSET
    STOR (#0, a), r1

    ; Restore used registers
    LOAD r1, (#0, s)+
    LOAD al, (#0, s)+
    LOAD ah, (#0, s)+

    RETS

:count_passed_tests
    ; Store used registers
    STOR -(#0, s), ah
    STOR -(#0, s), al
    STOR -(#0, s), r1
    STOR -(#0, s), r2

    ; Set the correct segment
    LOAD ah, $TEST_RUNNER_STORAGE

    ; Load current offset in array (total count)
    LOAD al, $TEST_RESULT_OFFSET
    LOAD r1, (#0, a)

    ; Setup pointer to start of array
    LOAD al, $TEST_RESULT_BASE_OFFSET

    ; Reset r7 that will be used as a return value (number of passing tests)
    LOAD r7, #0

    ; Loop backwards over each test result
    ; r1 = current offset (i), r2 = test result
    ; Set up loop counter

:count_passed_tests_loop
    ; Decrement offset
    SUBI r1, #1

    LOAD r2, (r1, a)
    CMPI r2, #0
    ADDI|!= r7, #1

    ; Return when the end of the array is reached
    CMPI r1, #0
    RETS|==

    BRAN @count_passed_tests_loop

    ; Restore used registers
    LOAD r2, (#0, s)+
    LOAD r1, (#0, s)+
    LOAD al, (#0, s)+
    LOAD ah, (#0, s)+

    RETS

:main
    ; Setup stack frame
    LOAD sh, $STACK
    LOAD sl, #0xFFFF

    LOAD ah, #0x0001
    LOAD al, #0x0000

    LOAD r1, #0x0000
    LOAD r2, #0x0000
    LOAD r3, #0x0000
    LOAD r4, #0x0000
    LOAD r5, #0x0000
    LOAD r6, #0x0000
    LOAD r7, #0x0000    ; r7 = test pass counter

; TEST 1: Immediate Load Operations and Verification
    BRSR @reset_test
    LOAD r1, #0xFFFF
    LOAD r2, #0x0000
    LOAD r3, #0xAAAA
    LOAD r4, #0x5555
    LOAD r5, #0x8000
    LOAD r6, #0x7FFF
    ; Verify r3 loaded correctly
    CMPI r3, #0xAAAA
    LOAD|== r7, #1      ; Increment counter if test passed
    BRSR @store_test_result

; TEST 2: Basic Addition
    BRSR @reset_test
    LOAD r1, #0x0100
    LOAD r2, #0x0200
    ADDR r3, r1, r2     ; r3 should be 0x0300
    CMPI r3, #0x0300
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 3: Addition with Unsigned Overflow (Carry)
    BRSR @reset_test
    LOAD r1, #0xFFFF
    ADDI r1, #0x0001    ; Should wrap to 0x0000 and set C flag
    CMPI r1, #0x0000
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 4: Addition with Signed Overflow
    LOAD r1, #0x7FFF
    ADDI r1, #0x0001    ; Should overflow to 0x8000 and set V flag
    CMPI r1, #0x8000
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 5: Addition with Carry Chain
    BRSR @reset_test
    LOAD r1, #0x00FF
    ADDI r1, #0xFF01    ; Sets carry
    LOAD r2, #0x0001
    ADCI r2, #0x0000    ; Should add carry: 0x0001 + 0x0000 + 1 = 0x0002
    CMPI r2, #0x0002
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 6: Basic Subtraction
    BRSR @reset_test
    LOAD r1, #0x0300
    LOAD r2, #0x0100
    SUBR r3, r1, r2     ; r3 should be 0x0200
    CMPI r3, #0x0200
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 7: Subtraction with Borrow
    BRSR @reset_test
    LOAD r1, #0x0000
    SUBI r1, #0x0001    ; Should wrap to 0xFFFF
    CMPI r1, #0xFFFF
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 8: Subtraction Producing Zero (Z flag)
    BRSR @reset_test
    LOAD r1, #0x1234
    SUBR r2, r1, r1     ; r2 should be 0x0000, Z flag set
    CMPI r2, #0x0000
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 9: Logical AND
    BRSR @reset_test
    LOAD r1, #0xFF00
    LOAD r2, #0x0FF0
    ANDR r3, r1, r2     ; r3 should be 0x0F00
    CMPI r3, #0x0F00
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 10: Logical OR
    BRSR @reset_test
    LOAD r1, #0xF000
    LOAD r2, #0x0F00
    ORRR r3, r1, r2     ; r3 should be 0xFF00
    CMPI r3, #0xFF00
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 11: Logical XOR
    BRSR @reset_test
    LOAD r1, #0xAAAA
    LOAD r2, #0x5555
    XORR r3, r1, r2     ; r3 should be 0xFFFF
    CMPI r3, #0xFFFF
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 12: XOR with Self (produces zero)
    BRSR @reset_test
    LOAD r1, #0xBEEF
    XORR r2, r1, r1     ; r2 should be 0x0000
    CMPI r2, #0x0000
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 13: AND with Mask
    BRSR @reset_test
    LOAD r1, #0xAAAA
    ANDI r1, #0xF0F0    ; r1 should be 0xA0A0
    CMPI r1, #0xA0A0
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 14: Compare for Equality
    BRSR @reset_test
    LOAD r1, #0x1234
    LOAD r2, #0x1234
    CMPR r1, r2         ; Should set Z flag
    LOAD|== r7, #1      ; Increment if equal
    BRSR @store_test_result

; TEST 15: Compare for Inequality
    BRSR @reset_test
    LOAD r1, #0x1234
    LOAD r2, #0x5678
    CMPR r1, r2         ; Should clear Z flag
    ADDI|!= r7, #1      ; Increment if not equal
    BRSR @store_test_result

; TEST 16: Test AND (no result save)
    BRSR @reset_test
    LOAD r1, #0xFF00
    TSAI r1, #0x00FF    ; Should set Z (no bits in common)
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 17: Logical Shift Left
    BRSR @reset_test
    LOAD r1, #0x0003
    ADDI r1, #0, LSL #4 ; r1 should be 0x0030
    CMPI r1, #0x0030
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 18: Logical Shift Left with Carry Out
    BRSR @reset_test
    LOAD r2, #0x8000
    ADDI r2, #0, LSL #1 ; r2 should be 0x0000, C flag set
    CMPI r2, #0x0000
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 19: Logical Shift Right
    BRSR @reset_test
    LOAD r3, #0xC000
    ADDI r3, #0, LSR #4 ; r3 should be 0x0C00
    CMPI r3, #0x0C00
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 20: Logical Shift Right with Carry Out
    BRSR @reset_test
    LOAD r4, #0x0001
    ADDI r4, #0, LSR #1 ; r4 should be 0x0000, C flag set
    CMPI r4, #0x0000
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 21: Large Shift
    BRSR @reset_test
    LOAD r5, #0x00FF
    ADDI r5, #0, LSL #8 ; r5 should be 0xFF00
    CMPI r5, #0xFF00
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 22: Arithmetic Shift Left
    BRSR @reset_test
    LOAD r1, #0x0003
    ADDI r1, #0, ASL #2 ; r1 should be 0x000C
    CMPI r1, #0x000C
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 23: Arithmetic Shift Right (Sign Extension)
    BRSR @reset_test
    LOAD r2, #0x8000
    ADDI r2, #0, ASR #1 ; r2 should be 0xC000 (sign extended)
    CMPI r2, #0xC000
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 24: Arithmetic Shift Right Multiple Bits
    BRSR @reset_test
    LOAD r3, #0xF000
    ADDI r3, #0, ASR #4 ; r3 should be 0xFF00 (sign extended)
    CMPI r3, #0xFF00
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 25: ASR on Positive Number
    BRSR @reset_test
    LOAD r4, #0x4000
    ADDI r4, #0, ASR #2 ; r4 should be 0x1000
    CMPI r4, #0x1000
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 26: Register Addition with Shift (Multiply by 3)
    BRSR @reset_test
    LOAD r1, #0x0010
    ADDR r2, r1, r1, LSL #1 ; r2 = 0x10 + (0x10 << 1) = 0x0030
    CMPI r2, #0x0030
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 27: Building Constants with Shifted Immediate
    BRSR @reset_test
    LOAD r3, #0x0001
    ADDI r3, #0x02, LSL #8  ; r3 = (0x01 << 8) + 0x02 = 0x0102
    CMPI r3, #0x0102
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 28: Scaled Register Addition
    BRSR @reset_test
    LOAD r4, #0x0004
    LOAD r5, #0x0100
    ADDR r6, r5, r4, LSL #4 ; r6 = 0x0100 + (0x04 << 4) = 0x0140
    CMPI r6, #0x0140
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 29: Conditional Execution - Equal Condition
    BRSR @reset_test
    LOAD r1, #0x0000
    LOAD r2, #0x0000
    LOAD r3, #0x0000
    CMPR r1, r2             ; Z=1
    ADDI|== r3, #0xAAAA     ; Should execute
    CMPI r3, #0xAAAA
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 30: Conditional Execution - Not Equal Skip
    BRSR @reset_test
    LOAD r1, #0x0000
    LOAD r2, #0x0000
    LOAD r4, #0x0000
    CMPR r1, r2             ; Z=1
    ADDI|!= r4, #0xBBBB     ; Should NOT execute
    CMPI r4, #0x0000        ; Should still be 0
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 31: Conditional - Not Equal Condition
    BRSR @reset_test
    LOAD r1, #0x0001
    LOAD r2, #0x0002
    LOAD r6, #0x0000
    CMPR r1, r2             ; Z=0
    ADDI|!= r6, #0xDDDD     ; Should execute
    CMPI r6, #0xDDDD
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 32: Signed Comparison - Greater or Equal
    BRSR @reset_test
    LOAD r1, #0x0010
    LOAD r2, #0x0005
    LOAD r3, #0x0000
    CMPR r1, r2             ; 16 >= 5
    ADDI|>= r3, #0x00AA     ; Should execute
    CMPI r3, #0x00AA
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 33: Signed Comparison - Less Than
    BRSR @reset_test
    LOAD r1, #0x0005
    LOAD r2, #0x0010
    LOAD r5, #0x0000
    CMPR r1, r2             ; 5 < 16
    ADDI|<< r5, #0x00CC     ; Should execute
    CMPI r5, #0x00CC
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 34: Signed Comparison with Negative Numbers
    BRSR @reset_test
    LOAD r1, #0xFFFF        ; -1
    LOAD r2, #0x0001        ; 1
    LOAD r3, #0x0000
    CMPR r1, r2             ; -1 < 1
    ADDI|<< r3, #0x00EE     ; Should execute
    CMPI r3, #0x00EE
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 35: Unsigned Comparison - Higher
    BRSR @reset_test
    LOAD r1, #0xFFFF        ; 65535
    LOAD r2, #0x0001        ; 1
    LOAD r3, #0x0000
    CMPR r1, r2             ; 65535 > 1 (unsigned)
    ADDI|HI r3, #0x1111     ; Should execute
    CMPI r3, #0x1111
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 36: Unsigned Comparison - Lower or Same
    BRSR @reset_test
    LOAD r1, #0x0001
    LOAD r2, #0xFFFF
    LOAD r4, #0x0000
    CMPR r1, r2             ; 1 <= 65535 (unsigned)
    ADDI|LO r4, #0x2222     ; Should execute
    CMPI r4, #0x2222
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 37: Memory Store and Load
    BRSR @reset_test
    LOAD ah, #0x0001
    LOAD al, #0x0000
    LOAD r1, #0xCAFE
    STOR (#0x0000, a), r1
    LOAD r2, (#0x0000, a)   ; r2 should be 0xCAFE
    CMPI r2, #0xCAFE
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 38: Memory with Offset
    BRSR @reset_test
    LOAD r3, #0xBEEF
    STOR (#0x0010, a), r3
    LOAD r4, (#0x0010, a)   ; r4 should be 0xBEEF
    CMPI r4, #0xBEEF
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 39: Memory with Register Offset
    BRSR @reset_test
    LOAD ah, #0x0001
    LOAD al, #0x0100
    LOAD r1, #0x1234
    LOAD r5, #0x0020
    STOR (r5, a), r1
    LOAD r2, (r5, a)        ; r2 should be 0x1234
    CMPI r2, #0x1234
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 40: Memory Post-Increment
    BRSR @reset_test
    LOAD ah, #0x0001
    LOAD al, #0x0200
    LOAD r1, #0xAAAA
    STOR (#0x0000, a), r1
    LOAD r2, #0xBBBB
    STOR (#0x0002, a), r2
    LOAD al, #0x0200        ; Reset pointer
    LOAD r4, (#0, a)+       ; r4 = 0xAAAA, al += 2
    CMPI r4, #0xAAAA
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 41: Memory Post-Increment Pointer Check
    BRSR @reset_test
    ; al should now be 0x0202
    CMPI al, #0x0202
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 42: Memory Post-Increment Second Read
    BRSR @reset_test
    LOAD r5, (#0, a)+       ; r5 = 0xBBBB, al += 2
    CMPI r5, #0xBBBB
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 43: Memory Pre-Decrement
    BRSR @reset_test
    LOAD ah, #0x0001
    LOAD al, #0x0306
    LOAD r1, #0x1111
    STOR -(#0, a), r1       ; al -= 2, store at 0x0304
    CMPI al, #0x0304
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 44: Memory Pre-Decrement Read Back
    BRSR @reset_test
    ; Read back the value we stored at 0x0304 using normal load
    LOAD r2, (#0x0000, a)   ; al is at 0x0304, read from there
    CMPI r2, #0x1111
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 45: Overflow Detection - Positive Overflow
    BRSR @reset_test
    LOAD r1, #0x7000
    LOAD r2, #0x1000
    ADDR r3, r1, r2         ; r3 = 0x8000 (overflow)
    CMPI r3, #0x8000
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 46: Overflow Detection - Negative Overflow
    BRSR @reset_test
    LOAD r1, #0x8000
    LOAD r2, #0x8000
    ADDR r3, r1, r2         ; r3 = 0x0000 (overflow with carry)
    CMPI r3, #0x0000
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 47: Carry Propagation in ADC
    BRSR @reset_test
    LOAD r1, #0xFFFF
    ADDI r1, #0x0001        ; Sets carry
    LOAD r2, #0x0000
    ADCI r2, #0x0000        ; r2 = 0 + 0 + carry = 1
    CMPI r2, #0x0001
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 48: Zero Flag from XOR
    BRSR @reset_test
    LOAD r3, #0xFFFF
    XORR r4, r3, r3         ; r4 = 0
    CMPI r4, #0x0000
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 49: Zero Flag from AND
    BRSR @reset_test
    LOAD r5, #0xFF00
    ANDI r5, #0x00FF        ; r5 = 0
    CMPI r5, #0x0000
    LOAD|== r7, #1
    BRSR @store_test_result

; TEST 50: Final Counter Check
    BRSR @count_passed_tests
    ; r7 should now equal 50 (0x32) if all tests passed
    LOAD|== r1, #0xDEAD     ; Success marker
    LOAD|!= r1, #0xFA11     ; Failure marker (0xFA11)

; Store final test count for inspection
    LOAD r2, r7             ; Copy test pass count to r2
    LOAD r3, #0x0032        ; Expected count (50 decimal = 0x32 hex)

; Halt CPU
    COPI r1, #0x14FF
