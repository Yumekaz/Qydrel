# Qydrel Evidence Report Snapshot

Snapshot command:

```bash
cargo run --locked --release -- --evidence-report evidence/latest --evidence-fuzz 5
```

This is a checked-in Markdown snapshot of the current evidence packet. The
generated `evidence/latest/` directory is intentionally ignored because it is a
local build artifact. Regenerate it with the command above when reviewing a new
commit.

## Verdict

Passed: corpus files=3, fuzz runs=8, fuzz cases executed=40, bug museum
entries=1, local fuzz artifacts=0.

## Coverage Dashboard

Observed coverage from checked-in corpus programs plus the deterministic
evidence fuzz matrix. This is execution evidence, not a completeness proof.
Opcodes observed: 26/34. AST oracle comparisons: 281. Metamorphic variants
checked: 241.

### Feature Coverage

| Feature | Cases | Share |
| --- | ---: | ---: |
| coverage-guided selected cases | 40 / 40 | 100.0% |
| optimizer stress cases | 20 / 40 | 50.0% |
| helper functions | 20 / 40 | 50.0% |
| helper calls | 20 / 40 | 50.0% |
| branches | 40 / 40 | 100.0% |
| loops | 39 / 40 | 97.5% |
| print statements | 35 / 40 | 87.5% |
| global array reads | 20 / 40 | 50.0% |
| global array writes | 14 / 40 | 35.0% |
| local array reads | 20 / 40 | 50.0% |
| local array writes | 20 / 40 | 50.0% |
| loop-indexed array writes | 19 / 40 | 47.5% |
| helper/array interactions | 20 / 40 | 50.0% |
| constant-fold patterns | 20 / 40 | 50.0% |
| dead-code shapes | 20 / 40 | 50.0% |
| metamorphic return-neutral variants | 40 / 241 | 16.6% |
| metamorphic dead-branch variants | 40 / 241 | 16.6% |
| metamorphic unused-local variants | 40 / 241 | 16.6% |
| metamorphic algebraic-neutral variants | 40 / 241 | 16.6% |
| metamorphic branch-inversion variants | 40 / 241 | 16.6% |
| metamorphic helper-wrapping variants | 40 / 241 | 16.6% |
| metamorphic statement-reordering variants | 1 / 241 | 0.4% |

### Opcode Coverage

| Opcode | Corpus | Fuzz | Status |
| --- | --- | --- | --- |
| LoadConst | yes | yes | observed |
| LoadLocal | yes | yes | observed |
| StoreLocal | yes | yes | observed |
| LoadGlobal | no | yes | observed |
| StoreGlobal | no | yes | observed |
| Add | yes | yes | observed |
| Sub | yes | yes | observed |
| Mul | yes | yes | observed |
| Div | no | yes | observed |
| Neg | no | yes | observed |
| Eq | yes | yes | observed |
| Ne | yes | yes | observed |
| Lt | yes | yes | observed |
| Gt | no | yes | observed |
| Le | no | yes | observed |
| Ge | no | yes | observed |
| And | no | no | missing |
| Or | no | no | missing |
| Not | no | no | missing |
| Jump | yes | yes | observed |
| JumpIfFalse | yes | yes | observed |
| JumpIfTrue | no | no | missing |
| Call | yes | yes | observed |
| Return | yes | yes | observed |
| ArrayLoad | yes | yes | observed |
| ArrayStore | yes | yes | observed |
| ArrayNew | no | no | missing |
| LocalArrayLoad | yes | yes | observed |
| LocalArrayStore | yes | yes | observed |
| AllocArray | yes | yes | observed |
| Print | yes | yes | observed |
| Pop | no | no | missing |
| Dup | no | no | missing |
| Halt | no | no | missing |

## Corpus Status

| File | Compile | Verify | Oracle | Backends | Replay | VM/GC Diff |
| --- | --- | --- | --- | --- | --- | --- |
| `tests/corpus/jit_scalar_locals.lang` | passed | passed | passed | passed | passed | passed |
| `tests/corpus/local_array_loop.lang` | passed | passed | passed | passed | passed | passed |
| `tests/corpus/optimizer_stress.lang` | passed | passed | passed | passed | passed | passed |

## Fuzz Matrix Detail

| Seed | Mode | Cases | Status | Branches | Loops | Local Arrays | Opcodes |
| --- | --- | ---: | --- | ---: | ---: | ---: | ---: |
| 0x0000000000005eed | general | 5 | passed | 5 | 5 | 10 | 26 |
| 0x0000000000c0ffee | general | 5 | passed | 5 | 4 | 10 | 26 |
| 0x000000000badc0de | general | 5 | passed | 5 | 5 | 10 | 26 |
| 0x0000000000051ced | general | 5 | passed | 5 | 5 | 10 | 26 |
| 0x0000000000005eed | optimizer-stress | 5 | passed | 5 | 5 | 0 | 12 |
| 0x0000000000c0ffee | optimizer-stress | 5 | passed | 5 | 5 | 0 | 12 |
| 0x000000000badc0de | optimizer-stress | 5 | passed | 5 | 5 | 0 | 12 |
| 0x0000000000051ced | optimizer-stress | 5 | passed | 5 | 5 | 0 | 12 |

## Backend Matrix

| Program | VM | GC VM | Optimized VM | JIT |
| --- | --- | --- | --- | --- |
| `tests/corpus/jit_scalar_locals.lang` | executed | executed | executed | skipped |
| `tests/corpus/local_array_loop.lang` | executed | executed | executed | skipped |
| `tests/corpus/optimizer_stress.lang` | executed | executed | executed | skipped |

## Bug Museum

| Entry | Status | Expected Behavior | Proof Gate | Repro | Docs |
| --- | --- | --- | --- | --- | --- |
| `jit-undefined-local-proof-gate` | fixed | `vm_trap_undefined_local_jit_skipped` | `verifier_possible_trap_blocks_jit` | yes | yes |

## Local Fuzz Artifacts

| Artifact | Manifest | Minimized Source |
| --- | --- | --- |
| none | no | no |
