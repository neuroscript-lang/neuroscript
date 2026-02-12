# Context Bindings

Context bindings define reusable neuron instances within a neuron definition using the `context:` block.

## Annotations

### `@lazy` — Lazy Instantiation

Created on first use in `forward()`. Supports dimension capture from match patterns and recursion.

```neuroscript
context:
  @lazy processor = Linear(d, 512)     # 'd' from match capture
  @lazy recurse = MyNeuron(depth - 1)  # self-reference OK
```

Generated as dictionary-cached modules:
```python
if 'processor' not in self._lazy_cache:
    self._lazy_cache['processor'] = Linear(d, 512)
```

### `@static` — Eager Instantiation

Created once in `__init__()`. Cannot capture match dimensions. Cannot be recursive.

```neuroscript
context:
  @static norm = LayerNorm(dim)     # dim must be a parameter
  @static ffn = FFN(dim, dim * 4)
```

Generated as standard module attributes:
```python
def __init__(self, dim):
    self.norm = LayerNorm(dim)
    self.ffn = FFN(dim, dim * 4)
```

### `@global` — Shared Singleton

Single shared instance across all instances of this neuron type.

```neuroscript
context:
  @global shared_embed = Embedding(vocab, dim)
```

## Recursion Patterns

Recursive neurons require `@lazy` bindings with a parameter that decreases toward a base case:

```neuroscript
neuron RecursiveStack(d_model, num_heads, d_ff, depth):
  in: [*, d_model]
  out: [*, d_model]
  context:
    @lazy recurse = RecursiveStack(d_model, num_heads, d_ff, depth - 1)
    @static norm = LayerNorm(d_model)
  graph:
    in -> match:
      [*] where depth > 0:
        norm
        Linear(d_model, d_model)
        recurse
        out
      [*]:
        Identity()
        out
```

### Recursion Rules

1. **Must use `@lazy`** — `@static` would cause infinite instantiation
2. **Must have base case** — a match arm where `depth == 0` (or similar) stops recursion
3. **Must decrease** — recursive call must modify at least one parameter toward the base case
4. **Parameter must be checkable** — typically an integer compared in a guard

### Invalid Recursion

```neuroscript
# ERROR: @static cannot be recursive
context:
  @static recurse = MyNeuron(depth - 1)

# ERROR: No base case
context:
  @lazy recurse = MyNeuron(depth - 1)
graph:
  in -> recurse -> out  # infinite recursion, no match/guard

# ERROR: Parameter doesn't decrease
context:
  @lazy recurse = MyNeuron(depth)  # same depth = infinite
```

## Binding in Graph

Bindings are referenced by name in the graph section:

```neuroscript
neuron TransformerBlock(dim, heads, d_ff):
  in: [batch, seq, dim]
  out: [batch, seq, dim]
  context:
    @static norm1 = LayerNorm(dim)
    @static norm2 = LayerNorm(dim)
    @static attn = MultiHeadSelfAttention(dim, heads)
    @static ffn = FFN(dim, d_ff)
  graph:
    in -> Fork() -> (main, skip1)
    main -> norm1 -> attn -> attn_out
    (attn_out, skip1) -> Add() -> residual1
    residual1 -> Fork() -> (main2, skip2)
    main2 -> norm2 -> ffn -> ffn_out
    (ffn_out, skip2) -> Add() -> out
```

## Validation Rules

- **No duplicate names** within the same neuron's context block
- **`@static` bindings**: args must be resolvable from neuron parameters only
- **`@lazy` bindings**: args can reference neuron params or match-captured dims
- **Recursive bindings**: must be `@lazy`, must have decreasing parameter, must have base case in graph
