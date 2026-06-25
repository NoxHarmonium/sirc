# Integer Maths Coprocessor Emulation

Reference source listing for a supervisor invalid-opcode handler that emulates one optional integer maths coprocessor operation.

The program enters protected mode and executes `MULU`. On CPU models without coprocessor 3, the invalid-opcode handler runs in supervisor mode, calculates the unsigned product for the fixture operands, and returns to the protected program as though the hardware operation had completed.

This listing is intended to match the manual's distribution-media style reference material while still being runnable by the project test tools.
