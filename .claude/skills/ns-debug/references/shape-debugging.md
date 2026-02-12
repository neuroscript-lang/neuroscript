# Shape Debugging

## Shape Inference Overview

The shape inference engine (`src/shape/inference.rs`) validates shape compatibility across all connections in a neuron graph. It works by unifying dimension variables and checking constraints.

## How Unification Works

For each connection `source -> destination`, the engine:

1. Gets the output shape of the source
2. Gets the expected input shape of the destination
3. Unifies each dimension pair

### Dimension Unification Rules

| Source Dim | Dest Dim | Result |
|-----------|----------|--------|
| `Literal(512)` | `Literal(512)` | OK — exact match |
| `Literal(512)` | `Literal(256)` | ERROR — dimension mismatch |
| `Named("dim")` | `Literal(512)` | Resolves `dim = 512` |
| `Literal(512)` | `Named("dim")` | Resolves `dim = 512` |
| `Named("a")` | `Named("b")` | Records equivalence `a == b` |
| `Wildcard` | anything | OK — matches without capture |
| `Expr(dim*4)` | `Literal(2048)` | Solves `dim = 512` |

### Conflict Detection

If `dim` was previously resolved to `512` and a new connection requires `dim = 256`, that's a contradiction → error.

## Variadic Shape Matching

Variadic patterns (`*name`) match zero or more dimensions:

```
Pattern: [*batch, seq, dim]
Shape:   [32, 16, 128, 512]

Split:
  prefix: (empty)
  variadic: [32, 16]  → batch = [32, 16]
  suffix: seq=128, dim=512
```

**Only one variadic per shape**. The engine splits the concrete shape into:
- Prefix (before variadic): matched position by position
- Variadic segment: captures remaining middle dims
- Suffix (after variadic): matched position by position

## Expression Solving

The solver handles single-unknown equations:

| Equation | Solution |
|----------|----------|
| `dim * 4 = 2048` | `dim = 512` |
| `x + 128 = 640` | `x = 512` |
| `x - 64 = 448` | `x = 512` |
| `heads * d_k = 768` (heads=12) | `d_k = 64` |

### What the Solver Cannot Do

- Two unknowns: `a * b = 512` (ambiguous)
- Non-linear: `dim^2 = 256` (not supported)
- Non-exact division: `512 / 3` (not integer)
- Complex expressions: nested operations with multiple unknowns

**Workaround**: Make dimensions explicit parameters instead of inferring them.

## Debugging Steps

### 1. Check Dimension Names

Ensure the same dimension name is used consistently:

```neuroscript
# WRONG: 'd_model' vs 'dim' — inference can't unify these automatically
neuron Bad(d_model):
  in: [*, d_model]
  out: [*, dim]        # 'dim' is unknown

# RIGHT: consistent naming
neuron Good(d_model):
  in: [*, d_model]
  out: [*, d_model]
```

### 2. Check Parameter Ordering

Linear takes `(in_dim, out_dim)`:
```neuroscript
# Input is [*, 512], want output [*, 256]
Linear(512, 256)    # Correct: in=512, out=256
Linear(256, 512)    # Wrong: in=256, but input is 512
```

### 3. Verify Connection Chain

Trace shapes through each connection:
```
in: [batch, seq, 512]
  → Linear(512, 256)
out: [batch, seq, 256]
  → GELU()
out: [batch, seq, 256]    # shape-preserving
  → Linear(256, 512)
out: [batch, seq, 512]
```

### 4. Use Verbose Validation

```bash
./target/release/neuroscript validate --verbose <file.ns>
```

Shows all resolved dimensions and detected issues.

### 5. Check Variadic Constraints

Only one variadic per shape:
```neuroscript
[*batch, *features]  # ERROR
[*batch, features]   # OK: one variadic + one named
```

## Shape Error Types (from src/interfaces.rs)

| Error | Meaning |
|-------|---------|
| `Mismatch` | Two shapes don't unify |
| `DimMismatch` | Specific dimension conflict |
| `UnknownDim` | Dimension variable not resolved |
| `ConstraintViolation` | Contradictory dimension constraints |
| `NodeInferenceFailed` | Can't infer shapes for a node |
| `UnknownNode` | Node reference not in scope |
| `UnsupportedFeature` | Shape operation not yet implemented |
