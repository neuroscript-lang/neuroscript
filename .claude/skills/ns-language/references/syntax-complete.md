# NeuroScript Complete Syntax Reference

## Program Structure

A NeuroScript program consists of:
1. Optional `use` statements (imports)
2. Optional `@global` declarations
3. One or more `neuron` definitions

```neuroscript
use core,nn/*

@global shared_embed = Embedding(vocab, dim)

neuron MyModel(dim):
  in: [*, dim]
  out: [*, dim]
  graph:
    in -> Linear(dim, dim) -> out
```

## Neuron Definition

```
neuron <Name>(<params>):
  in: <port_spec>
  out: <port_spec>
  <body>
```

### Parameters

```neuroscript
neuron NoParams:                          # no params
neuron OneParam(dim):                     # required
neuron WithDefault(dim, eps=1e-5):        # with default
neuron Typed(count, nt: NeuronType):      # type-annotated
```

Parameter values: integers (`512`), floats (`0.1`, `1e-5`), strings (`` `relu` ``), booleans (`true`/`false`), expressions (`dim * 4`).

### Port Specifications

**Inline single port:**
```neuroscript
in: [*, dim]
out: [*, dim]
```

**Named ports (inline):**
```neuroscript
in query: [batch, seq, dim]
out left: [*shape]
out right: [*shape]
```

**Multi-port (indented):**
```neuroscript
in:
  query: [batch, seq, dim]
  key: [batch, seq, dim]
  value: [batch, seq, dim]
out:
  attention: [batch, seq, dim]
```

### Neuron Bodies

**Primitive** — references external implementation:
```neuroscript
impl: core,nn/Linear
```

**Composite** — defines internal connection graph:
```neuroscript
context:
  # optional bindings
graph:
  in -> ... -> out
```

## Graph Connections

### Pipeline Styles

```neuroscript
# Inline
in -> A() -> B() -> out

# Multi-line (indent = implicit ->)
in ->
  A()
  B()
  out

# Named intermediates
in -> A() -> mid
mid -> B() -> out
```

### Tuple Unpacking & Implicit Fork

```neuroscript
# Implicit fork (preferred) — any single output → N-way tuple
in -> (main, skip)                 # 2-way implicit fork
in -> (a, b, c)                    # 3-way implicit fork
in -> (a, b, c, d, e)             # N-way implicit fork (no limit)
(processed, skip) -> Add() -> out  # recombine

# Explicit Fork/Fork3 (for named port access)
in -> Fork() -> (main, skip)       # 2-way with explicit Fork
in -> Fork3() -> (a, b, c)        # 3-way with explicit Fork3
```

Implicit fork: any single-output source can flow into an N-element tuple — the tensor is automatically replicated to all bindings. Explicit Fork/Fork3 are only needed when you want named port access (e.g., `f.left`, `f.right`).

### Port Access

```neuroscript
in -> Fork() -> f
f.left -> Linear(dim, dim) -> processed
f.right -> Identity() -> skip
```

Named ports accessed with dot notation: `<ref>.<port_name>`.

### Frozen Neurons

```neuroscript
in -> Freeze(Linear(dim, dim)) -> out  # parameters not trained
```

## Operators

| Category | Operators |
|----------|-----------|
| Pipeline | `->` |
| Arithmetic | `+`, `-`, `*`, `/` |
| Comparison | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| Logical | `and`, `or` |
| Assignment | `=` |
| Structural | `:`, `,`, `.`, `/` |

## Identifiers

Pattern: `[a-zA-Z_][a-zA-Z0-9_]*`

Cannot be a keyword. Conventions:
- Neuron names: `PascalCase`
- Parameters/dimensions: `snake_case`
- Port names: `snake_case`

## Literals

```neuroscript
512          # integer
-1           # negative integer
3.14         # float
1e-5         # scientific notation
`hello`      # string (backtick-quoted)
true, false  # boolean
```

## Comments

```neuroscript
# Single-line comment
/// Documentation comment (extracted by doc tools)
```

## Indentation

- 2-space or tab indentation (consistent per file)
- Indentation creates block structure
- Blank/comment-only lines ignored during indent tracking
