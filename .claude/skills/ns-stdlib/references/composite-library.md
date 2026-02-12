# Composite Library Neurons

The `stdlib/` directory contains pre-built composite neurons assembled from primitives.

## FFN.ns — Feed-Forward Networks

### FFN(dim, expansion)
```
in: [*shape, dim] → out: [*shape, dim]
```
Standard feed-forward block: Linear expand → GELU → Linear project back.
- `dim`: input/output feature dimension
- `expansion`: expansion factor (hidden_dim = dim * expansion)

### FFNWithHidden(in_dim, hidden_dim, out_dim)
```
in: [*shape, in_dim] → out: [*shape, out_dim]
```
Explicit hidden dimension variant. Useful when input and output dims differ.

## TransformerBlock.ns — Transformer Layers

### SimpleTransformerBlock(dim)
```
in: [*, dim] → out: [*, dim]
```
Minimal transformer: LayerNorm → Linear → Dropout. Useful as building block.

### TransformerBlock(dim, num_heads, d_ff)
```
in: [batch, seq, dim] → out: [batch, seq, dim]
```
Full pre-norm transformer block with:
1. Fork → LayerNorm → MultiHeadSelfAttention → Add (residual)
2. Fork → LayerNorm → FFN(dim, d_ff) → Add (residual)

## TransformerStack.ns — Stacked Transformers

### TransformerStack2(d_model, num_heads, d_ff)
```
in: [*, d_model] → out: [*, d_model]
```
Two sequential SimpleTransformerBlock layers.

### SequentialTransformer(d_model, num_heads, d_ff)
```
in: [*, d_model] → out: [*, d_model]
```
Single SimpleTransformerBlock (base case for stacking).

## MetaNeurons.ns — Routing and Composition

### ParallelFFN(dim)
```
in: [*, dim] → out: [*, dim]
```
FFN with `dim * 2` expansion factor.

## Using Stdlib Neurons

Import and use in your neurons:

```neuroscript
use stdlib,FFN/*

neuron MyBlock(dim):
  in: [*, dim]
  out: [*, dim]
  graph:
    in -> FFN(dim, 4) -> out
```

Or reference them directly in composite graphs — the compiler resolves stdlib neurons automatically when they're defined in `.ns` files passed to the compiler.
