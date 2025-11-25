# NeuroScript (Rust Implementation)

A neural architecture composition language. Neurons all the way down.

## Quick Start

```bash
# Build
cargo build --release

# Parse a file
./target/release/neuroscript examples/residual.ns
```

## Language Overview

```neuroscript
# Everything is a neuron
neuron Linear(in_dim, out_dim):
  in: [*, in_dim]
  out: [*, out_dim]
  impl: core,nn/Linear

# Neurons compose into neurons
neuron MLP(dim):
  in: [*, dim]
  out: [*, dim]
  graph:
    in ->
      Linear(dim, dim * 4)
      GELU()
      Linear(dim * 4, dim)
      out

# Multi-port neurons for branching
neuron Residual(dim):
  in: [*, dim]
  out: [*, dim]
  graph:
    in -> Fork() -> (main, skip)
    main -> MLP(dim) -> processed
    (processed, skip) -> Add() -> out
```

## Syntax Summary

| Construct | Example |
|-----------|---------|
| Pipe | `in -> A() -> B() -> out` |
| Pipeline (indented) | `in ->\n  A()\n  B()\n  out` |
| Tuple | `(a, b)` |
| Port access | `fork.left` |
| Parameters | `neuron Name(p1, p2=default):` |
| Shapes | `[*, dim]`, `[batch, seq, 512]`, `[*shape]` |
| Match | `match:\n  [*, 512]: Identity() -> out` |
| Import | `use core,nn/*` |
| Comment | `# comment` |
| String | `` `backtick string` `` |

## Project Structure

```
neuroscript-rs/
├── Cargo.toml
├── src/
│   ├── lib.rs      # Public API
│   ├── ir.rs       # Intermediate representation types
│   ├── lexer.rs    # Tokenizer with indent handling
│   ├── parser.rs   # Recursive descent parser
│   └── main.rs     # CLI
└── examples/
    └── residual.ns
```

## Architecture

```
Source (.ns) → Lexer → Tokens → Parser → IR → Codegen → PyTorch (.py)
                                          ↓
                                    [Future: ONNX, JAX, etc.]
```

## Why Rust?

1. **Algebraic types** - The IR maps perfectly to Rust enums
2. **Great errors** - miette gives beautiful diagnostics
3. **Fast** - Parses instantly, compiles to native ARM on your M2
4. **PyO3** - Can expose to Python when needed
5. **LLM quality** - Models write better Rust than Python

## Next Steps

1. Add codegen (IR → PyTorch nn.Module)
2. Shape inference/validation
3. PyO3 bindings for Python integration
4. LSP server for editor support

## License

MIT
