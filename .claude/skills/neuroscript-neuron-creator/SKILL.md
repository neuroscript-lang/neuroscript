---
name: neuroscript-neuron-creator
description: Create NeuroScript neurons for neural architecture composition. Use when the user requests to create, add, implement, or build neurons in NeuroScript, including primitives (with impl references), composites (with graph connections), pattern-based neurons (with match expressions), or any other neuron types. Triggered by phrases like "create a neuron", "add a [component] neuron", "implement [architecture] in NeuroScript", or "build a [layer/block] neuron".
---

# NeuroScript Neuron Creator

Create well-structured neurons for the NeuroScript neural architecture composition language.

## Workflow

### 1. Understand Requirements

Clarify the neuron type and specifications:
- **Primitive**: External PyTorch implementation reference
- **Composite**: Internal graph with connections
- **Pattern-based**: Shape routing with match expressions
- **Residual**: Skip connections with Fork/Add

Key questions:
- What are the input/output shapes?
- What parameters does it need?
- What internal transformations occur?

### 1.5. Check Existing Primitives

Before implementing, check if existing primitives already support your use case:

```bash
# Search stdlib primitives
grep -r "neuron.*YourConcept" stdlib/primitives/
grep -r "your_parameter" neuroscript_runtime/primitives/

# Check stdlib registry
grep -i "keyword" src/stdlib_registry.rs
```

**Decision tree:**
- **Parameter exists**: Use existing primitive with the parameter (e.g., `Conv2d` supports `dilation`)
- **Composable**: Create composite from existing primitives (e.g., ASPP from Conv2d + Fork3 + Concat)
- **Truly new**: Create new primitive with impl reference

### 2. Choose Pattern

Select the appropriate pattern from references/patterns.md:
- Simple composite for sequential operations
- Residual for skip connections
- Match patterns for shape-based routing
- Multi-head for parallel processing paths

### 3. Implement Neuron

Write the neuron definition following syntax rules:

**Primitive neurons:**
```neuroscript
neuron NeuronName(param1, param2):
    in: [shape]
    out: [shape]
    impl: provider,library/Class
```

**Composite neurons:**
```neuroscript
neuron NeuronName(param1, param2):
    in: [shape]
    out: [shape]
    graph:
        in ->
            Layer1(args)
            Layer2(args)
            out
```

**Key syntax rules:**
- Named dimensions must be consistent across connections
- Match expressions must be exhaustive (use catch-all patterns)
- Use Fork() for branching, Add() for combining paths
- Impl references: `provider,library/Class` (no dots in provider/library field)

### 4. Validate

Use the validation script to check all compilation stages:

```bash
scripts/validate_neuron.sh path/to/neuron.ns
```

Or run stages individually:
```bash
./target/release/neuroscript parse neuron.ns
./target/release/neuroscript validate neuron.ns
./target/release/neuroscript compile neuron.ns
```

Fix any errors reported by the compiler.

### 5. Test Runtime Behavior (Optional)

After validation, optionally create a Python test to verify runtime behavior:

```python
# test_my_neuron.py
import torch
import sys
sys.path.insert(0, '/path/to/neuroscript-rs')

from neuroscript_runtime.primitives import *

# Paste generated code or import compiled module
class MyNeuron(torch.nn.Module):
    # ... generated code from compile step ...
    pass

def test_shapes():
    """Test that tensor shapes are correct."""
    neuron = MyNeuron(param1, param2)
    x = torch.randn(batch_size, *input_dims)
    y = neuron(x)

    print(f"Input shape:  {x.shape}")
    print(f"Output shape: {y.shape}")
    assert y.shape == expected_shape, f"Shape mismatch!"
    print("✓ Shape test passed!")

if __name__ == "__main__":
    test_shapes()
```

Run with: `source ~/.venv_ai/bin/activate && python test_my_neuron.py`

**Why test?**
- Validation checks compile-time correctness (syntax, shapes, graph)
- Testing checks runtime correctness (actual tensor operations, numerical behavior)
- Both are valuable for production-ready neurons

## Reference Files

Load these as needed for detailed guidance:

- **references/patterns.md**: Common neuron patterns with examples (primitives, composites, residuals, match expressions, multi-head, transformer blocks)
- **references/stdlib_primitives.md**: Available stdlib primitives (Linear, ReLU, GELU, LayerNorm, Attention, etc.)
- **references/syntax_quick_ref.md**: Syntax rules for shapes, ports, connections, match expressions, guards

## Templates

Use templates from assets/templates/ as starting points:
- **primitive.ns**: Template for primitive neurons with impl references
- **composite.ns**: Template for simple composite neurons
- **residual.ns**: Template for residual connections
- **match_pattern.ns**: Template for shape-based routing

## Common Pitfalls

- **Match expressions**: Must be exhaustive - add catch-all pattern `[*shape]:` or `[batch, d]:`
- **Dimension binding**: Captured dimensions in match patterns can be used in guards and neuron calls
- **Impl format**: Use `provider,library/Class` not `provider.library.Class`
- **Port references**: Use dot notation `neuron.port` (e.g., `fork.a`, `fork.b`)
- **Shape consistency**: Named dimensions must match across all connections
- **Concat arity**: `Concat(dim)` takes exactly 2 inputs. For 3+ tensors, use pairwise concatenation:
  ```neuroscript
  # Wrong - 3 inputs to Concat
  (a, b, c) -> Concat(1) -> out

  # Right - pairwise concatenation
  (a, b) -> Concat(1) -> temp
  (temp, c) -> Concat(1) -> out
  ```
  Without dim parameter, Concat can take tuples: `(a, b, c) -> Concat() -> out`

## Examples

See references/patterns.md for complete examples of:
- MLP with sequential layers
- ResidualBlock with Fork and Add
- DimensionRouter with match expressions
- TransformerBlock with attention and feed-forward paths
