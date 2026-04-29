# Qydrel Architecture

Qydrel is shaped around one question: can a small compiler/runtime prove its
own behavior from several independent angles before anyone trusts an optimized
or native backend?

```mermaid
flowchart TD
    Source["MiniLang source"] --> Lexer["Lexer"]
    Lexer --> Parser["Parser"]
    Parser --> Sema["Semantic analyzer"]
    Sema --> AST["Typed AST"]

    AST --> Oracle["Independent AST oracle"]
    AST --> Compiler["Bytecode compiler"]
    Compiler --> Bytecode["Bytecode program"]
    Bytecode --> Verifier["Verifier and proof gates"]

    Verifier --> VM["Reference VM"]
    Verifier --> GCVM["GC VM"]
    Verifier --> OptVM["Optimized VM"]
    Verifier --> JITGate["JIT eligibility gate"]
    JITGate --> JIT["Linux x86-64 JIT subset"]
    JITGate --> Skip["Skip native backend when unsafe"]

    Oracle --> Compare["Observable backend comparison"]
    VM --> Compare
    GCVM --> Compare
    OptVM --> Compare
    JIT --> Compare

    VM --> Trace["Replayable instruction trace"]
    GCVM --> TraceDiff["VM vs GC trace diff"]
    Trace --> Replay["Trace replay"]

    Corpus["Regression corpus"] --> Evidence["Evidence report"]
    Fuzzer["Deterministic and metamorphic fuzzer"] --> Shrinker["Exact-fingerprint shrinker"]
    Shrinker --> Corpus
    Compare --> Evidence
    Replay --> Evidence
    TraceDiff --> Evidence
    Verifier --> Evidence
    BugMuseum["Bug museum"] --> Evidence
```

## Core Boundary

The source language is intentionally small. The novelty is not syntax breadth;
it is the surrounding correctness machinery:

- Independent AST execution gives a source-level oracle that does not reuse the
  bytecode VM implementation.
- The verifier decides whether bytecode is structurally valid and whether a
  backend is allowed to execute it.
- Backend comparison checks the reference VM, GC VM, optimized VM, and eligible
  JIT path against the same observable result.
- Trace replay and VM/GC trace diff catch runtime-level divergence that plain
  output comparison can miss.
- Fuzzing, metamorphic variants, shrinking, corpus replay, and bug-museum tests
  turn discovered failures into durable evidence.

## Trust Model

Qydrel does not claim a formal proof. It claims executable, reproducible
evidence for the current language contract. A backend earns trust only when the
verifier allows it and the audit pipeline can compare it against independent
checks.
