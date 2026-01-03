pub mod core_types;
pub mod ast;
pub mod parser;
pub mod vm;
pub mod timeloop;
pub mod provenance;
pub mod smt_encoder;
pub mod action;
pub mod types;
pub mod module;
pub mod memo;
pub mod jit;
pub mod optimizer;
pub mod aot;

pub use core_types::{Value, Address, Memory, MEMORY_SIZE};
pub use ast::{OpCode, Stmt, Program};
pub use parser::{tokenize, Parser};
pub use vm::{Executor, EpochStatus};
pub use timeloop::{TimeLoop, ConvergenceStatus, TimeLoopConfig as Config, ExecutionMode};
pub use smt_encoder::SmtEncoder;
pub use action::{ActionPrinciple, ActionConfig, FixedPointSelector, FixedPointCandidate, ProvenanceMap, SeedStrategy, SeedGenerator};
pub use types::{TemporalType, TypeChecker, TypeCheckResult, type_check};
pub use module::{Module, ModuleRegistry};
pub use memo::{EpochCache, CacheStats, DeltaTracker};
pub use jit::{JitCompiler, CompiledFunction, CompileStats, JitError, JitResult};
pub use optimizer::{Optimizer, OptLevel, OptInstr, OptStats, TieredExecutor};
pub use aot::{AotCompiler, AotStats, ObjectFile};
pub mod analysis;

mod tests;

