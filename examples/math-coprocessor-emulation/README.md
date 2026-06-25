# Integer Maths Coprocessor Emulation

Reference source listing for a supervisor invalid-opcode handler that emulates one optional integer maths coprocessor operation.

The program enters protected mode and executes several `MULU` examples. On CPU models without coprocessor 3, the invalid-opcode handler runs in supervisor mode, dispatches from the command stored in the fault metadata register, retargets the saved return state to a normal supervisor-mode trampoline, and returns from the fault. The trampoline calculates the unsigned product with a software shift-and-add routine, then uses a software-exception return gate so the original protected-mode status register and program counter are restored atomically.

This listing is intended to match the manual's distribution-media style reference material while still being runnable by the project test tools.
