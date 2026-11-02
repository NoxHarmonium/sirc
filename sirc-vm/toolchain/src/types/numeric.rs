//! Numeric operands retained in the AST until object generation.
use super::shared::{NumberToken, NumberType};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct ConstantReference {
    pub name: String,
    /// Original parser input length at the use site, never an expanded-source offset.
    #[serde(skip)]
    pub input_length: usize,
}

#[derive(Debug, Clone, Serialize)]
pub enum NumericOperand {
    Literal(NumberToken),
    Constant(ConstantReference),
}

impl From<NumberToken> for NumericOperand {
    fn from(value: NumberToken) -> Self {
        Self::Literal(value)
    }
}

pub type Constants = HashMap<String, NumberToken>;

impl NumericOperand {
    pub fn resolve(&self, constants: &Constants) -> Result<NumberToken, String> {
        match self {
            Self::Literal(number) => Ok(number.clone()),
            Self::Constant(reference) => constants.get(&reference.name).cloned().ok_or_else(|| {
                format!(
                    "Undefined constant ${}; define it with .EQU before use",
                    reference.name
                )
            }),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NumericField {
    Immediate16,
    Immediate8,
    ShiftCount,
    ExceptionVector,
    ExceptionLinkIndex,
    DmaRegisterCount,
    DmaTransferCount,
}

impl NumericField {
    pub fn encode(self, number: &NumberToken) -> Result<u16, String> {
        let value = if matches!(number.number_type, NumberType::SignedDecimal) {
            i64::from(number.value.cast_signed())
        } else {
            i64::from(number.value)
        };
        let (minimum, maximum, description) = match self {
            Self::Immediate16 => (-32768, 65535, "16-bit immediate"),
            Self::Immediate8 => (0, 255, "8 bits for a shifted immediate"),
            Self::ShiftCount => (0, 15, "shift count (range of 0-15)"),
            Self::ExceptionVector => (0, 255, "EXCP vector (8 bits)"),
            Self::ExceptionLinkIndex => (0, 15, "exception link index (4 bits)"),
            Self::DmaRegisterCount => (-7, 7, "DMA register count (range -7..7)"),
            Self::DmaTransferCount => (0, 255, "DMA transfer count (8 bits)"),
        };
        if !(minimum..=maximum).contains(&value) {
            return Err(format!(
                "Value {value} does not fit in {description}; expected {minimum}..{maximum}"
            ));
        }
        if matches!(self, Self::DmaRegisterCount) {
            Ok(u16::try_from(value.abs()).expect("validated DMA magnitude")
                | if value < 0 { 0x20 } else { 0 })
        } else if value < 0 {
            Ok(i16::try_from(value)
                .expect("validated signed immediate")
                .cast_unsigned())
        } else {
            Ok(u16::try_from(value).expect("validated numeric field"))
        }
    }
}

#[derive(Debug, Clone)]
pub struct NumericPlaceholder {
    pub reference: ConstantReference,
    pub field: NumericField,
}

impl NumericPlaceholder {
    #[must_use]
    pub fn new(reference: ConstantReference, field: NumericField) -> Self {
        Self { reference, field }
    }
}
