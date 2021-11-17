use super::log::Log;
use super::Memory;
use super::Sealed;

// TODO: find better ident
pub enum SingleExecutionResult {
    Finished,
    Unfinished,
}
pub enum ExecutionResult {
    Finished(Log),
    Unfinished(Log, Sealed<Memory>),
}
