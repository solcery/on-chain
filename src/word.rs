use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
/// Одна ячейка памяти на стеке может содержать либо число, либо логическое значение.
/// Операции будут проверять, что значение нужного типа, поэтому вызвать 1 + True нельзя, это
/// вызовет панику.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Word {
    Numeric(i32),
    Boolean(bool),
}

impl Word {
    #[must_use]
    pub fn unwrap_numeric(self) -> i32 {
        match self {
            Word::Numeric(i) => i,
            Word::Boolean(_) => {
                panic!("Called unwrap_numeric on Word::Boolean.");
            }
        }
    }
}

impl Default for Word {
    fn default() -> Self {
        Self::Numeric(0)
    }
}
impl From<i32> for Word {
    fn from(val: i32) -> Self {
        Self::Numeric(val)
    }
}
impl From<bool> for Word {
    fn from(val: bool) -> Self {
        Self::Boolean(val)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum ConversionError {
    WasNumeric,
    WasBoolean,
    NegativeNumeric,
}

impl TryFrom<Word> for i32 {
    type Error = ConversionError;

    fn try_from(value: Word) -> Result<Self, Self::Error> {
        match value {
            Word::Numeric(val) => Ok(val),
            Word::Boolean(_) => Err(Self::Error::WasBoolean),
        }
    }
}

impl TryFrom<Word> for bool {
    type Error = ConversionError;

    fn try_from(value: Word) -> Result<Self, Self::Error> {
        match value {
            Word::Numeric(_) => Err(Self::Error::WasNumeric),
            Word::Boolean(val) => Ok(val),
        }
    }
}
impl TryFrom<Word> for usize {
    type Error = ConversionError;

    fn try_from(value: Word) -> Result<Self, Self::Error> {
        match value {
            Word::Numeric(val) if val >= 0 => Ok(val as Self),
            Word::Numeric(_) => Err(Self::Error::NegativeNumeric),
            Word::Boolean(_) => Err(Self::Error::WasBoolean),
        }
    }
}

#[macro_export]
macro_rules! word_vec {
        ($($elem:expr),* $(,)?) => { vec![$(Word::from($elem)),*]  };

}
