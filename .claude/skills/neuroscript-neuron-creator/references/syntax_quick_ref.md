# NeuroScript Syntax Quick Reference

## Neuron Definition

```neuroscript
neuron NeuronName(param1, param2):
    in: [shape]
    out: [shape]
    graph:
        # connections...
```

Or for primitives:

```neuroscript
neuron NeuronName(param1):
    in: [shape]
    out: [shape]
    impl: provider,library/Class
```

## Shapes

- **Literal dimensions:** `[512, 256]`
- **Named dimensions:** `[batch, seq, dim]`
- **Single wildcard:** `[*, dim]` (matches one dimension)
- **Variadic wildcard:** `[*shape]` (matches any number of dimensions)
- **Expressions:** `[dim * 4]`, `[seq - 1]`, `[d + 512]`

## Ports

- **Default ports:** `in` and `out` (no declaration needed)
- **Named input:** `in name: [shape]`
- **Named output:** `out name: [shape]`
- **Port reference:** `neuron.port` (e.g., `fork.a`, `fork.b`)

## Connections

### Simple pipeline

```neuroscript
in -> Linear(dim, dim * 4) -> out
```

### Multi-line pipeline

```neuroscript
in ->
    Linear(dim, dim * 4)
    ReLU()
    Linear(dim * 4, dim)
    out
```

### Tuple unpacking

```neuroscript
in -> Fork() -> (branch1, branch2)
branch1 -> Linear(dim, dim) -> b1
branch2 -> Linear(dim, dim) -> b2
(b1, b2) -> Add() -> out
```

### Named intermediate

```neuroscript
in -> Linear(dim, dim) -> hidden
hidden -> ReLU() -> out
```

## Match Expressions

### Basic pattern

```neuroscript
in -> match:
    [*, d] where d > 512: Linear(d, 512) -> out
    [*, d]: Identity() -> out
```

### Multi-line arms

```neuroscript
in -> match:
    [*, d] where d > 512:
        Linear(d, 512) ->
        ReLU() ->
        out
    [*, d]:
        Identity() -> out
```

### Dimension binding

Captured dimensions (e.g., `d`, `seq`) can be:
- Used in guards: `where d > 512`
- Passed to neurons: `Linear(d, 512)`
- Used in expressions: `Linear(d, d * 4)`

## Operators

- `->` : Pipeline operator
- `,` : Separator in tuples and arguments
- `()` : Grouping for tuple unpacking
- `where` : Guard condition in match expressions

## Guard Conditions

- **Comparison:** `d > 512`, `d <= 256`, `d == 512`
- **Logical:** `s > 1 && d >= 64`, `d < 128 || d > 1024`

## Comments

```neuroscript
# Single-line comment

/// Documentation comment for neuron
neuron MyNeuron(dim):
    in: [*, dim]
    out: [*, dim]
    graph:
        # Implementation comment
        in -> out
```
