# Shapes and Dimensions

## Dimension Types

### 1. Literal Dimensions
```neuroscript
[512, 256, 128]    # concrete integer dimensions
[1, 1024]          # batch=1, seq=1024
```

### 2. Named Dimensions
```neuroscript
[batch, seq, dim]       # variables unified across connections
[b, heads, seq, d_k]   # bound through shape inference
```

Named dimensions enable shape polymorphism — the inference engine resolves them by tracking constraints across connections.

### 3. Single Wildcard (`*`)
```neuroscript
[*, dim]        # one arbitrary dim + fixed dim
[batch, *]      # fixed batch + one arbitrary dim
```

Matches exactly **one** dimension without capturing its value.

### 4. Variadic Wildcard (`*name` or bare `*` in patterns)
```neuroscript
[*shape]              # any rank (0+ dims)
[*batch, seq, dim]    # variadic prefix + fixed trailing
[batch, seq, *rest]   # fixed prefix + variadic trailing
```

Matches **zero or more** dimensions and captures the matched sequence.

**Constraint**: Only one variadic per shape. `[*a, *b]` is invalid.

### 5. Expression Dimensions
```neuroscript
[dim * 4]           # product
[seq - 1]           # subtraction
[heads * d_k]       # product of named dims
[d_model / heads]   # division (must be exact)
```

Operators: `+`, `-`, `*`, `/`. Division must be exact (integer).

## Shape Inference

### How Unification Works

When two shapes connect, the inference engine unifies each dimension pair:

| Source | Destination | Result |
|--------|-------------|--------|
| `Literal(512)` | `Literal(512)` | Match |
| `Literal(512)` | `Literal(256)` | **Error**: dimension mismatch |
| `Named("dim")` | `Literal(512)` | Resolves `dim = 512` |
| `Named("a")` | `Named("b")` | Records equivalence `a == b` |
| `Wildcard` | anything | Match (no capture) |
| `Expr(dim*4)` | `Literal(2048)` | Solves `dim = 512` |

### Expression Solving

The solver handles single-unknown linear equations:
- `a * x = b` → `x = b / a` (if exact division)
- `x + a = b` → `x = b - a`
- `x - a = b` → `x = b + a`
- `a / x = b` → not solvable in general

**Limitations**:
- Non-linear or multi-unknown equations may not solve
- Integer division must be exact
- Ambiguous systems produce errors

### Variadic Matching

Variadic patterns split into prefix and suffix around the variadic position:

```
Pattern: [*batch, seq, dim]  matches  [32, 16, 128, 512]
  prefix: (empty)
  variadic captures: [32, 16]  → batch = [32, 16]
  suffix: seq=128, dim=512
```

### Broadcasting

NumPy-style broadcasting rules apply when checking compatibility:
- Dimensions align from the right
- Dimension `1` broadcasts to any size
- Variadic dimensions can absorb mismatched ranks

## Shape Algebra Operations

| Operation | Description |
|-----------|-------------|
| `size()` | Total element count: `[10, 20, 30] → 6000` |
| `rank()` | Number of dimensions: `[batch, seq, dim] → 3` |
| `broadcastable(other)` | NumPy broadcasting check |
| `refine_axis(i, factors)` | Split dimension: `[6] → [2, 3]` |
| `coarsen_axes(i, j)` | Merge dimensions: `[10, 20, 30] → [200, 30]` |
| `flatten()` | Reduce to rank 1 |

All arithmetic uses `BigUint` to prevent overflow with large tensor sizes.

## Common Patterns

```neuroscript
# Shape-preserving (activations, norms)
in: [*shape]
out: [*shape]

# Feature transformation
in: [*, in_dim]
out: [*, out_dim]

# Sequence processing
in: [batch, seq, dim]
out: [batch, seq, dim]

# Multi-head reshape
in: [batch, seq, d_model]
out: [batch, heads, seq, d_model / heads]
```
