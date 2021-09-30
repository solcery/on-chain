use serde::{Deserialize, Serialize};
/// Одна ячейка памяти на стеке может содержать либо число, либо логическое значение.
/// Операции будут проверять, что значение нужного типа, поэтому вызвать 1 + True нельзя, это
/// вызовет панику.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Word {
    Numeric(i32),
    Boolean(bool),
}

impl Word {
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
        Word::Numeric(0)
    }
}
impl From<i32> for Word {
    fn from(val: i32) -> Self {
        Word::Numeric(val)
    }
}
impl From<bool> for Word {
    fn from(val: bool) -> Self {
        Word::Boolean(val)
    }
}

#[macro_export]
macro_rules! word_vec {
        ($($elem:expr),* $(,)?) => { vec![$(Word::from($elem)),*]  };

}
