#[derive(PartialEq, Eq, Debug)]
pub enum Sign {
    Positive,
    Negative,
}

pub fn sign(value: u16) -> Sign {
    let sign_bit = 0x8000;

    if (value & sign_bit) == sign_bit {
        Sign::Negative
    } else {
        Sign::Positive
    }
}
