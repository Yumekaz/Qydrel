//! MiniLang - A minimal systems programming language compiler
//!
//! Features:
//! - Complete compiler pipeline (lexer → parser → semantic analysis → bytecode)
//! - x86-64 JIT compiler with native code generation
//! - Custom memory allocators (bump, free-list, slab) - ACTUALLY USED
//! - Mark-sweep garbage collector - ACTUALLY USED
//! - Stack-based bytecode VM
//! - Optimization passes (constant folding, DCE, strength reduction)
//! - Interactive REPL with incremental compilation
//!
//! This project demonstrates core systems programming concepts:
//! - Manual memory management
//! - Machine code generation
//! - Low-level OS interfaces (mmap/mprotect)
//! - Performance optimization

pub mod token;
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod sema;
pub mod compiler;
pub mod vm;
pub mod gc_vm;
pub mod alloc;
pub mod gc;
pub mod jit;
pub mod arena_ast;
pub mod runtime;
pub mod optimizer;
pub mod repl;

pub use token::{Token, TokenKind, Span};
pub use lexer::Lexer;
pub use ast::{Program, Function, Stmt, Expr, Type, BinaryOp, UnaryOp};
pub use parser::Parser;
pub use sema::SemanticAnalyzer;
pub use compiler::{Compiler, CompiledProgram, Opcode};
pub use vm::{Vm, VmResult, TrapCode};
pub use gc_vm::{GcVm, GcVmResult, GcValue, HeapArray};
pub use alloc::{BumpAllocator, FreeListAllocator, SlabAllocator, AllocatorStats};
pub use gc::{GarbageCollector, GcStats, TypeTag};
pub use jit::{JitCompiler, MachineCode, ExecutableMemory, Reg};
pub use arena_ast::{AstArena, ArenaExpr, ArenaStmt, ArenaStr, ArenaVec};
pub use runtime::{Value, GcArray, ValueStack, LocalFrame, GlobalStore};
pub use optimizer::{Optimizer, OptimizationStats};
pub use repl::Repl;

/// Compile and run a MiniLang program
pub fn run(source: &str) -> Result<VmResult, String> {
    // Lexical analysis
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    // Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| format!("Parse error: {}", e))?;

    // Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program).map_err(|errors| {
        errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    })?;

    // Compilation
    let (compiled, _) = Compiler::new().compile(&program);

    // Execution
    let mut vm = Vm::new(&compiled);
    Ok(vm.run())
}

/// Compile to bytecode without running
pub fn compile(source: &str) -> Result<CompiledProgram, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| format!("Parse error: {}", e))?;

    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program).map_err(|errors| {
        errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    })?;

    Ok(Compiler::new().compile(&program).0)
}

/// JIT compile and run (Linux x86-64 only)
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn run_jit(source: &str) -> Result<i64, String> {
    let compiled = compile(source)?;
    
    let jit = JitCompiler::new();
    let exec_mem = jit.compile(&compiled)
        .ok_or("Failed to JIT compile")?;
    
    // Call the JIT-compiled code
    let func: extern "C" fn() -> i64 = exec_mem.as_fn();
    Ok(func())
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
pub fn run_jit(_source: &str) -> Result<i64, String> {
    Err("JIT compilation only supported on Linux x86-64".to_string())
}
