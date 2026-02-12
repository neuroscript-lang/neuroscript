# Match Expressions and Conditionals

## Match Expressions

Pattern match on tensor shapes at the point of connection:

```neuroscript
in -> match:
  [pattern]: pipeline
  [pattern] where guard: pipeline
  [catch-all]: pipeline
```

### Dimension Capture

Named dimensions in patterns are captured and available in guards and call arguments:

```neuroscript
in -> match:
  [batch, seq, d] where d > 512: Linear(d, 512) -> out
  [batch, seq, d]: Linear(d, 256) -> out
```

Captured: `batch`, `seq`, `d` — usable in `Linear(d, ...)` calls.

### Guard Expressions

Boolean expressions filtering which arm matches:

```neuroscript
where d > 512                   # comparison
where d == 256                  # equality
where d > 128 and d <= 512     # logical and
where d < 64 or d > 1024      # logical or
```

**Operators**: `==`, `!=`, `<`, `>`, `<=`, `>=`, `and`, `or`
**Operands**: captured dimension names, integer literals

### Exhaustiveness

The last match arm **must** be a catch-all pattern (no guard, or a pattern that matches any shape):

```neuroscript
# Valid catch-all patterns:
[*]: ...                  # variadic, matches anything
[*, d]: ...               # named dim, matches any rank-2+
[*shape]: ...             # named variadic
[batch, seq, d]: ...      # if all preceding arms use guards

# ERROR: Non-exhaustive
in -> match:
  [*, 512]: Identity() -> out
  [*, 256]: Linear(256, 512) -> out
  # Missing catch-all!
```

### Pattern Shadowing

Earlier patterns shadow later ones. The validator warns about unreachable arms:

```neuroscript
in -> match:
  [*, d]: Linear(d, 512) -> out      # catches everything
  [*, 256]: Identity() -> out        # WARNING: unreachable
```

Order patterns from most specific to most general.

### Multi-line Match Pipelines

```neuroscript
in -> match:
  [*, d] where d > 512:
    Linear(d, 512)
    GELU()
    Dropout(0.1)
    out
  [*, d]:
    Linear(d, 512)
    out
```

### Dimension Binding in Codegen

When match captures dimensions, the compiler generates lazy module instantiation:

```neuroscript
in -> match:
  [*, d]: Linear(d, 512) -> out
```

Generates Python that reads `d` from the input tensor shape at runtime and creates `Linear(d, 512)` on first use, caching it for subsequent calls with the same `d`.

## If/Elif/Else Expressions

Conditional routing based on parameter values (not shapes):

```neuroscript
in -> if condition:
  pipeline
elif condition:
  pipeline
else:
  pipeline
```

### Example

```neuroscript
neuron Flexible(dim, use_norm=true):
  in: [*, dim]
  out: [*, dim]
  graph:
    in -> if use_norm:
      LayerNorm(dim) -> Linear(dim, dim) -> out
    else:
      Linear(dim, dim) -> out
```

### Difference from Match

| Feature | `match:` | `if/elif/else:` |
|---------|----------|-----------------|
| Routes on | Tensor shapes | Parameter values |
| Captures dims | Yes | No |
| Guards | Shape-based (`where`) | Value-based |
| Use case | Runtime shape dispatch | Compile-time config |
