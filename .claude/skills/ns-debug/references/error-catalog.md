# Error Catalog

All 10 ValidationError variants from `src/interfaces.rs`.

## 1. MissingNeuron

```
Neuron '<name>' not found (in <context>)
```

**Cause**: A connection references a neuron that doesn't exist in the current scope — not defined in the file, not in stdlib registry, not imported.

**Fix**:
- Check spelling (case-sensitive: `LayerNorm` not `Layernorm`)
- Add `use` import for external neurons
- Define the neuron in the same file
- Check the primitives list: `grep -A1 'self\.register(' src/stdlib_registry.rs | grep '"' | sed 's/.*"\([^"]*\)".*/\1/' | sort`

## 2. PortMismatch

```
Port mismatch: <source> <shape> -> <dest> <shape> (in <context>)
Suggestion: check if dimensions match or if a transpose/reshape is needed.
```

**Cause**: The output shape of the source doesn't match the expected input shape of the destination.

**Fix**:
- Verify dimension names are consistent across connections
- Check parameter order (e.g., `Linear(in_dim, out_dim)` — first param is input dim)
- Add Reshape/Transpose if dimensions need reordering
- If using named dims, ensure the same name means the same value everywhere

## 3. CycleDetected

```
Cycle detected in <context>: A -> B -> C -> A
```

**Cause**: Neurons form a circular dependency (A calls B, B calls C, C calls A).

**Fix**:
- Break the cycle by restructuring the dependency graph
- For intended recursion: use `@lazy` context binding with a depth parameter and base-case guard
- Self-edges within a single neuron's graph are allowed (pipeline connections)

## 4. ArityMismatch

```
Arity mismatch: expected <n> ports, got <m> (in <context>)
```

**Cause**: Tuple unpacking size doesn't match the number of outputs from the source neuron (for multi-output sources).

**Fix**:
- Single-output sources support implicit fork to any tuple size: `in -> (a, b, c, d)` is valid
- Multi-output sources must match: Fork → 2 outputs `(a, b)`, Fork3 → 3 outputs `(a, b, c)`
- Add → 2 inputs: use `(main, skip) -> Add()`
- Prefer implicit fork over explicit Fork/Fork3 when you don't need named ports

## 5. UnknownNode

```
Unknown node '<name>' (in <context>)
```

**Cause**: A reference to a variable or binding that hasn't been defined.

**Fix**:
- Ensure tuple unpacking creates the reference: `in -> Fork() -> (main, skip)` before using `main`
- Check context bindings define the name
- Check for typos in reference names
- Named references must be defined before use in the graph

## 6. NonExhaustiveMatch

```
Non-exhaustive match expression (in <context>): <suggestion>
```

**Cause**: No catch-all pattern — all match arms have specific patterns or guards, so some inputs might not match anything.

**Fix**:
- Add a final arm with no guard:
  ```neuroscript
  [*, d]: Linear(d, 512) -> out     # catches any rank-2+ tensor
  [*]: Identity() -> out             # catches anything
  ```
- The catch-all must be the last arm

## 7. UnreachableMatchArm

```
Unreachable match arm <index> shadowed by arm <shadowed_by> (in <context>)
```

**Cause**: An earlier match arm already catches all inputs that this arm would match.

**Fix**:
- Reorder patterns: most specific first, most general last
- Check if earlier patterns with guards still leave room for later patterns
- Example: `[*, d]` (no guard) catches everything — any arm after it is unreachable

## 8. DuplicateBinding

```
Duplicate binding '<name>' in neuron '<neuron>'
```

**Cause**: Two context bindings with the same name in the same neuron.

**Fix**:
- Rename one of the bindings
- Each binding name must be unique within a neuron's context block

## 9. InvalidRecursion

```
Invalid recursion in binding '<binding>' in neuron '<neuron>': <reason>
```

**Cause**: Recursive binding violates recursion rules.

**Common reasons**:
- Using `@static` for recursive binding (must be `@lazy`)
- No base case in the graph (infinite recursion)
- Parameter doesn't decrease toward base case

**Fix**:
- Change `@static` to `@lazy`
- Add match expression with depth guard: `where depth > 0`
- Add base case arm: `[*]: Identity() -> out`
- Ensure recursive call decreases a parameter: `depth - 1`

## 10. Custom

```
<message>
```

**Cause**: Catch-all for other validation issues not covered by specific variants.

**Fix**: Read the message carefully — it describes the specific issue.
