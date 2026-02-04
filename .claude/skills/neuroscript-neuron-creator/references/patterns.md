# NeuroScript Neuron Patterns

Common patterns for creating neurons in NeuroScript.

## Primitive Neurons

Primitives reference external PyTorch implementations via `impl:` field.

```neuroscript
neuron Linear(in_dim, out_dim):
    in: [*, in_dim]
    out: [*, out_dim]
    impl: core,nn/Linear

neuron ReLU:
    in: [*shape]
    out: [*shape]
    impl: core,activations/ReLU
```

**Impl reference format:** `<provider>,<library>/<class>`
- Example: `core,nn/Linear` → `from neuroscript_runtime.primitives.nn import Linear`
- Example: `core,activations/GELU` → `from neuroscript_runtime.primitives.activations import GELU`

## Composite Neurons

Composite neurons contain a `graph:` section with connections.

```neuroscript
neuron MLP(dim, hidden_dim):
    in: [*, dim]
    out: [*, dim]
    graph:
        in ->
            Linear(dim, hidden_dim)
            ReLU()
            Linear(hidden_dim, dim)
            out
```

## Residual Connections

Use `Fork()` for tuple unpacking and `Add()` to merge paths.

```neuroscript
neuron ResidualBlock(dim):
    in: [*, dim]
    out: [*, dim]
    graph:
        in -> Fork() -> (main, skip)

        main ->
            Linear(dim, dim * 4)
            ReLU()
            Linear(dim * 4, dim)
            processed

        (processed, skip) -> Add() -> out
```

## Match Expressions

Route based on shape patterns with dimension binding.

### Inline syntax

```neuroscript
neuron DimensionRouter(out_dim):
    in: [batch, in_dim]
    out: [batch, out_dim]
    graph:
        in -> match:
            [batch, d] where d > 1024: Linear(d, 512) -> Linear(512, out_dim)
            [batch, d] where d > 256: Linear(d, out_dim)
            [batch, d]: Linear(d, out_dim)
        -> out
```

### Multi-line syntax

```neuroscript
neuron DimensionRouter(out_dim):
    in: [batch, in_dim]
    out: [batch, out_dim]
    graph:
        in -> match:
            [batch, d] where d > 1024:
                Linear(d, 512) ->
                Linear(512, out_dim)
            [batch, d] where d > 256:
                Linear(d, out_dim)
            [batch, d]:
                Linear(d, out_dim)
        -> out
```

**Key points:**
- Captured dimensions (e.g., `d`) can be used in guards and neuron calls
- Match expressions must be exhaustive (use catch-all pattern `[batch, d]:` if needed)
- The match block returns a value that is piped to `out`

## Multi-Head Patterns

Use `Fork3()`, `Fork4()`, etc., for multiple paths.

```neuroscript
neuron ParallelProcessing(dim):
    in: [*, dim]
    out: [*, dim * 3]
    graph:
        in -> Fork3() -> (path1, path2, path3)

        path1 -> Linear(dim, dim) -> p1
        path2 -> Linear(dim, dim) -> p2
        path3 -> Linear(dim, dim) -> p3

        # Concat with dimension parameter takes exactly 2 inputs
        (p1, p2) -> Concat(-1) -> temp
        (temp, p3) -> Concat(-1) -> out
```

**Important**: `Concat(dim)` takes exactly 2 inputs. For 3+ tensors, use pairwise concatenation.

## Multi-Scale Feature Extraction (ASPP Pattern)

Process input through multiple parallel branches at different scales, then combine. Common in semantic segmentation (DeepLab, PSPNet).

```neuroscript
neuron ASPPBlock(in_channels, out_channels):
    in: [batch, in_channels, height, width]
    out: [batch, out_channels * 3, height, width]
    graph:
        # Split into parallel branches
        in -> Fork3() -> (branch1, branch2, branch3)

        # Process at different dilation rates (multi-scale context)
        branch1 -> Conv2d(in_channels, out_channels, 3, dilation=6, padding=6) -> feat1
        branch2 -> Conv2d(in_channels, out_channels, 3, dilation=12, padding=12) -> feat2
        branch3 -> Conv2d(in_channels, out_channels, 3, dilation=18, padding=18) -> feat3

        # Concatenate pairwise (Concat takes 2 inputs)
        (feat1, feat2) -> Concat(1) -> temp
        (temp, feat3) -> Concat(1) -> out
```

**Key points:**
- Fork3 creates 3 independent data paths
- Each branch can use different parameters (dilation, kernel_size, stride, etc.)
- Concat with dimension parameter requires pairwise combination
- Output channels multiply by number of branches
- For 3x3 kernels: `padding = dilation` preserves spatial dimensions

**Receptive field with dilation:**
- Formula: `receptive_field = kernel_size + (kernel_size - 1) * (dilation - 1)`
- 3x3 kernel, dilation=1 → 3×3 receptive field
- 3x3 kernel, dilation=6 → 13×13 receptive field
- 3x3 kernel, dilation=12 → 25×25 receptive field
- 3x3 kernel, dilation=18 → 37×37 receptive field

## Named Ports

Define custom input/output ports.

```neuroscript
neuron Gated(dim):
    in data: [*, dim]
    in gate: [*, dim]
    out: [*, dim]
    graph:
        data -> Linear(dim, dim) -> transformed
        gate -> Sigmoid() -> gate_signal
        (transformed, gate_signal) -> Multiply() -> out
```

## Complete Example: Transformer Block

```neuroscript
neuron TransformerBlock(dim, num_heads, d_ff):
    in: [batch, seq, dim]
    out: [batch, seq, dim]
    graph:
        # First residual-attention with pre-norm
        in -> Fork() -> (skip1, attn_path)

        attn_path ->
            LayerNorm(dim)
            MultiHeadSelfAttention(dim, num_heads)
            Dropout(0.1)
            attn_out

        # Add residual
        (skip1, attn_out) -> Add() -> attn_residual

        # Second residual-forward with pre-norm
        attn_residual -> Fork() -> (skip2, ffn_path)

        ffn_path ->
            LayerNorm(dim)
            FFN(dim, d_ff)
            Dropout(0.1)
            ffn_out

        # Add residual
        (skip2, ffn_out) -> Add() -> out
```
