# MiniLang - A Systems Programming Language Compiler in Rust

A minimal language compiler demonstrating core systems programming concepts:
- **Custom memory allocators** (bump, free-list, slab)
- **Mark-sweep garbage collector**
- **x86-64 JIT compiler** with native code generation
- **Bytecode VM** interpreter

## Building

```bash
cargo build --release
```

## Usage

```bash
# Run with interpreter
./target/release/minilang examples/fibonacci.lang

# Run with JIT compiler (Linux x86-64 only)
./target/release/minilang examples/fibonacci.lang --jit

# Show bytecode IR
./target/release/minilang examples/fibonacci.lang --ir

# Benchmark mode
./target/release/minilang examples/fibonacci.lang --bench

# Show allocator/GC stats
./target/release/minilang examples/fibonacci.lang --stats
```

## Project Structure

```
src/
├── lib.rs          # Library exports
├── main.rs         # CLI entry point
├── token.rs        # Token definitions
├── lexer.rs        # Lexical analyzer
├── ast.rs          # AST node definitions
├── parser.rs       # Recursive descent parser
├── sema.rs         # Semantic analyzer (type checking)
├── compiler.rs     # Bytecode compiler
├── vm.rs           # Stack-based VM interpreter
├── jit.rs          # x86-64 JIT compiler
├── alloc.rs        # Custom memory allocators
└── gc.rs           # Mark-sweep garbage collector
```

## Systems Engineering Features

### Memory Allocators (`src/alloc.rs`)

Three allocator implementations:

1. **Bump Allocator**: O(1) allocation, bulk deallocation
   - Perfect for compiler phases with known lifetimes
   - Zero fragmentation, cache-friendly

2. **Free-List Allocator**: General purpose with coalescing
   - First-fit allocation strategy
   - Adjacent block coalescing on free

3. **Slab Allocator**: Fixed-size object pools
   - Extremely fast for uniform allocations
   - No external fragmentation

### Garbage Collector (`src/gc.rs`)

Mark-sweep GC implementation:
- Object headers with type tags and mark bits
- Root set management
- Automatic collection at threshold

### JIT Compiler (`src/jit.rs`)

x86-64 native code generation:
- Direct machine code emission (no LLVM)
- System V AMD64 ABI compliance
- Executable memory via mmap/mprotect

### Bytecode VM (`src/vm.rs`)

Stack-based interpreter:
- 30+ bytecode instructions
- Call stack with frames
- Runtime error trapping

## Benchmarking

```bash
# Run benchmarks
cargo bench

# Profile with perf (Linux)
cargo build --release
perf record -g ./target/release/minilang examples/fibonacci.lang --bench
perf report
```

## Language Reference

### Types
- `int`: 32-bit signed integer
- `bool`: boolean (true/false)

### Operators
- Arithmetic: `+`, `-`, `*`, `/`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Logical: `&&`, `||`, `!`

### Statements
```
int x = 10;              // Variable declaration
x = x + 1;               // Assignment
if (x > 5) { } else { }  // Conditional
while (x < 100) { }      // Loop
return x;                // Return
print x;                 // Output
```

### Functions
```
func add(int a, int b) {
    return a + b;
}

func main() {
    return add(1, 2);
}
```

### Arrays
```
int arr[10];             // Array declaration
arr[0] = 42;             // Array assignment
return arr[0];           // Array access
```

## Performance Characteristics

| Component | Complexity | Notes |
|-----------|------------|-------|
| Lexer | O(n) | Single pass, no backtracking |
| Parser | O(n) | Recursive descent, predictive |
| Semantic | O(n) | Single pass with symbol table |
| Compiler | O(n) | Direct translation |
| VM | O(cycles) | Stack-based, ~100K cycle limit |
| JIT | O(n) | Linear code generation |

## License

MIT
