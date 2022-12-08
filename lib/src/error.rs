use nbitmask::error::BitMaskError;
use std::fmt;

#[derive(Clone, Debug)]
pub enum EnigmindError {
    BitmaskError(BitMaskError),
    ColumnIndexOutOfBounds,
}

impl From<BitMaskError> for EnigmindError {
    fn from(value: BitMaskError) -> Self {
        Self::BitmaskError(value)
    }
}

impl fmt::Display for EnigmindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            EnigmindError::BitmaskError(err) => write!(f, "{err}"),
            EnigmindError::ColumnIndexOutOfBounds => write!(f, "ColumnIndexOutOfBounds"),
        }
    }
}
