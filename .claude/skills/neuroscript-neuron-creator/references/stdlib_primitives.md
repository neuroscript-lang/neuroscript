# Standard Library Primitives

Quick reference of available primitive neurons in NeuroScript stdlib.

## Core Neural Layers

```neuroscript
neuron Linear(in_features, out_features):
    in: [*, in_features]
    out: [*, out_features]
    impl: core,nn/Linear

neuron Conv1d(in_channels, out_channels, kernel_size):
    in: [batch, in_channels, length]
    out: [batch, out_channels, length]
    impl: core,nn/Conv1d

neuron Embedding(num_embeddings, embedding_dim):
    in: [*, *]
    out: [*, *, embedding_dim]
    impl: core,nn/Embedding
```

## Activations

```neuroscript
neuron ReLU:
    in: [*shape]
    out: [*shape]
    impl: core,activations/ReLU

neuron GELU:
    in: [*shape]
    out: [*shape]
    impl: core,activations/GELU

neuron Sigmoid:
    in: [*shape]
    out: [*shape]
    impl: core,activations/Sigmoid

neuron Tanh:
    in: [*shape]
    out: [*shape]
    impl: core,activations/Tanh

neuron Softmax(dim):
    in: [*shape]
    out: [*shape]
    impl: core,activations/Softmax
```

## Normalization

```neuroscript
neuron LayerNorm(normalized_shape):
    in: [*, normalized_shape]
    out: [*, normalized_shape]
    impl: core,normalization/LayerNorm

neuron BatchNorm1d(num_features):
    in: [batch, num_features, *]
    out: [batch, num_features, *]
    impl: core,normalization/BatchNorm1d

neuron RMSNorm(dim):
    in: [*, dim]
    out: [*, dim]
    impl: core,normalization/RMSNorm
```

## Regularization

```neuroscript
neuron Dropout(p):
    in: [*shape]
    out: [*shape]
    impl: core,regularization/Dropout
```

## Attention

```neuroscript
neuron MultiHeadSelfAttention(d_model, num_heads):
    in: [batch, seq, d_model]
    out: [batch, seq, d_model]
    impl: core,attention/MultiHeadSelfAttention

neuron ScaledDotProductAttention:
    in q: [batch, seq_q, d_k]
    in k: [batch, seq_k, d_k]
    in v: [batch, seq_k, d_v]
    out: [batch, seq_q, d_v]
    impl: core,attention/ScaledDotProductAttention
```

## Structural Operations

```neuroscript
neuron Identity:
    in: [*shape]
    out: [*shape]
    impl: core,builtin/Identity

neuron Fork:
    in: [*shape]
    out a: [*shape]
    out b: [*shape]
    impl: core,builtin/Fork

neuron Fork3:
    in: [*shape]
    out a: [*shape]
    out b: [*shape]
    out c: [*shape]
    impl: core,builtin/Fork3

neuron Add:
    in left: [*shape]
    in right: [*shape]
    out: [*shape]
    impl: core,builtin/Add

neuron Multiply:
    in left: [*shape]
    in right: [*shape]
    out: [*shape]
    impl: core,builtin/Multiply

neuron Concat(dim):
    in: [*shapes]
    out: [*shape]
    impl: core,builtin/Concat
```

## Pooling

```neuroscript
neuron MaxPool1d(kernel_size):
    in: [batch, channels, length]
    out: [batch, channels, length_out]
    impl: core,pooling/MaxPool1d

neuron AvgPool1d(kernel_size):
    in: [batch, channels, length]
    out: [batch, channels, length_out]
    impl: core,pooling/AvgPool1d
```

## Usage Notes

- All primitives use the format `impl: <provider>,<library>/<class>`
- Shape wildcards (`*shape`, `*shapes`) allow any rank tensors
- Named dimensions (e.g., `dim`, `batch`) must be consistent across connections
- Parameters (e.g., `kernel_size`, `p`) are passed to the PyTorch implementation
