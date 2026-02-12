---
name: ns-debug
description: Debug NeuroScript validation errors, shape mismatches, parse failures, cycle detection, and compilation problems. Use when encountering errors from neuroscript parse/validate/compile or debugging .ns files.
allowed-tools: Read, Grep, Glob, Bash
---

# NeuroScript Debug & Validate

## Validation Output (if file provided)

!`[ -n "$ARGUMENTS" ] && [ -f "$ARGUMENTS" ] && ./target/release/neuroscript validate --verbose "$ARGUMENTS" 2>&1 || true`

## Error Quick Reference

| Error | Cause | Fix |
|-------|-------|-----|
| `MissingNeuron` | Referenced neuron not defined or registered | Check spelling, add `use` import, or define the neuron |
| `PortMismatch` | Shape incompatibility between connected ports | Check dimension names match, add Reshape/Linear to adapt |
| `CycleDetected` | Circular dependency between neurons | Break cycle, use `@lazy` recursion with depth guard |
| `ArityMismatch` | Wrong number of ports in tuple unpacking | Single-output sources can implicit-fork to any tuple size; multi-output sources must match exactly |
| `UnknownNode` | Reference to undefined variable/binding | Check binding names, ensure tuple unpacking creates the ref |
| `NonExhaustiveMatch` | Match missing catch-all pattern | Add final arm: `[*, d]: ... -> out` or `[*]: ...` |
| `UnreachableMatchArm` | Pattern shadowed by earlier arm | Reorder: specific patterns first, catch-all last |
| `DuplicateBinding` | Same name used twice in context block | Rename one of the duplicate bindings |
| `InvalidRecursion` | Recursive binding violates rules | Use `@lazy` (not `@static`), add depth guard, decrease param |
| `Custom` | Other validation error | Read the message for specifics |

## Diagnosis Workflow

### Step 1: Parse

```bash
./target/release/neuroscript parse --verbose <file.ns>
```

If this fails, the issue is **syntax**: indentation, missing colons, wrong keywords, unmatched brackets.

### Step 2: Validate

```bash
./target/release/neuroscript validate --verbose <file.ns>
```

If parse passes but validate fails, the issue is **semantic**: missing neurons, shape mismatches, cycles, bad bindings.

### Step 3: Compile

```bash
./target/release/neuroscript compile --verbose <file.ns> --neuron <Name>
```

If validate passes but compile fails, the issue is **codegen**: auto-detect failure (use `--neuron`), or unsupported feature.

## Common Error Patterns

### Shape Mismatch

```
Port mismatch: Linear [*, 512] -> GELU [*, 256]
```

**Diagnosis**: The output of Linear is `[*, 512]` but the next neuron expects `[*, 256]`.
**Fix**: Check dimension parameters — likely `Linear(256, 512)` should be `Linear(512, 256)` or vice versa.

### Missing Neuron

```
Neuron 'CustomBlock' not found
```

**Diagnosis**: `CustomBlock` isn't defined in the file or registered as a primitive.
**Fix**: Define `CustomBlock` in the same file, add a `use` import, or check the spelling against available primitives.

### Non-Exhaustive Match

```
Non-exhaustive match expression: add a catch-all pattern
```

**Diagnosis**: All match arms have specific patterns or guards; none catches everything.
**Fix**: Add a final arm with no guard: `[*, d]: Linear(d, target) -> out`

### Arity Mismatch

```
Arity mismatch: expected 2 ports, got 3
```

**Diagnosis**: Tuple unpacking size doesn't match the neuron's output count (for multi-output sources).
**Fix**: Single-output sources can implicit-fork to any tuple size (`in -> (a, b, c)` works). Multi-output sources (explicit Fork=2, Fork3=3) must match exactly.

## CLI Commands for Debugging

```bash
# Progressive diagnosis
./target/release/neuroscript parse <file.ns>              # syntax only
./target/release/neuroscript validate <file.ns>            # + semantics
./target/release/neuroscript validate --verbose <file.ns>  # detailed
./target/release/neuroscript compile <file.ns>             # + codegen

# List neurons (verify structure)
./target/release/neuroscript list <file.ns>
./target/release/neuroscript list --verbose <file.ns>

# Compile specific neuron (bypass auto-detect)
./target/release/neuroscript compile <file.ns> --neuron MyNeuron

# Compile without optimizations (simpler output)
./target/release/neuroscript compile <file.ns> --no-optimize
```

## See Also

- `references/error-catalog.md` — all 10 error types with detailed explanations
- `references/shape-debugging.md` — shape inference rules and debugging
- `references/common-fixes.md` — cookbook of error → fix patterns
