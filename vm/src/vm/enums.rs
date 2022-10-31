use super::log::Log;
use super::Memory;
use super::Sealed;

// TODO: find better ident
#[derive(Debug)]
pub enum SingleExecutionResult {
    Finished,
    Unfinished,
}

#[derive(Debug)]
pub enum ExecutionResult {
    Finished(Log),
    Unfinished(Log, Sealed<Memory>),
}
