#[cfg(test)]
mod tests {

    use quickcheck::{Arbitrary, Gen};

    use super::definitions::*;
    use super::encoding::*;

    // TODO: fuzz tests

    impl Arbitrary for Point {
        fn arbitrary(g: &mut Gen) -> HaltInstructionData {
            Point {
                condition_flag: u8::arbitrary(g),
            }
        }
    }

    #[test]
    fn round_trip_encoding_test() {
        let all_instructions = vec![Instruction::Halt(HaltInstructionData {
            data: ImpliedInstructionData {
                condition_flag: ConditionFlags::UnsignedLowerOrSame,
            },
        })];
    }
}
