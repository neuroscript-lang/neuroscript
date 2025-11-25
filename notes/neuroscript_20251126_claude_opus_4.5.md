# Neuroscript — Conversation Summary

*A condensed, shareable summary of the multi-day exploration, design, and refinement of the Neuroscript language.*

## 1. Origin of the Idea

You began with a desire to:

* Explore neural network topology and architecture as *first-class objects*.
* Build a language that lets people assemble AI architectures like LEGO pieces.
* Allow researchers and hobbyists to implement novel papers quickly, without deep PyTorch knowledge.
* Lower the barrier to creativity by turning dataflow graphs into a **textual DSL**.
* Avoid enterprise overhead (containers, CI, infra) in favor of *experimentation-first* design.

This sparked the recognition that **current ML frameworks expose the wrong primitives** and force users to think in terms of layers rather than *flow*.

## 2. The Breakthrough Insight

You reached a key conceptual shift:

* A “block” isn’t the right abstraction.
* A “neuron” **is**: something that can have N inputs, M outputs, and contain a transform that may itself be a graph of neurons.

A neuron can represent:

* A Linear layer
* A Mamba block
* A Transformer
* An entire model
* An LMStudio call
* A gate
* Any recursive composition

Everything becomes a **topologically unconstrained N→M graph**, matching biological neurons and hardware design languages (Verilog/VHDL).

This unified abstraction became the conceptual bedrock for the language.

## 3. Why Existing Approaches Feel Wrong

You walked through the mismatch between modern ML frameworks and what you’re building:

* PyTorch forces imperative wiring.
* Layers conflate computation + shape + topology.
* Most tools treat edges as implicit when they should be **typed, explicit, and smart**.
* Frameworks assume fixed architectures, not exploratory combinatorial topology.

This led to recognizing that your mental model is **graph-first**, while mainstream frameworks are **layer-first**.

## 4. The LEGO-Like Primitives

You identified the eight orthogonal primitives required to build *all* deep learning architectures:

**Transforms**

1. `Map` (elementwise ops)
2. `Project` (Linear, Conv, Embedding lookup)
3. `Reshape` (Einops-style rearrange)

**Topology**
4. `Sequence` (A → B → C)
5. `Parallel` (fan-out)
6. `Merge` (add, concat, attention combine)

**Control**
7. `Gate` (shape/content conditional routing)
8. `Loop` (repeat / recurrence)

Everything else becomes a composite neuron.

## 5. Syntax Principles

A strong emphasis was placed on:

* Minimal syntactic constructs
* High expressive power
* Zero glyph soup
* “Sugar that reshapes cognition,” similar to Clojure’s `->` or Scala’s pattern matching
* Python-style `#` comments
* Backtick-only strings
* Explicitness over magic (e.g., requiring `()` for neuron calls)
* Async-by-default neurons
* Avoiding block comments

You refined piping semantics so that:

```
in ->
  A(args)
  B()
  C
  out
```

is sugar for:

```
in -> A(args) -> B() -> C -> out
```

## 6. Gates, Patterns, and Shape Typing

Shape-based routing became a central novel feature:

```
in -> match:
  [*, 512]: Identity() -> out
  [*, d] where d > 512: Linear(d, 512) -> out
  [*]: LoggingNode(*) -> StopToken() -> out
```

This introduces **dependent typing** and **pattern matching on tensor shape**, a capability that does not exist in PyTorch, TensorFlow, or JAX today.

It is one of the most innovative parts of Neuroscript.

## 7. Rejection of Container Runtime

You decided:

* No containers in the MVP.
* No hardware detection.
* Everything only needs to run on your M2 Max for now.
* Skip complicated enterprise infra and focus on expressiveness.

This opened the path for a *clean, experimental-first* design.

## 8. Decision to Write the Compiler in Rust

Pain with Lark and Python parsing led to a major pivot:

* Rust gives clean IR through enums and algebraic types.
* A hand-built recursive descent parser is simpler and more correct.
* PyO3 bridges Rust → Python for code generation.
* The compiler outputs PyTorch modules.

Rust provides:

* Stability
* Performance
* Error messages that are actually helpful
* Long-term sustainability of tooling

## 9. Current Status

You now have:

* A working Rust project scaffolding
* Lexer/tokenizer with indentation handling
* Parser and IR in place
* Early examples parsed successfully
* A clean language design with strong semantics
* A roadmap toward Python/PyTorch codegen

## 10. Planned Next Steps

1. Finalize grammar (especially around default args, match blocks, pipes).
2. Implement PyTorch codegen in Rust.
3. Build the core neuron stdlib (`Linear`, `Add`, `Fork`, etc.).
4. Create Stack/Repeat/Parallel/Merge combinators.
5. Build shape inference.
6. Build the visualization tools (Mermaid output).
7. Add external neurons (LMStudio, MCP, REST, etc.).

Neuroscript is now a coherent language with a solid conceptual foundation.
