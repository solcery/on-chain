use super::memory::Error as InternalError;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Error {
    pub instruction: u32,
    pub error: InternalError,
}
