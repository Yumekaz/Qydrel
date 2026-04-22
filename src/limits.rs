//! Shared MiniLang resource limits.
//!
//! These constants are part of the executable language contract. Frontend
//! checks reject source programs that exceed them, and VM backends validate
//! compiled bytecode before execution.

pub const MAX_GLOBAL_SLOTS: usize = 256;
pub const MAX_LOCAL_SLOTS: usize = 1024;
pub const MAX_FRAMES: usize = 100;
pub const MAX_OPERAND_STACK: usize = 1000;
pub const MAX_CYCLES: u64 = 100_000;
pub const MAX_INSTRUCTIONS: usize = 10_000;
pub const GC_THRESHOLD: usize = 8;
