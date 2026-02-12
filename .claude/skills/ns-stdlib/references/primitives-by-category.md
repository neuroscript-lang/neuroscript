# Primitives by Category

## Core Operations

| Neuron | Parameters | Input Shape | Output Shape | Description |
|--------|-----------|-------------|--------------|-------------|
| `Linear` | `in_dim, out_dim` | `[*, in_dim]` | `[*, out_dim]` | Dense/fully-connected layer |
| `Bias` | `dim` | `[*, dim]` | `[*, dim]` | Additive bias |
| `Scale` | `dim` | `[*, dim]` | `[*, dim]` | Multiplicative scale |
| `MatMul` | — | 2 inputs | matrix product | Matrix multiplication |
| `Einsum` | `equation` | varies | varies | Einstein summation |

## Activations

All activations: `[*shape] → [*shape]` (shape-preserving)

| Neuron | Parameters | Notes |
|--------|-----------|-------|
| `GELU` | — | Gaussian Error Linear Unit (default for transformers) |
| `ReLU` | — | Rectified Linear Unit (classic) |
| `Tanh` | — | Hyperbolic tangent |
| `Sigmoid` | — | Sigmoid function |
| `SiLU` | — | Sigmoid Linear Unit / Swish |
| `Softmax` | `dim=-1` | Normalizes to probability distribution |
| `Mish` | — | Self-regularizing activation |
| `PReLU` | — | Parametric ReLU (learnable slope) |
| `ELU` | — | Exponential Linear Unit |

## Normalization

| Neuron | Parameters | Input Shape | Output Shape | Notes |
|--------|-----------|-------------|--------------|-------|
| `LayerNorm` | `dim, eps=1e-5` | `[*, dim]` | `[*, dim]` | Used in transformers |
| `RMSNorm` | `dim` | `[*, dim]` | `[*, dim]` | Efficient variant (no mean subtraction) |
| `GroupNorm` | `dim, groups` | `[*, dim]` | `[*, dim]` | Works well with small batches |
| `BatchNorm` | `dim` | `[*, dim]` | `[*, dim]` | Standard batch normalization |
| `InstanceNorm` | `dim` | `[*, dim]` | `[*, dim]` | Per-instance normalization |

## Regularization

All regularization: `[*shape] → [*shape]` (shape-preserving)

| Neuron | Parameters | Notes |
|--------|-----------|-------|
| `Dropout` | `p=0.1` | Standard dropout |
| `DropPath` | `p=0.1` | Stochastic depth (drops entire residual paths) |
| `DropConnect` | `p=0.1` | Drops weights instead of activations |

## Convolutions

| Neuron | Parameters | Notes |
|--------|-----------|-------|
| `Conv1d` | `in_ch, out_ch, kernel` | 1D convolution |
| `Conv2d` | `in_ch, out_ch, kernel` | 2D convolution |
| `Conv3d` | `in_ch, out_ch, kernel` | 3D convolution |
| `DepthwiseConv` | `channels, kernel` | Per-channel convolution |
| `SeparableConv` | `in_ch, out_ch, kernel` | Depthwise + pointwise |
| `TransposedConv` | `in_ch, out_ch, kernel` | Upsampling convolution |

## Pooling

| Neuron | Parameters | Notes |
|--------|-----------|-------|
| `MaxPool` | `kernel` | Maximum value pooling |
| `AvgPool` | `kernel` | Average value pooling |
| `AdaptiveAvgPool` | `output_size` | Fixed output size avg pool |
| `GlobalAvgPool` | — | Reduces spatial to 1x1 |
| `AdaptiveMaxPool` | `output_size` | Fixed output size max pool |
| `GlobalMaxPool` | — | Reduces spatial to 1x1 |

## Embeddings

| Neuron | Parameters | Input Shape | Output Shape | Notes |
|--------|-----------|-------------|--------------|-------|
| `Embedding` | `vocab, dim` | `[*batch]` | `[*batch, dim]` | Token → dense vector |
| `PositionalEncoding` | `seq_len, dim` | `[batch, seq, dim]` | `[batch, seq, dim]` | Sinusoidal (Vaswani) |
| `LearnedPositionalEmbedding` | `seq_len, dim` | `[batch, seq]` | `[batch, seq, dim]` | BERT/GPT style |
| `RotaryEmbedding` | `dim, seq_len` | `[batch, seq, dim]` | `[batch, seq, dim]` | RoPE (modern LLMs) |

## Structural Operations

| Neuron | Parameters | Ports | Shape | Notes |
|--------|-----------|-------|-------|-------|
| `Identity` | — | 1→1 | `[*shape] → [*shape]` | Pass-through no-op |
| `Fork` | — | 1→2 | `[*shape] → (main:[*shape], skip:[*shape])` | 2-way split (prefer implicit fork) |
| `Fork3` | — | 1→3 | `[*shape] → (a:[*shape], b:[*shape], c:[*shape])` | 3-way split (prefer implicit fork) |
| `ForkN` | `n` | 1→N | `[*shape] → N × [*shape]` | Explicit N-way split |
| `Add` | — | 2→1 | `(main:[*shape], skip:[*shape]) → [*shape]` | Element-wise add |
| `Multiply` | — | 2→1 | `(main:[*shape], skip:[*shape]) → [*shape]` | Element-wise multiply |
| `Concat` | `dim=-1` | 2→1 | `(a:[*a], b:[*b]) → [*c]` | Concatenate along dim |
| `Reshape` | `*target_shape` | 1→1 | `[*source] → [*target]` | Reshape (preserves elements) |
| `Transpose` | `dim0, dim1` | 1→1 | `[*shape] → [*shape]` | Permute dimensions |
| `Flatten` | `start_dim, end_dim` | 1→1 | varies | Flatten dim range |
| `Split` | `chunks, dim` | 1→multi | varies | Split into chunks |
| `Slice` | `dim, start, end` | 1→1 | varies | Slice along dim |
| `Pad` | `padding, value` | 1→1 | varies | Pad tensor |

**Important**: Prefer implicit fork (`in -> (a, b, c)`) for splitting — it supports any number of outputs. Explicit Fork/Fork3/ForkN are only needed when you want named port access.

## Attention

| Neuron | Parameters | Ports | Shape |
|--------|-----------|-------|-------|
| `ScaledDotProductAttention` | `d_k` | query, key, value → out | `[*, seq, d_k] × 3 → [*, seq, d_v]` |
| `MultiHeadSelfAttention` | `d_model, heads` | 1→1 | `[batch, seq, d_model] → [batch, seq, d_model]` |

## Debug

| Neuron | Parameters | Shape | Notes |
|--------|-----------|-------|-------|
| `Log` | `label` | `[*shape] → [*shape]` | Prints tensor info, pass-through |
