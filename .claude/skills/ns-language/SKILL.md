---
name: ns-language
description: NeuroScript language syntax reference. Use when questions arise about shapes, dimensions, ports, connections, match expressions, context bindings, if/elif/else, or impl references. Also use when writing or reviewing .ns files.
allowed-tools: Read, Grep, Glob

---

# NeuroScript Language Syntax Reference

## Additional Skills

<important>
Always use the appropriate skill for the job
</important>

**Neuroscrupt Standard Library**: `ns-stdlib`
**Packaging Neurons**: `ns-package`
**Creating New Neurons**: `ns-create`
**Debugging Neurons**: `ns-debug`

## Neuron Definitions

```neuroscript
neuron Name(param1, param2=default):
  in: [shape]
  out: [shape]
  impl: provider,module/Class     # primitive
  # OR
  graph:                           # composite
    in -> ... -> out
```

- Names: `PascalCase` — Parameters/dimensions/ports: `snake_case`
- Params accept: integers, floats, strings (backtick-quoted), booleans, expressions
- Type annotation: `param: NeuronType` for meta-parameters

## Ports

```neuroscript
# Single default port
in: [*, dim]
out: [*, dim]

# Named ports
in query: [batch, seq, dim]
in key: [batch, seq, dim]
out attention: [batch, seq, dim]

# Indented multi-port
in:
  query: [batch, seq, dim]
  key: [batch, seq, dim]
out:
  result: [batch, seq, dim]
```

Default port name is `in`/`out` (internal name "default"). Access named ports with dot notation: `fork.left`, `mha.query`.

## Shapes (Quick Reference)

| Syntax | Meaning | Example |
|--------|---------|---------|
| `[512, 256]` | Literal dims | Fixed sizes |
| `[batch, seq, dim]` | Named dims | Unify across connections |
| `[*, dim]` | Single wildcard | Any one dim |
| `[*shape]` | Variadic wildcard | Zero or more dims |
| `[*batch, seq, dim]` | Variadic prefix | Any leading + fixed trailing |
| `[dim * 4]` | Expression dim | Arithmetic: `+`, `-`, `*`, `/` |

**Constraint**: Only one variadic per shape. `[*a, *b]` is invalid.

See `references/shapes-and-dimensions.md` for full shape system details.

## Connections & Pipelines

```neuroscript
# Single-line pipeline
in -> Linear(512, 256) -> GELU() -> out

# Multi-line (indentation = implicit ->)
in ->
  Linear(512, 256)
  GELU()
  Dropout(0.1)
  out

# Named references
in -> Linear(dim, dim) -> hidden
hidden -> GELU() -> out

# Implicit fork (v0.3.0+) — preferred for splitting
in -> (main, skip)                   # 2-way implicit fork
in -> (a, b, c, d)                   # N-way implicit fork
main -> Linear(dim, dim) -> processed
(processed, skip) -> Add() -> out

# Explicit Fork (when you need named port access)
in -> Fork() -> f
f.left -> Linear(dim, dim) -> processed
f.right -> Identity() -> skip
```

## Match Expressions

```neuroscript
in -> match:
  [*, 512]: Identity() -> out           # literal match
  [*, d] where d > 256: Linear(d, 512) -> out  # guard
  [*, d]: Linear(d, 512) -> out         # catch-all (required)
```

- Captured dims (`d`, `batch`) usable in guards and call args
- Guards: `==`, `!=`, `<`, `>`, `<=`, `>=`, `and`, `or`
- Last arm must be catch-all (variadic or unguarded named)
- Earlier patterns shadow later ones — order matters

See `references/match-and-conditionals.md` for full details.

## Context Bindings

```neuroscript
neuron Block(dim, depth):
  in: [*, dim]
  out: [*, dim]
  context:
    @lazy recurse = Block(dim, depth - 1)   # lazy: created on first use
    @static norm = LayerNorm(dim)            # static: created in __init__
    @global shared = SharedModule()          # global: shared across instances
  graph:
    in -> match:
      [*] where depth > 0: norm -> Linear(dim, dim) -> recurse -> out
      [*]: Identity() -> out
```

- `@lazy` — lazy instantiation, supports recursion and match-captured dims
- `@static` — eager instantiation in `__init__()`, no recursion
- `@global` — shared singleton across all instances

See `references/context-bindings.md` for recursion patterns.

## Impl References

```neuroscript
impl: provider,module/ClassName
```

Format: `<provider>,<path>/<class>` — **not** dot notation.
- `core,nn/Linear` — maps to `from neuroscript_runtime.primitives.linear import Linear`
- `core,attention/ScaledDotProductAttention`
- `core,structural/Fork`

## Comments & Documentation

```neuroscript
# Single-line comment
/// Doc comment (triple-slash) — extracted by doc tools
```

## Keywords

`neuron`, `use`, `in`, `out`, `impl`, `graph`, `match`, `where`, `context`, `if`, `elif`, `else`, `external`, `and`, `or`, `true`, `false`

## Use Statements

```neuroscript
use core,nn/*                    # wildcard import
use core,nn/Linear               # specific import
```

Format: `use <source>,<path>/<item>`
