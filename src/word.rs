/// Одна ячейка памяти на стеке может содержать либо число, либо логическое значение.
/// Операции будут проверять, что значение нужного типа, поэтому вызвать 1 + True нельзя, это
/// вызовет панику.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Word {
    Numeric(i32),
    Boolean(bool),
}

impl Word {
    pub fn unwrap_numeric(self) -> i32 {
        match self {
            Word::Numeric(i) => {
                i
            }
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
