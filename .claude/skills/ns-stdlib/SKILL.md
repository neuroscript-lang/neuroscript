---
name: ns-stdlib
description: NeuroScript standard library catalog. Lists all primitive and composite neurons with signatures, shapes, parameters, and categories. Use when looking up available neurons, checking signatures, or finding which neuron to use.
allowed-tools: Read, Grep, Glob, Bash
---

# NeuroScript Standard Library Catalog

## Available Primitives (live)

!`grep -A1 'self\.register(' src/stdlib_registry.rs | grep '"' | sed 's/.*"\([^"]*\)".*/\1/' | sort`

## Composite Library Neurons (live)

!`for f in stdlib/FFN.ns stdlib/Residual.ns stdlib/MultiHeadAttention.ns stdlib/TransformerBlock.ns stdlib/TransformerStack.ns stdlib/MetaNeurons.ns; do [ -f "$f" ] && echo "=== $f ===" && ./target/release/neuroscript list "$f" 2>/dev/null; done`

## Category Index

| Category | Primitives | Use For |
|----------|-----------|---------|
| **Core** | Linear, Bias, Scale, MatMul, Einsum | Dense layers, linear transforms |
| **Activations** | GELU, ReLU, Tanh, Sigmoid, SiLU, Softmax, Mish, PReLU, ELU | Non-linearities |
| **Normalization** | LayerNorm, RMSNorm, GroupNorm, BatchNorm, InstanceNorm | Stabilizing training |
| **Regularization** | Dropout, DropPath, DropConnect | Preventing overfitting |
| **Convolution** | Conv1d, Conv2d, Conv3d, DepthwiseConv, SeparableConv, TransposedConv | Spatial feature extraction |
| **Pooling** | MaxPool, AvgPool, AdaptiveAvgPool, GlobalAvgPool, AdaptiveMaxPool, GlobalMaxPool | Spatial reduction |
| **Embeddings** | Embedding, PositionalEncoding, LearnedPositionalEmbedding, RotaryEmbedding | Token/position encoding |
| **Structural** | Identity, Fork, Fork3, ForkN, Add, Multiply, Concat, Reshape, Transpose, Flatten, Split, Slice, Pad | Routing and reshaping (implicit fork preferred for splitting) |
| **Attention** | ScaledDotProductAttention, MultiHeadSelfAttention | Attention mechanisms |
| **Debug** | Log | Debugging tensor flow |

## Decision Tree: Which Neuron?

**Need to transform features?** → `Linear(in_dim, out_dim)`
**Need non-linearity?** → `GELU()` (default), `ReLU()` (legacy), `SiLU()` (modern)
**Need normalization?** → `LayerNorm(dim)` (transformer), `RMSNorm(dim)` (efficient), `BatchNorm(dim)` (CNN)
**Need residual connection?** → `in -> (main, skip)` + processing + `Add()` (implicit fork)
**Need N-way split?** → `in -> (a, b, c, ...)` (implicit fork — any number of outputs)
**Need to concatenate?** → `Concat(dim=-1)` — takes 2 inputs via named ports
**Need attention?** → `MultiHeadSelfAttention(d_model, heads)` (complete) or compose from `ScaledDotProductAttention(d_k)`
**Need convolution?** → `Conv2d(in_ch, out_ch, kernel)` (standard), `SeparableConv(...)` (efficient)
**Need position info?** → `PositionalEncoding(seq, dim)` (sinusoidal), `RotaryEmbedding(dim, seq)` (modern)

## Standard Library Composites

The `stdlib/` directory provides higher-level neurons built from primitives:

- **FFN.ns** — Feed-forward networks: `FFN(dim, expansion)`, `FFNWithHidden(in, hidden, out)`
- **TransformerBlock.ns** — `SimpleTransformerBlock(dim)`, `TransformerBlock(dim, heads, d_ff)`
- **TransformerStack.ns** — `TransformerStack2(d, heads, d_ff)`, `SequentialTransformer(d, heads, d_ff)`
- **MetaNeurons.ns** — `ParallelFFN(dim)` and routing patterns

See `references/primitives-by-category.md` for full signatures.
See `references/composite-library.md` for stdlib neuron details.
See `references/impl-format.md` for how impl references map to Python.
