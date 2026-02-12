# Neuron Patterns

## Pattern 1: Simple Pipeline

Linear sequence of operations:

```neuroscript
neuron MLP(dim):
  in: [*, dim]
  out: [*, dim]
  graph:
    in ->
      Linear(dim, dim * 4)
      GELU()
      Linear(dim * 4, dim)
      out
```

## Pattern 2: Residual Connection

Implicit fork → process → Add:

```neuroscript
neuron ResidualMLP(dim):
  in: [*, dim]
  out: [*, dim]
  graph:
    in -> (main, skip)
    main ->
      LayerNorm(dim)
      Linear(dim, dim * 4)
      GELU()
      Linear(dim * 4, dim)
      Dropout(0.1)
      processed
    (processed, skip) -> Add() -> out
```

## Pattern 3: Double Residual (Transformer Block)

Two sequential residual blocks:

```neuroscript
neuron TransformerBlock(dim, heads, d_ff):
  in: [batch, seq, dim]
  out: [batch, seq, dim]
  context:
    @static norm1 = LayerNorm(dim)
    @static norm2 = LayerNorm(dim)
    @static attn = MultiHeadSelfAttention(dim, heads)
  graph:
    # Residual 1: attention
    in -> (main1, skip1)
    main1 -> norm1 -> attn -> attn_out
    (attn_out, skip1) -> Add() -> res1

    # Residual 2: FFN
    res1 -> (main2, skip2)
    main2 ->
      norm2
      Linear(dim, d_ff)
      GELU()
      Linear(d_ff, dim)
      ffn_out
    (ffn_out, skip2) -> Add() -> out
```

## Pattern 4: Shape-Based Routing

Different processing based on input dimensions:

```neuroscript
neuron AdaptiveProjection(target):
  in: [*, target]
  out: [*, target]
  graph:
    in -> match:
      [*, d] where d == target: Identity() -> out
      [*, d] where d > target: Linear(d, target) -> out
      [*, d]: Linear(d, target) -> GELU() -> out
```

## Pattern 5: Recursive Stack

Self-referencing with depth control:

```neuroscript
neuron RecursiveEncoder(dim, depth):
  in: [*, dim]
  out: [*, dim]
  context:
    @lazy recurse = RecursiveEncoder(dim, depth - 1)
    @static norm = LayerNorm(dim)
  graph:
    in -> match:
      [*] where depth > 0:
        norm
        Linear(dim, dim)
        GELU()
        recurse
        out
      [*]:
        Identity() -> out
```

## Pattern 6: Multi-Port Neuron

Multiple named input/output ports:

```neuroscript
neuron CrossAttentionBlock(dim, heads):
  in:
    query: [batch, seq_q, dim]
    context: [batch, seq_k, dim]
  out: [batch, seq_q, dim]
  graph:
    query -> Linear(dim, dim) -> q_proj
    context -> (k_input, v_input)
    k_input -> Linear(dim, dim) -> k_proj
    v_input -> Linear(dim, dim) -> v_proj
    (q_proj, k_proj, v_proj) -> ScaledDotProductAttention(dim / heads) -> out
```

## Pattern 7: Gated Connection

Using Multiply for gating:

```neuroscript
neuron GatedMLP(dim):
  in: [*, dim]
  out: [*, dim]
  graph:
    in -> (gate_path, value_path)
    gate_path -> Linear(dim, dim) -> Sigmoid() -> gate
    value_path -> Linear(dim, dim) -> GELU() -> value
    (gate, value) -> Multiply() -> Linear(dim, dim) -> out
```

## Pattern 8: Convolution Block

Conv + Norm + Activation:

```neuroscript
neuron ConvBlock(in_ch, out_ch, kernel):
  in: [batch, in_ch, height, width]
  out: [batch, out_ch, height, width]
  graph:
    in ->
      Conv2d(in_ch, out_ch, kernel)
      BatchNorm(out_ch)
      ReLU()
      out
```
