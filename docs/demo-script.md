# Qydrel Demo Script

This is a short live-review path. It is designed to show the project as a
correctness lab, not as a toy language demo.

## 0. Windows setup if needed

Use a Visual Studio Developer Command Prompt, or run this once in `cmd.exe`:

```bat
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
```

## 1. Prove the repository builds and tests

```bash
cargo test --locked --all-targets --all-features
```

Say: this runs the unit and integration test surface, including corpus and bug
museum checks.

## 2. Show source-level oracle comparison

```bash
cargo run --locked --release -- examples/fibonacci.lang --oracle
```

Say: the AST interpreter is independent from the bytecode VM, so this is not
just the VM agreeing with itself.

## 3. Show backend equivalence

```bash
cargo run --locked --release -- examples/fibonacci.lang --compare-backends
```

Say: the same compiled bytecode is checked against the reference VM, GC VM,
optimized VM, and JIT only when the verifier says the JIT is safe to use.

## 4. Show trace replay and VM/GC trace diff

```bash
cargo run --locked --release -- examples/hello.lang --trace-replay --audit-json trace-replay.audit.json
cargo run --locked --release -- examples/hello.lang --trace-diff --audit-json trace-diff.audit.json
```

Say: this checks runtime behavior at instruction-trace level, not only final
printed output.

## 5. Generate the reviewer evidence packet

```bash
cargo run --locked --release -- --evidence-report evidence/latest --evidence-fuzz 5
```

Say: this produces Markdown and JSON evidence covering corpus programs, fuzz
matrix runs, backend comparison, trace replay, VM/GC diff, opcode coverage, and
bug museum status.

## 6. Show the historical bug gate

```bash
cargo test --locked --test bug_museum_tests
```

Say: every fixed bug should become a minimized repro with an executable proof
gate, so future changes cannot quietly break the same class again.

## Close

The honest pitch is:

> Qydrel is a small language wrapped in serious compiler/runtime correctness
> machinery. The value is not language features; it is the evidence pipeline
> that makes backend behavior inspectable and regression-resistant.
