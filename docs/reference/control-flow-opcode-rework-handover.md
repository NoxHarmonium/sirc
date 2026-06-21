# Control-Flow Opcode Rework Handover

Date: 2026-06-21

## Summary

This handover covers the proposed cleanup of SIRC-1 control-flow opcodes and the related `0x1_` opcode-map regularisation.

The current ISA has real `BRAN`, `BRSR`, and `LJSR` opcodes, but their architectural behavior overlaps with more general effective-address operations. Now that protected mode handles all address-register-pair control-flow writes by preserving the high word, the original privilege distinction between branch and long-jump forms no longer appears necessary.

The proposed direction is:

- Keep `LDEA` as the primitive "load effective address" instruction.
- Rename the real link-writing effective-address instruction to `LDEL` ("Load Effective Address and Link").
- Make `BRAN`, `BRSR`, `LJMP`, and `LJSR` assembler conveniences rather than real CPU opcodes.
- Reuse the freed opcodes `0x1A`, `0x1B`, `0x1E`, and `0x1F` as auto-update forms of `LDEA` and `LDEL`, following the existing `STOR`/`LOAD` bit pattern.
- Document aliased register writes as undefined behavior unless the hardware implementation later proves a stable behavior worth specifying. Rejecting those forms in the assembler is desirable but can be deferred.

## Current Opcode Map

```text
0x10  STOR  indirect immediate
0x11  STOR  indirect register
0x12  STOR  pre-decrement immediate
0x13  STOR  pre-decrement register
0x14  LOAD  indirect immediate
0x15  LOAD  indirect register
0x16  LOAD  post-increment immediate
0x17  LOAD  post-increment register
0x18  LDEA  indirect immediate
0x19  LDEA  indirect register
0x1A  BRAN  immediate displacement
0x1B  BRAN  register displacement
0x1C  LJSR  immediate displacement
0x1D  LJSR  register displacement
0x1E  BRSR  immediate displacement
0x1F  BRSR  register displacement
```

## Proposed Opcode Map

Decision: reuse the existing `STOR`/`LOAD` auto-update pattern so the hardware can share the same direction-selection logic.

```text
0x10  STOR  indirect immediate
0x11  STOR  indirect register
0x12  STOR  pre-decrement immediate
0x13  STOR  pre-decrement register
0x14  LOAD  indirect immediate
0x15  LOAD  indirect register
0x16  LOAD  post-increment immediate
0x17  LOAD  post-increment register
0x18  LDEA  indirect immediate
0x19  LDEA  indirect register
0x1A  LDEA  pre-decrement immediate
0x1B  LDEA  pre-decrement register
0x1C  LDEL  indirect immediate
0x1D  LDEL  indirect register
0x1E  LDEL  post-increment immediate
0x1F  LDEL  post-increment register
```

## Rejected Alternative

A previous option was to make all new `LDEA`/`LDEL` auto-update forms use the same direction. That is no longer preferred because it would not directly reuse the existing `0x12`, `0x13`, `0x16`, and `0x17` pattern.

## Rationale

`LDEA` is the real effective-address primitive. `LDEL` is the real effective-address-and-link primitive: it writes the normal return address to `l` and writes the computed effective address to its destination address-register pair.

The control-flow mnemonics become assembler conveniences over those primitives:

```asm
LJMP src         ; alias for LDEA p, (#0, src)
LJSR src         ; alias for LDEL p, (#0, src)
BRAN @label      ; alias for LDEA p, (#offset, p)
BRSR @function   ; alias for LDEL p, (#offset, p)
```

Reusing the opcodes as auto-update forms makes the `0x1_` table more regular and potentially gives optimizers useful addressing idioms, even if the direction split limits the most obvious cursor-copy use case.

With the selected direction pattern, the main `LDEA` idiom is stack/allocation shaped:

```asm
LDEA a, -(#0, s) ; s = s - 1; a = new s
```

The post-increment idiom remains useful for `LDEL`, though the exact optimization value depends on the program:

```asm
LDEL p, (#0, a)+ ; l = next p; p = old a; a = a + 1
```

## Resolved Design Decisions

These decisions were made during review.

- `0x1A` and `0x1B` become `LDEA` pre-decrement immediate/register forms.
- `0x1E` and `0x1F` become `LDEL` post-increment immediate/register forms.
- Auto-update syntax should match existing `LOAD` and `STOR` syntax.
- `LDEL` is the real mnemonic for "Load Effective Address and Link".
- `LJSR` becomes an assembler alias for `LDEL` with destination `p`.
- `LJMP` remains an assembler alias for `LDEA` with destination `p`.
- `BRAN` and `BRSR` remain accepted by the assembler permanently as common PC-relative shorthand.
- `BRAN` and `BRSR` aliases are limited to PC-relative immediate/label forms.
- `BRAN @label` and `BRSR @label` lower to `LDEA`/`LDEL` forms using `p` and keep linker `RefType::Offset` behavior.
- Old manually encoded `BRAN/BRSR` opcodes immediately take their new meanings; no transition or deprecation period is needed because the manual is still draft.
- The manual should present the final architecture directly rather than describing old `BRAN/BRSR` real opcodes as removed or deprecated.
- `LDEL` should be documented as a mirror of `LDEA` that also writes the return address to `l`.
- `LDEL` should support explicit destination address-register forms matching `LDEA`.
- The common control-flow forms are `BRAN`, `BRSR`, `LJMP`, and `LJSR`, because they all default destination/source behavior around `p`.
- Explicit-destination `LDEL` is neither encouraged nor discouraged; it is a normal documented option for programmers who want it.
- Aliased register writes are undefined behavior. Rejecting them in the assembler is preferred but deferred with a TODO.
- Undefined aliasing covers both direct register-half overlap and whole address-register-pair overlap, unless later implementation experience shows that this makes the ISA impractical.
- Protected mode masks all high address-register writes, including auto-updated source address-register writes.
- Conditional-false auto-update forms skip all side effects, including destination write-back, source address-register update, link-register update, and memory access. This matches the existing LOAD/STOR stage behavior.
- Segment-overflow faults suppress write-back side effects. This matches the existing effective-address behavior: the fault is raised during effective-address execution and later phases abort while a pending fault exists.
- `LDEL` auto-update forms should be documented. If hardware proves too difficult, the architecture can publish an addendum later.

## Remaining Implementation Ambiguities

The main design questions are now resolved. Remaining choices are implementation details:

- What exact TODO/diagnostic text should be used for deferred aliased-register-write rejection?
- Should assembler tests assert current permissiveness for aliased register writes until rejection is implemented, or simply add TODO comments near the parser cases?

## Aliased Register Writes

Any instruction that would write the same architectural register through more than one write-back path has undefined behavior unless the instruction description explicitly defines the result.

Examples to treat as undefined:

```asm
LDEA a, (#0, a)+
LDEA p, (#0, p)+
LDEL l, (#0, a)
LOAD al, (#0, a)+
STOR -(#0, a), al
```

Rationale: the eventual hardware may overlay write-enable/data paths in a way that is not equivalent to the simulator's current ordered writes. The ISA should not promise deterministic behavior until the hardware implementation is known.

Assemblers and compiler backends should avoid generating these forms. The assembler should eventually reject them; this can be deferred with an implementation TODO if needed.

## Known Existing Bug

The current `LJSR` assembler parser likely encodes immediate-form `LJSR` incorrectly.

Observed issue:

- CPU tests construct `LongJumpToSubroutine*` instructions with destination `p`.
- The write-back stage appears to use the destination address-register field for subroutine control-flow target write-back.
- `sirc-vm/toolchain/src/parsers/opcodes/ljsr.rs` currently sets the immediate-form destination field to `0x0` and comments it as unused.

Expected fix:

- Introduce `LDEL` as the real parser/emitted instruction.
- `LDEL dest, (#offset, src)` should encode explicit destination `dest`, source `src`, immediate offset.
- `LDEL dest, (rN, src)` should encode explicit destination `dest`, source `src`, register offset.
- `LJSR a` should become an alias that encodes `LDEL p, (#0, a)`.
- `LJSR a, #offset` should become an alias that encodes `LDEL p, (#offset, a)`.
- `LJSR a, rN` should become an alias that encodes `LDEL p, (rN, a)`.
- Parser tests should verify these fields.

## LDEL Syntax and Control-Flow Aliases

`LDEL` should mirror `LDEA` and additionally write the return address to `l`.

Real `LDEL` forms:

```asm
LDEL dest, (#offset, src)
LDEL dest, (rN, src)
LDEL dest, (#offset, src)+
LDEL dest, (rN, src)+
```

Control-flow aliases:

```asm
LJMP src          ; LDEA p, (#0, src)
LJMP src, #offset ; LDEA p, (#offset, src)
LJMP src, rN      ; LDEA p, (rN, src)

LJSR src          ; LDEL p, (#0, src)
LJSR src, #offset ; LDEL p, (#offset, src)
LJSR src, rN      ; LDEL p, (rN, src)

BRAN @label       ; LDEA p, (#offset, p)
BRSR @label       ; LDEL p, (#offset, p)
```

The alias forms are the common flow-control cases. Explicit `LDEL` forms are available when the programmer wants a link-writing effective-address operation that targets an address register other than `p`.

## Implementation Plan

This should be handled as a staged change. Avoid doing the simulator, assembler, linker, examples, and manual rewrite in one unreviewable diff.

### Phase 1: Lock the Design Contract

1. Keep this handover as the source of truth for the rework.
2. Confirm final syntax for `LDEL`, `LJMP`, `LJSR`, `BRAN`, and `BRSR`.
3. Decide where TODO-backed aliased-register-write rejection should live: parser, validation pass, or both.

### Phase 2: Parser and Encoding Tests

1. Add parser tests for real `LDEL` forms:
   - `LDEL dest, (#offset, src)`
   - `LDEL dest, (rN, src)`
   - `LDEL dest, (#offset, src)+`
   - `LDEL dest, (rN, src)+`
2. Add parser tests for alias forms:
   - `LJMP src`
   - `LJSR src`
   - `BRAN @label`
   - `BRSR @label`
3. Add TODO-backed diagnostics for aliased register writes, or implement rejection immediately.

### Phase 3: Simulator Opcode Rename

1. Rename `LongJumpToSubroutine*` CPU enum variants to `LoadEffectiveAddressAndLink*` or equivalent `LDEL` names while keeping opcode values stable for `0x1C` and `0x1D`.
2. Rename or replace CPU enum variants for `0x1A`, `0x1B`, `0x1E`, and `0x1F`.
3. Update fetch/decode tests and encoding round-trip opcode lists.
4. Keep behavior-preserving tests for old `LJSR`-style manually constructed instructions until alias/parser tests cover the replacement surface.

### Phase 4: Execution and Write-Back Semantics

1. Update effective-address execution so new auto-update forms compute both the target address and source address-register update.
2. Update write-back so new `LDEA` auto-update forms write destination address pair plus updated source pair.
3. Update write-back so new `LDEL` auto-update forms write link, destination address pair, and updated source pair.
4. Preserve protected-mode high-word masking for all address-pair write-back paths.
5. Preserve current conditional-false and segment-overflow side-effect behavior.

### Phase 5: Alias Lowering and Linker Behavior

1. Convert assembler `LJMP` parsing to emit `LDEA p, ...`.
2. Convert assembler `LJSR` parsing to emit `LDEL p, ...`.
3. Convert assembler `BRAN` parsing to emit `LDEA p, (#offset, p)` for immediate/label PC-relative forms only.
4. Convert assembler `BRSR` parsing to emit `LDEL p, (#offset, p)` for immediate/label PC-relative forms only.
5. Preserve linker support for PC-relative `RefType::Offset` label references.

### Phase 6: Documentation and Examples

1. Update opcode map, instruction summary, addressing modes, control-flow chapter, and undefined-behavior appendix.
2. Replace assembly examples that treat `LJSR` as a real primitive with either `LDEL` or an alias form, depending on context.
3. Ensure examples keep using `@label` for symbolic label references.
4. Regenerate any generated encoding/opcode tables.
5. Rebuild the reference manual.

## Test Plan

CPU/simulator tests:

- `LDEA p, (#offset, p)` matches old `BRAN #offset` behavior.
- `LDEL p, (#offset, p)` matches old `BRSR #offset` behavior.
- New `LDEA` pre-decrement immediate form updates destination and source pair for non-aliasing registers.
- New `LDEA` pre-decrement register form updates destination and source pair for non-aliasing registers.
- New `LDEL` post-increment immediate/register forms update link, explicit destination, and source pair.
- `LJSR` alias forms update link and program counter through emitted `LDEL p, ...`.
- Protected mode preserves all high words touched through address-pair write-back.
- Conditional-false forms produce no write-back.
- Segment-overflow faults suppress write-back side effects.

Assembler/parser tests:

- `LDEL dest, (#offset, src)` and `LDEL dest, (rN, src)` encode explicit destination forms matching `LDEA`.
- `LJSR a`, `LJSR a, #offset`, and `LJSR a, rN` encode `LDEL` with destination `p`.
- `BRAN @label` lowers to an `LDEA p, ...` form with `RefType::Offset`.
- `BRSR @label` lowers to an `LDEL p, ...` form with `RefType::Offset`.
- New `LDEA` auto-update syntax encodes `0x1A/0x1B`.
- New `LDEL` auto-update syntax encodes `0x1E/0x1F`.
- Aliased register writes have TODO-backed diagnostics or are rejected according to the chosen assembler policy.

Documentation tests:

- Generated opcode tables include the new `0x1_` meanings.
- Legal forms tables show aliases separately from real opcodes.
- `LDEL` is documented as the real link-writing effective-address instruction.
- `LJSR` and `BRSR` are documented as aliases.
- Examples use `@label` for symbolic label references.
- Undefined behavior rules are stated consistently in the instruction chapter and appendix.

Minimum commands:

```text
cargo test -p peripheral_cpu --test mod
cargo test -p toolchain
make -C docs/reference
```

## Documentation Update Checklist

- `docs/reference/chapters/11-instruction-summary.tex`
- `docs/reference/chapters/14-control-flow.tex`
- `docs/reference/chapters/07-addressing-modes.tex`
- `docs/reference/chapters/appendix-a-opcode-map.tex`
- `docs/reference/chapters/appendix-c-undocumented.tex`
- `docs/reference/manual-handover.md`
- Generated opcode/encoding tables, if applicable

## Recommended Review Questions

- Does the proposed `LDEL` syntax parse cleanly alongside `LJSR` shorthand alias forms?
- Should the alias-rejection TODO live in the parser, a validation pass, or both?
