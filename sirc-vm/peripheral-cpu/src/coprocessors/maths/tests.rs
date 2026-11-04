use super::*;

// Native arithmetic is an independent test oracle, never the simulated datapath.
fn expected(opcode: u16, high: u16, low: u16, divisor: u16) -> (u16, u16, u16) {
    match opcode {
        MULU_OPCODE => {
            let product = u32::from(high) * u32::from(low);
            (
                u16::try_from(product >> WORD_BITS).unwrap(),
                u16::try_from(product & WORD_MASK).unwrap(),
                0,
            )
        }
        MULS_OPCODE => {
            let left = i16::from_ne_bytes(high.to_ne_bytes());
            let right = i16::from_ne_bytes(low.to_ne_bytes());
            let product = u32::from_ne_bytes((i32::from(left) * i32::from(right)).to_ne_bytes());
            (
                u16::try_from(product >> WORD_BITS).unwrap(),
                u16::try_from(product & WORD_MASK).unwrap(),
                0,
            )
        }
        DIVU_OPCODE | DIVS_OPCODE if divisor == 0 => (high, low, ERROR | DIVIDE_BY_ZERO),
        DIVU_OPCODE => {
            let dividend = (u32::from(high) << WORD_BITS) | u32::from(low);
            let quotient = dividend / u32::from(divisor);
            u16::try_from(quotient).map_or((high, low, ERROR | QUOTIENT_OVERFLOW), |quotient| {
                (
                    u16::try_from(dividend % u32::from(divisor)).unwrap(),
                    quotient,
                    0,
                )
            })
        }
        DIVS_OPCODE => {
            let dividend = i64::from(i32::from_ne_bytes(
                ((u32::from(high) << WORD_BITS) | u32::from(low)).to_ne_bytes(),
            ));
            let divisor = i64::from(i16::from_ne_bytes(divisor.to_ne_bytes()));
            i16::try_from(dividend / divisor).map_or(
                (high, low, ERROR | QUOTIENT_OVERFLOW),
                |quotient| {
                    (
                        u16::from_ne_bytes(
                            i16::try_from(dividend % divisor).unwrap().to_ne_bytes(),
                        ),
                        u16::from_ne_bytes(quotient.to_ne_bytes()),
                        0,
                    )
                },
            )
        }
        _ => unreachable!(),
    }
}
fn check(opcode: u16, high: u16, low: u16, divisor: u16) {
    let mut registers = Registers {
        r1: high,
        r2: low,
        r3: divisor,
        r4: 0xabcd,
        sr: 0x00a5,
        ..Registers::default()
    };
    let before = registers;
    let mut unit = MathsCoprocessor::default();
    unit.start(0x3000 | opcode, &mut registers).unwrap();
    assert_eq!(registers, before);
    let mut steps = 0;
    while !unit.is_finished() {
        assert!(steps < 40);
        let out = unit
            .step(ExecutionPhase::InstructionFetchHigh, &mut registers)
            .unwrap();
        assert!(!out.bus_access_strobe);
        steps += 1;
        let work = unit.work.as_ref().unwrap();
        assert!(work.wide <= 0x0001_ffff);
        if !unit.is_finished() {
            assert_eq!(registers, before, "no early architectural result");
        }
    }
    assert!(steps == 5 || steps == 40);
    let result = expected(opcode, high, low, divisor);
    let mut expected_registers = before;
    (
        expected_registers.r1,
        expected_registers.r2,
        expected_registers.r3,
    ) = result;
    assert_eq!(
        registers, expected_registers,
        "opcode={opcode:04x} input={high:04x}:{low:04x}/{divisor:04x}"
    );
}

#[test]
fn arithmetic_matches_an_independent_oracle_at_boundaries_and_across_operand_space() {
    let edges = [
        0, 1, 2, 3, 0x3fff, 0x4000, 0x7ffe, 0x7fff, 0x8000, 0x8001, 0xfffe, 0xffff,
    ];
    for opcode in [MULU_OPCODE, MULS_OPCODE, DIVU_OPCODE, DIVS_OPCODE] {
        for high in edges {
            for low in edges {
                for divisor in edges {
                    check(opcode, high, low, divisor);
                }
            }
        }
        let mut random = 0x1234_5678u32;
        for _ in 0..4096 {
            let mut word = || {
                random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                u16::try_from(random >> WORD_BITS).unwrap()
            };
            check(opcode, word(), word(), word());
        }
    }
}

#[test]
fn every_signed_operand_handles_negation_including_the_most_negative_word() {
    for operand in 0..=u16::MAX {
        check(MULS_OPCODE, operand, 0x8000, 0);
        check(DIVS_OPCODE, 0xffff, operand, 0x8000);
    }
}

#[test]
fn the_datapath_really_advances_one_add_or_shift_per_cycle() {
    let mut registers = Registers {
        r1: 3,
        r2: 5,
        ..Registers::default()
    };
    let mut unit = MathsCoprocessor::default();
    unit.start(0x3000, &mut registers).unwrap();
    for _ in 0..4 {
        unit.step(ExecutionPhase::InstructionFetchHigh, &mut registers)
            .unwrap();
    }
    let work = unit.work.as_ref().unwrap();
    assert_eq!((work.wide, work.low, work.bits_left), (0, 5, 16));
    unit.step(ExecutionPhase::MemoryAccessExecutor, &mut registers)
        .unwrap();
    let work = unit.work.as_ref().unwrap();
    assert_eq!((work.wide, work.low, work.bits_left), (3, 5, 16));
    unit.step(ExecutionPhase::WriteBackExecutor, &mut registers)
        .unwrap();
    let work = unit.work.as_ref().unwrap();
    assert_eq!((work.wide, work.low, work.bits_left), (1, 0x8002, 15));
    assert_eq!((registers.r1, registers.r2), (3, 5));
}

#[test]
fn division_retains_the_shifted_remainder_on_borrow_and_sets_a_quotient_bit_on_success() {
    let mut registers = Registers {
        r1: 1,
        r2: 0,
        r3: 3,
        ..Registers::default()
    };
    let mut unit = MathsCoprocessor::default();
    unit.start(0x3200, &mut registers).unwrap();
    for _ in 0..4 {
        unit.step(ExecutionPhase::InstructionFetchHigh, &mut registers)
            .unwrap();
    }
    unit.step(ExecutionPhase::MemoryAccessExecutor, &mut registers)
        .unwrap();
    assert_eq!(unit.work.as_ref().unwrap().wide, 2);
    unit.step(ExecutionPhase::WriteBackExecutor, &mut registers)
        .unwrap();
    let work = unit.work.as_ref().unwrap();
    assert_eq!((work.high, work.low, work.bits_left), (2, 0, 15));
    unit.step(ExecutionPhase::InstructionFetchLow, &mut registers)
        .unwrap();
    assert_eq!(unit.work.as_ref().unwrap().wide, 4);
    unit.step(ExecutionPhase::InstructionFetchHigh, &mut registers)
        .unwrap();
    let work = unit.work.as_ref().unwrap();
    assert_eq!((work.high, work.low, work.bits_left), (1, 1, 14));
    assert_eq!((registers.r1, registers.r2, registers.r3), (1, 0, 3));
    unit.reset();
    assert!(unit.is_finished());
    assert!(
        !unit
            .step(ExecutionPhase::InstructionDecode, &mut registers)
            .unwrap()
            .bus_access_strobe
    );
    assert_eq!((registers.r1, registers.r2, registers.r3), (1, 0, 3));
    // Reusing the installed hardware must not revive the interrupted divide.
    unit.start(0x3000, &mut registers).unwrap();
    for _ in 0..40 {
        unit.step(ExecutionPhase::InstructionFetchHigh, &mut registers)
            .unwrap();
    }
    assert_eq!((registers.r1, registers.r2, registers.r3), (0, 0, 0));
}
