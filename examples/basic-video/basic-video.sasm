.EQU $FALSE                         #0x0
.EQU $TRUE                          #0x1
.EQU $MAX_FRAMES                    #60

;; Segments
.EQU $PROGRAM_SEGMENT               #0x0000
.EQU $SCRATCH_SEGMENT               #0x0001

;; Devices

; Video
.EQU $VIDEO_DEVICE_SEGMENT         #0x000C
; Video device mapping
; 0x0000..0x00FF => ppu_registers
; 0x6000..0x6100 => palette (256 entries of CGRAM)
; 0x8000..0xFFFF => VRAM (32767 words / 65534 bytes / 262136 pixels)

; PPU Registers (Offsets in segment 0x000C)
.EQU $PPU_BASE_CONFIG              #0x0000
.EQU $PPU_TILE_SIZE                #0x0001
.EQU $PPU_BG1_TILEMAP_ADDR         #0x0002
.EQU $PPU_BG2_TILEMAP_ADDR         #0x0003
.EQU $PPU_BG3_TILEMAP_ADDR         #0x0004
.EQU $PPU_BG1_TILE_ADDR            #0x0006
.EQU $PPU_BG2_TILE_ADDR            #0x0007
.EQU $PPU_BG3_TILE_ADDR            #0x0008
.EQU $PPU_BG1_PALETTE_CONFIG       #0x0012
.EQU $PPU_BG2_PALETTE_CONFIG       #0x0013
.EQU $PPU_BG3_PALETTE_CONFIG       #0x0014

; VRAM Layout
.EQU $CGRAM_PALETTE_BASE           #0x6000
; Start tileset at the bottom of the VRAM to allow room to grow
.EQU $VRAM_TILESET_BASE            #0x8000
; Tilemaps can live in 0xF000-0xFFFF to give the tilesets some breathing room
.EQU $VRAM_BG1_TILEMAP_BASE        #0xF000
.EQU $VRAM_BG2_TILEMAP_BASE        #0xF400
.EQU $VRAM_BG3_TILEMAP_BASE        #0xF800

;; Scratch Variables
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

; Enable all hardware interrupts (set bits 9-13 of SR)
ORRI sr, #0b0001_1110_0000_0000

; Setup stack frame - stack works backwards from end of segment
LOAD sh, $SCRATCH_SEGMENT
LOAD sl, #0xFFFF

; Setup routines
BRSR @setup_serial
BRSR @setup_video

:wait_for_interrupt

; Wait for exception (will spin until interrupted)
WAIT

; Exit when MAX_FRAMES have been rendered so it doesn't loop forever
LOAD    ah, $SCRATCH_SEGMENT
LOAD    al, $FRAME_COUNTER
LOAD    r1, (a)
CMPI    r1, $MAX_FRAMES
BRAN|>= @finish

BRAN @wait_for_interrupt

:finish

BRSR    @print_exit

; Halt CPU
COPI    r1, #0x14FF

:print_exit

LOAD    r1, @exit_message
BRAN    @print


:print_delimiter

LOAD    r1, @delimiter
BRAN    @print


;;; EXCEPTIONS

:exception_handler_p5

; Save the used registers
STOR -(s), lh
STOR -(s), ll
STOR -(s), ah
STOR -(s), al
STOR -(s), r1

; TODO: Make sure exception handlers don't stomp all over the registers

; Increment frame count
LOAD    ah, $SCRATCH_SEGMENT
LOAD    al, $FRAME_COUNTER
LOAD    r1, (a)
ADDI    r1, #1
STOR    (a), r1

; Restore the used registers
LOAD r1, (s)+
LOAD al, (s)+
LOAD ah, (s)+
LOAD ll, (s)+
LOAD lh, (s)+

RETE

;;;;;;; Subroutines

:setup_video
; Save link register
LDEA s, (l)

; 1. Copy Tileset (10224 words)
LOAD r1, @tileset_0.l
LOAD r2, @tileset_0.u
LOAD r3, $VRAM_TILESET_BASE
LOAD r4, $VIDEO_DEVICE_SEGMENT
; TODO: Embed the number of words in the generated data so we don't need to hardcode this
LOAD r5, #6176
BRSR @memcpy

BRSR @print_delimiter

; 2. Copy Tilemaps (1024 words each)
; BG1
LOAD r1, @tilemap__0_2.l
LOAD r2, @tilemap__0_2.u
LOAD r3, $VRAM_BG1_TILEMAP_BASE
LOAD r4, $VIDEO_DEVICE_SEGMENT
LOAD r5, #1024
BRSR @memcpy

BRSR @print_delimiter

; BG2
LOAD r1, @tilemap__0_0.l
LOAD r2, @tilemap__0_0.u
LOAD r3, $VRAM_BG2_TILEMAP_BASE
LOAD r4, $VIDEO_DEVICE_SEGMENT
LOAD r5, #1024
BRSR @memcpy

BRSR @print_delimiter

; BG3
LOAD r1, @tilemap__0_1.l
LOAD r2, @tilemap__0_1.u
LOAD r3, $VRAM_BG3_TILEMAP_BASE
LOAD r4, $VIDEO_DEVICE_SEGMENT
LOAD r5, #1024
BRSR @memcpy

BRSR @print_delimiter

; 3. Copy Palette (9 words)
LOAD r1, @palette__0_0.l
LOAD r2, @palette__0_0.u
LOAD r3, $CGRAM_PALETTE_BASE
LOAD r4, $VIDEO_DEVICE_SEGMENT
LOAD r5, #14
BRSR @memcpy

BRSR @print_delimiter

; 4. Setup PPU Registers
LOAD ah, $VIDEO_DEVICE_SEGMENT

; Tile Size (all 8x8, all Single tilemap size)
LOAD al, $PPU_TILE_SIZE
LOAD r1, #0
STOR (a), r1

; BG Tilemap Addresses
LOAD al, $PPU_BG1_TILEMAP_ADDR
LOAD r1, $VRAM_BG1_TILEMAP_BASE
STOR (a), r1

LOAD al, $PPU_BG2_TILEMAP_ADDR
LOAD r1, $VRAM_BG2_TILEMAP_BASE
STOR (a), r1

LOAD al, $PPU_BG3_TILEMAP_ADDR
LOAD r1, $VRAM_BG3_TILEMAP_BASE
STOR (a), r1

; BG Tile Data Addresses
LOAD al, $PPU_BG1_TILE_ADDR
LOAD r1, $VRAM_TILESET_BASE
STOR (a), r1

LOAD al, $PPU_BG2_TILE_ADDR
LOAD r1, $VRAM_TILESET_BASE
STOR (a), r1

LOAD al, $PPU_BG3_TILE_ADDR
LOAD r1, $VRAM_TILESET_BASE
STOR (a), r1

; pub struct PaletteRegister {
;    pub palette_size: PaletteSize, // Bits 0-1
;    pub reserved: B6,              // Bits 2-7
;    pub palette_offset: u8,        // Bits 8-15
; }
; Size 16 is 2. (0 << 8) | 2 = 0x0002
LOAD r1, #0x0002

LOAD al, $PPU_BG1_PALETTE_CONFIG
STOR (a), r1

LOAD al, $PPU_BG2_PALETTE_CONFIG
STOR (a), r1

LOAD al, $PPU_BG3_PALETTE_CONFIG
STOR (a), r1

; Base Config (Enable all BG and Sprites, Brightness 15)
; Brightness 15 = 0xF << 3 = 0x0078
; All disable bits = 0
LOAD al, $PPU_BASE_CONFIG
LOAD r1, #0x0078
STOR (a), r1

; Restore link register
LDEA l, (s)
RETS

; r1 = source address (offset)
; r2 = source segment
; r3 = destination address (offset)
; r4 = destination segment
; r5 = length (words)
; One day replace this with DMA
:memcpy
    CMPI r5, #0
    RETS|==

    ; Align source pointer because the toolchain packs words in the binary as double words where the higher word is always zero.
    ; A fix for that is in the works
    ; The destination pointer doesn't need the same treatment
    ADDI r1, #1
    ; Set lower byte to zero and use register displacement addressing to reduce number of instructions
    LOAD al, #0


:memcpy_loop
    ; Load from source
    LOAD ah, r2
    LOAD r6, (r1, a)

    ; Store to destination
    LOAD ah, r4
    STOR (r3, a), r6

    ; Increment pointers
    ; Add 2 to the source pointer because the toolchain packs words in the binary as double words where the higher word is always zero.
    ; A fix for that is in the works
    ; The destination pointer doesn't need the same treatment
    ADDI r1, #2
    ADDI r3, #1
    SUBI r5, #1
    BRAN|>> @memcpy_loop

    RETS

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


:delimiter
.DW #61
.DW #61
.DW #61
.DW #61
.DW #61
.DW #61
.DW #61
.DW #0

; TODO: Why do I need this here to make this file parse???
NOOP
