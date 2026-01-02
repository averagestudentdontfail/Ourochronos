pub mod core_types;
pub mod ast;
pub mod parser;
pub mod vm;
pub mod timeloop;
pub mod provenance;
pub mod smt_encoder; // Export SMT

pub use core_types::{Value, Address, Memory, MEMORY_SIZE};
pub use ast::{OpCode, Stmt, Program};
pub use parser::{tokenize, Parser};
pub use vm::{Executor, EpochStatus};
pub use timeloop::{TimeLoop, ConvergenceStatus, TimeLoopConfig as Config, ExecutionMode};
pub use smt_encoder::SmtEncoder;
pub mod analysis;

mod tests;
