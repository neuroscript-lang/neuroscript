# Match Expression Partial Optimization Design

## Overview

This document outlines a future optimization for match expressions in NeuroScript: **partial static resolution**. While full compile-time resolution of match expressions is impossible (shapes are runtime values), we can perform partial evaluation when some dimensions are statically known.

## Current Implementation (Phase 0)

Currently, all match expressions generate runtime shape checks:

```python
# Generated code for: in -> match: [*, 512]: Identity() -> out
if x.shape[1] == 512:
    x = self.identity_0(x)
```

Every pattern generates an `if`/`elif` branch evaluated at every forward pass.

## Motivation

Consider this neuron:

```neuroscript
neuron AdaptiveProjection:
  in: [*, *]
  out: [*, 512]

  graph:
    in -> match:
      [*, 512]: Identity() -> out
      [*, d]: Linear(d, 512) -> out
```

If we instantiate with a known input shape:

```neuroscript
neuron MyModel:
  in: [batch, 256]  # Known at compile time
  out: [batch, 512]

  graph:
    in -> AdaptiveProjection() -> out
```

The `batch` dimension is unknown, but `256` is statically known. We could resolve the match at compile time and inline the `Linear(256, 512)` path, eliminating runtime checks.

## Optimization Opportunity

### Case 1: Fully Static Resolution
When all matched dimensions are known at compile time:

```neuroscript
# Input shape: [32, 512] (both literal)
in -> match:
  [*, 512]: Identity() -> out
  [*, d]: Linear(d, 512) -> out
```

**Optimization**: Replace entire match with `Identity()` call. No runtime check needed.

### Case 2: Partial Static Resolution
When some dimensions are known:

```neuroscript
# Input shape: [batch, 256] (second dim known)
in -> match:
  [*, 512]: Identity() -> out
  [*, 256]: Linear(256, 512) -> out
  [*, d]: Linear(d, 512) -> out
```

**Optimization**: Replace match with `Linear(256, 512)` call. No runtime check needed.

### Case 3: Guard Partial Evaluation
When captured dimensions have known values:

```neuroscript
# Input shape: [*, 1024] (second dim known)
in -> match:
  [*, d] where d > 512: Linear(d, 512) -> out
  [*, d]: Linear(d, 256) -> Linear(256, 512) -> out
```

**Optimization**: Evaluate `1024 > 512` at compile time → true. Replace with `Linear(1024, 512)`.

### Case 4: Conservative Fallback
When dimensions are truly runtime-dependent:

```neuroscript
# Input shape: [*, *] (both unknown)
in -> match:
  [*, 512]: Identity() -> out
  [*, d]: Linear(d, 512) -> out
```

**No optimization**: Generate full runtime match expression (current behavior).

## Implementation Strategy

### Phase 1: Shape Constraint Tracking

Extend the shape inference system to track dimension constraints:

```rust
pub enum DimConstraint {
    Literal(usize),           // Dimension is known: 512
    Variable(String),         // Dimension is a variable: batch
    Bounded(String, Range),   // Variable with bounds: d where 256 < d < 1024
    Unknown,                  // Truly unknown: *
}

pub struct InferenceContext {
    // Map from dimension name to constraint
    constraints: HashMap<String, DimConstraint>,
}
```

### Phase 2: Compile-Time Pattern Matching

Add a `try_resolve_match()` function to the shape inference engine:

```rust
pub enum MatchResolution {
    StaticArm(usize),              // Arm N always matches
    RuntimeCheck,                  // Need runtime check
    Impossible,                    // No arm can match (error)
}

impl ShapeInference {
    fn try_resolve_match(&self,
                        match_expr: &MatchExpr,
                        input_shape: &Shape,
                        ctx: &InferenceContext) -> MatchResolution {
        // For each arm:
        //   1. Try to unify pattern with input_shape using constraints
        //   2. If pattern matches and guard is statically true, return StaticArm
        //   3. If pattern definitely doesn't match, skip to next arm
        //   4. If uncertain, return RuntimeCheck
    }
}
```

### Phase 3: Codegen Integration

Modify code generator to use resolution result:

```rust
impl CodeGenerator {
    fn generate_match(&mut self, match_expr: &MatchExpr, input: &str) -> String {
        let resolution = self.shape_inference.try_resolve_match(
            match_expr,
            &self.get_shape(input),
            &self.inference_ctx
        );

        match resolution {
            MatchResolution::StaticArm(arm_idx) => {
                // Generate only the selected arm's pipeline
                let arm = &match_expr.arms[arm_idx];
                self.generate_pipeline(&arm.pipeline, input)
            }
            MatchResolution::RuntimeCheck => {
                // Generate full if/elif chain (current behavior)
                self.generate_runtime_match(match_expr, input)
            }
            MatchResolution::Impossible => {
                panic!("Shape validation should have caught this")
            }
        }
    }
}
```

## Examples

### Example 1: Direct Inlining

**Input**:
```neuroscript
neuron Encoder:
  in: [batch, 256]  # Literal dimension
  out: [batch, 512]

  graph:
    in -> match:
      [*, 256]: Linear(256, 512) -> out
      [*, d]: Linear(d, 512) -> out
```

**Generated (Optimized)**:
```python
class Encoder(nn.Module):
    def __init__(self):
        super().__init__()
        self.linear_0 = Linear(256, 512)  # Statically resolved

    def forward(self, x):
        # No shape check - arm known at compile time
        x = self.linear_0(x)
        return x
```

### Example 2: Guard Evaluation

**Input**:
```neuroscript
neuron Compressor:
  in: [*, 2048]  # Large dimension
  out: [*, 128]

  graph:
    in -> match:
      [*, d] where d > 1024: Linear(d, 512) -> Linear(512, 128) -> out
      [*, d]: Linear(d, 128) -> out
```

**Generated (Optimized)**:
```python
class Compressor(nn.Module):
    def __init__(self):
        super().__init__()
        # Guard evaluated: 2048 > 1024 is true
        self.linear_0 = Linear(2048, 512)
        self.linear_1 = Linear(512, 128)

    def forward(self, x):
        # No shape check or guard evaluation needed
        x = self.linear_0(x)
        x = self.linear_1(x)
        return x
```

### Example 3: Conservative Fallback

**Input**:
```neuroscript
neuron DynamicProjection:
  in: [*, *]  # Both dimensions unknown
  out: [*, 512]

  graph:
    in -> match:
      [*, 512]: Identity() -> out
      [*, d]: Linear(d, 512) -> out
```

**Generated (No Optimization)**:
```python
class DynamicProjection(nn.Module):
    def __init__(self):
        super().__init__()
        self.identity_0 = Identity()
        # Linear instantiated lazily in forward

    def forward(self, x):
        # Full runtime check required
        if x.shape[1] == 512:
            x = self.identity_0(x)
        else:
            d = x.shape[1]
            if not hasattr(self, 'linear_0'):
                self.linear_0 = Linear(d, 512).to(x.device)
            x = self.linear_0(x)
        return x
```

## Challenges and Limitations

### 1. Lazy Module Instantiation
Current implementation uses lazy instantiation for captured dimensions. Optimization requires deciding at compile time which modules to create, which may waste memory for unused arms.

**Mitigation**: Only optimize when it eliminates all runtime checks. If any arm requires lazy instantiation, keep full runtime match.

### 2. Shape Inference Complexity
Tracking constraints through complex graphs is non-trivial. Conservative fallback ensures correctness.

**Mitigation**: Start with simple cases (literal dimensions, direct connections). Extend incrementally.

### 3. Higher-Order Neurons
When neurons are parameterized by other neurons, shape resolution becomes more complex.

**Mitigation**: Defer optimization for higher-order cases until Phase 4 features are implemented.

### 4. Multi-Input Matches
Matching on tuples or multiple inputs requires joint constraint solving.

**Mitigation**: Phase 1 implementation focuses on single-input matches only.

## Testing Strategy

1. **Correctness Tests**: Ensure optimized code produces identical results to unoptimized
2. **Regression Tests**: All existing examples should work unchanged
3. **Optimization Tests**: Verify that expected cases actually get optimized
4. **Negative Tests**: Ensure conservative fallback doesn't break edge cases

## Evaluation Metrics

- **Optimization Rate**: % of match expressions successfully resolved at compile time
- **Runtime Speedup**: Benchmark forward pass with/without optimization
- **Memory Usage**: Track module instantiation overhead
- **Code Size**: Compare generated Python LOC

## Future Extensions

### Dead Code Elimination
If a match arm is proven unreachable at compile time, don't generate its modules:

```neuroscript
# Input: [*, 256]
in -> match:
  [*, 512]: Identity() -> out           # Unreachable
  [*, 256]: Linear(256, 512) -> out     # Always matches
  [*, d]: Linear(d, 512) -> out         # Unreachable
```

Only instantiate `Linear(256, 512)`.

### Partial Guard Evaluation
For guards with mixed static/dynamic terms:

```neuroscript
# Input: [*, known_dim]
in -> match:
  [*, d] where d > 512 and batch > 100: ...
```

Evaluate `known_dim > 512` statically, keep `batch > 100` as runtime check.

### Cross-Neuron Optimization
When a match feeds into another match, propagate constraints:

```neuroscript
neuron Pipeline:
  in: [*, 256]
  out: [*, 128]

  graph:
    in -> AdaptiveProjection() -> Compressor() -> out
```

If `AdaptiveProjection` resolves to `Linear(256, 512)`, feed shape `[*, 512]` to `Compressor`'s match.

## Implementation Timeline

- **Phase 0** (Current): Runtime match expressions only
- **Phase 1** (This Document): Design and planning
- **Phase 2** (Future): Constraint tracking in shape inference
- **Phase 3** (Future): Pattern resolution algorithm
- **Phase 4** (Future): Codegen integration and optimization
- **Phase 5** (Future): Testing and evaluation
- **Phase 6** (Future): Advanced optimizations (dead code elimination, cross-neuron)

## Related Work

- **Dependent Types**: Languages like Idris, Agda perform similar compile-time evaluation
- **Template Metaprogramming**: C++ templates resolve type-level matches at compile time
- **Staged Computation**: Multi-stage programming (MetaML, MetaOCaml) separates compile-time and runtime
- **JIT Compilation**: PyTorch's TorchScript traces and optimizes dynamic shapes

## Conclusion

Partial static resolution of match expressions offers significant performance benefits for common cases where shapes are partially known at compile time. The optimization is entirely optional - conservative fallback to runtime checks ensures correctness. This document provides a roadmap for future implementation while keeping the door open for incremental improvement.
