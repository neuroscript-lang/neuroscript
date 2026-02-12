# Implementation Reference Format

## Format

```neuroscript
impl: <provider>,<module_path>/<ClassName>
```

**Not** dot notation. Uses comma to separate provider from path, slash to separate path from class.

## Examples

| impl reference | Python import generated |
|---------------|------------------------|
| `core,nn/Linear` | `from neuroscript_runtime.primitives.linear import Linear` |
| `core,attention/ScaledDotProductAttention` | `from neuroscript_runtime.primitives.attention import ScaledDotProductAttention` |
| `core,structural/Fork` | `from neuroscript_runtime.primitives.structural import Fork` |
| `core,activations/GELU` | `from neuroscript_runtime.primitives.activations import GELU` |
| `core,normalization/LayerNorm` | `from neuroscript_runtime.primitives.normalization import LayerNorm` |

## Provider Categories

The `core` provider maps to `neuroscript_runtime.primitives.<category>`:

| Module Path | Python Module | Contains |
|------------|---------------|----------|
| `nn` | `linear` | Linear |
| `operations` | `operations` | Bias, Scale, MatMul, Einsum |
| `activations` | `activations` | GELU, ReLU, Tanh, Sigmoid, SiLU, Softmax, Mish, PReLU, ELU |
| `normalization` | `normalization` | LayerNorm, RMSNorm, GroupNorm, BatchNorm, InstanceNorm |
| `regularization` | `regularization` | Dropout, DropPath, DropConnect |
| `convolutions` | `convolutions` | Conv1d, Conv2d, Conv3d, DepthwiseConv, SeparableConv, TransposedConv |
| `pooling` | `pooling` | MaxPool, AvgPool, AdaptiveAvgPool, GlobalAvgPool, AdaptiveMaxPool, GlobalMaxPool |
| `embeddings` | `embeddings` | Embedding, PositionalEncoding, LearnedPositionalEmbedding, RotaryEmbedding |
| `structural` | `structural` | Identity, Fork, Fork3, Add, Multiply, Concat, Reshape, Transpose, Flatten, Split, Slice, Pad |
| `attention` | `attention` | ScaledDotProductAttention, MultiHeadSelfAttention |
| `logging` | `logging` | Log |

## External Implementations

For non-stdlib implementations:

```neuroscript
impl: external(module=`torch.nn`, class=`Dropout`, p=0.1)
```

The `external` variant passes kwargs directly to the Python class constructor.

## How the Registry Works

1. Compiler encounters `impl: core,nn/Linear`
2. Looks up `"Linear"` in `StdlibRegistry`
3. Registry returns `ImplRef::Source { source: "neuroscript_runtime.primitives.linear", path: "Linear" }`
4. Codegen emits `from neuroscript_runtime.primitives.linear import Linear`

The registry is defined in `src/stdlib_registry.rs` with all 45+ primitives pre-registered.
