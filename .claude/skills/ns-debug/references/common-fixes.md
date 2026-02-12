# Common Fixes Cookbook

## Parse Errors

### "Expected neuron definition"
**Cause**: Indentation error or missing colon after neuron name.
```neuroscript
# Wrong
neuron MyBlock(dim)     # missing colon
  in: [*, dim]

# Right
neuron MyBlock(dim):    # colon required
  in: [*, dim]
```

### "Unexpected token"
**Cause**: Using reserved words as identifiers, or syntax error in expression.
```neuroscript
# Wrong — 'match' is a keyword
in -> match -> out

# Right
in -> match:
  [*, d]: Linear(d, 512) -> out
```

### "Expected indent"
**Cause**: Block body not indented after `:`.
```neuroscript
# Wrong
graph:
in -> Linear(dim, dim) -> out    # not indented

# Right
graph:
  in -> Linear(dim, dim) -> out  # indented
```

## Validation Errors

### MissingNeuron → "not found"

Check these in order:
1. Spelling and case: `LayerNorm` not `Layernorm` or `layernorm`
2. Is it a registered primitive? Check: `grep -A1 'self\.register(' src/stdlib_registry.rs | grep '"' | sed 's/.*"\([^"]*\)".*/\1/' | grep -i <name>`
3. Is it defined in the same file?
4. Does it need a `use` import?

### PortMismatch → "shape incompatibility"

1. Trace the shape chain manually:
   ```
   in: [batch, seq, 768]
   → Linear(768, 512) → [batch, seq, 512]
   → LayerNorm(512) → [batch, seq, 512]   # OK
   → Linear(768, 256) → ERROR: expects [*, 768] but gets [*, 512]
   ```
2. Fix dimension parameters to match the actual data flow

### ArityMismatch → "expected N ports, got M"

```neuroscript
# Implicit fork (preferred) — any single-output source → N-way tuple
in -> (a, b)                   # OK: 2-way implicit fork
in -> (a, b, c, d)            # OK: N-way implicit fork

# Explicit Fork/Fork3 must match arity
in -> Fork() -> (a, b)        # OK
in -> Fork() -> (a, b, c)     # ERROR: Fork has 2 outputs

# Add = 2 inputs
(a, b) -> Add() -> out        # OK
(a, b, c) -> Add() -> out     # ERROR: Add has 2 inputs
```

### NonExhaustiveMatch

Add a catch-all as the last arm:
```neuroscript
in -> match:
  [*, 512]: Identity() -> out
  [*, d]: Linear(d, 512) -> out    # ← add this
```

### UnreachableMatchArm

Move specific patterns before general ones:
```neuroscript
# Wrong order
in -> match:
  [*, d]: Linear(d, 512) -> out        # catches everything
  [*, 512]: Identity() -> out          # unreachable

# Right order
in -> match:
  [*, 512]: Identity() -> out          # specific first
  [*, d]: Linear(d, 512) -> out        # general last
```

### CycleDetected

For intentional recursion, use `@lazy` binding:
```neuroscript
context:
  @lazy recurse = MyNeuron(depth - 1)
graph:
  in -> match:
    [*] where depth > 0: ... -> recurse -> out
    [*]: Identity() -> out
```

### InvalidRecursion

Ensure all three requirements:
1. `@lazy` annotation (not `@static` or `@global`)
2. Decreasing parameter in recursive call (`depth - 1`)
3. Base case in graph (`where depth > 0` guard + Identity fallback)

## Compile Errors

### "Cannot determine which neuron to compile"

File has multiple neurons and auto-detect can't choose.
```bash
# List neurons first
./target/release/neuroscript list <file.ns>

# Then compile specific one
./target/release/neuroscript compile <file.ns> --neuron MyNeuron
```

### "Neuron '<Name>' not found in file"

The `--neuron` name doesn't match any definition in the file. Check:
```bash
./target/release/neuroscript list <file.ns>
```

Names are case-sensitive.
