#!/usr/bin/env python3
"""
generate-index.py — Tree-sitter source indexer + call graph generator

Produces:
  .claude/context/source-index.md  — Function/type catalog with line numbers
  .claude/context/call-graph.md    — Caller/callee cross-references

Usage:
  python3 scripts/generate-index.py <project_dir> <context_dir>
"""

import sys
import os
from pathlib import Path
from collections import defaultdict

import tree_sitter_rust as ts_rust
from tree_sitter import Language, Parser

RUST_LANGUAGE = Language(ts_rust.language())


def create_parser():
    parser = Parser(RUST_LANGUAGE)
    return parser


def get_rs_files(src_dir: Path):
    """Find all .rs files under src/, excluding test-only files."""
    files = sorted(src_dir.rglob("*.rs"))
    return files


def get_visibility(node):
    """Extract visibility from a node's children."""
    for child in node.children:
        if child.type == "visibility_modifier":
            text = child.text.decode("utf-8")
            if text == "pub":
                return "pub"
            elif "crate" in text:
                return "pub(crate)"
            return text
    return ""


def get_name(node, name_type="name"):
    """Extract name from a node's named children."""
    for child in node.children:
        if child.type == name_type:
            return child.text.decode("utf-8")
        if child.type == "type_identifier":
            return child.text.decode("utf-8")
        if child.type == "identifier":
            return child.text.decode("utf-8")
    return None


def get_function_name(node):
    """Extract function name from function_item."""
    for child in node.children:
        if child.type == "identifier":
            return child.text.decode("utf-8")
    return None


def get_params_summary(node):
    """Extract a brief parameter summary from a function."""
    for child in node.children:
        if child.type == "parameters":
            params = []
            for param in child.children:
                if param.type == "parameter":
                    # Get just the parameter name
                    for pc in param.children:
                        if pc.type == "identifier":
                            params.append(pc.text.decode("utf-8"))
                            break
                elif param.type == "self_parameter":
                    params.append("self")
            return ", ".join(params)
    return ""


def is_test_module(node):
    """Check if a node is inside a #[cfg(test)] module."""
    # Check for cfg(test) attribute on the node or parent
    prev = node.prev_sibling
    while prev is not None:
        if prev.type == "attribute_item":
            text = prev.text.decode("utf-8")
            if "cfg(test)" in text or "cfg(feature" in text:
                return True
        elif prev.type != "line_comment" and prev.type != "block_comment":
            break
        prev = prev.prev_sibling
    return False


SKIP_CALLS = {
    "format", "println", "eprintln", "write", "writeln", "print",
    "unwrap", "expect", "clone", "into", "from", "new",
    "push", "insert", "contains", "get", "iter", "map",
    "filter", "collect", "join", "len", "is_empty",
    "to_string", "as_str", "to_owned", "default",
    "ok_or", "ok_or_else", "map_err", "and_then",
    "Some", "None", "Ok", "Err", "Box",
    "write_str", "fmt", "any", "all", "find",
    "enumerate", "flat_map", "for_each", "zip",
    "with_capacity", "entry", "or_insert", "or_insert_with",
    "cloned", "copied", "flatten", "take", "skip",
    "as_ref", "as_mut", "borrow", "borrow_mut",
    "display", "trim", "split", "starts_with", "ends_with",
    "replace", "chars", "bytes", "lines",
    "ok", "err", "is_some", "is_none", "is_ok", "is_err",
    "unwrap_or", "unwrap_or_else", "unwrap_or_default",
    "extend", "drain", "retain", "sort", "sort_by",
    "first", "last", "nth", "count", "sum", "max", "min",
    "position", "rev", "peekable", "chain", "cycle",
    "read_to_string", "write_all", "flush",
}


def extract_call_targets(node, seen=None):
    """Extract function/method call targets from a function body.

    Only captures:
    - Direct function calls: foo(), Self::bar()
    - Path-based calls: crate::module::func()
    Skips:
    - Method chains on variables: x.foo().bar()
    - Common std library methods
    """
    if seen is None:
        seen = set()
    targets = []

    if node.type == "call_expression":
        func = node.children[0] if node.children else None
        if func:
            # Only capture simple identifiers or scoped paths (not method chains)
            if func.type == "identifier":
                name = func.text.decode("utf-8")
                if name not in seen and name not in SKIP_CALLS and not name.startswith("vec"):
                    seen.add(name)
                    targets.append(name)
            elif func.type == "scoped_identifier":
                # Self::method or Module::func
                text = func.text.decode("utf-8")
                parts = text.split("::")
                name = parts[-1]
                if name not in seen and name not in SKIP_CALLS:
                    seen.add(name)
                    targets.append(name)

    for child in node.children:
        targets.extend(extract_call_targets(child, seen))

    return targets


def index_file(parser, filepath: Path, src_dir: Path):
    """Parse a single .rs file and extract definitions + calls."""
    source = filepath.read_bytes()
    tree = parser.parse(source)
    root = tree.root_node

    rel_path = filepath.relative_to(src_dir.parent)
    line_count = source.count(b"\n") + 1

    definitions = []  # (kind, vis, name, start_line, end_line, params)
    calls = {}  # func_name -> [callee_names]
    current_impl = None

    def walk(node, depth=0, in_test=False):
        nonlocal current_impl

        # Check for test module
        if node.type == "mod_item" and is_test_module(node):
            in_test = True

        if in_test:
            # Skip test content for definitions, but we still recurse
            for child in node.children:
                walk(child, depth + 1, in_test=True)
            return

        if node.type == "struct_item":
            vis = get_visibility(node)
            name = get_name(node, "type_identifier")
            if name and vis:
                definitions.append((
                    "struct", vis, name,
                    node.start_point[0] + 1,
                    node.end_point[0] + 1,
                    None
                ))

        elif node.type == "enum_item":
            vis = get_visibility(node)
            name = get_name(node, "type_identifier")
            if name and vis:
                # Extract variant names
                variants = []
                for child in node.children:
                    if child.type == "enum_variant_list":
                        for vc in child.children:
                            if vc.type == "enum_variant":
                                vname = get_name(vc)
                                if vname:
                                    variants.append(vname)
                definitions.append((
                    "enum", vis, name,
                    node.start_point[0] + 1,
                    node.end_point[0] + 1,
                    variants
                ))

        elif node.type == "type_item":
            vis = get_visibility(node)
            name = get_name(node, "type_identifier")
            if name and vis:
                definitions.append((
                    "type", vis, name,
                    node.start_point[0] + 1,
                    node.end_point[0] + 1,
                    None
                ))

        elif node.type == "impl_item":
            # Get impl target type
            for child in node.children:
                if child.type == "type_identifier":
                    current_impl = child.text.decode("utf-8")
                    break
            for child in node.children:
                walk(child, depth + 1, in_test)
            current_impl = None
            return

        elif node.type == "function_item":
            vis = get_visibility(node)
            name = get_function_name(node)
            if name:
                params = get_params_summary(node)
                prefix = f"impl {current_impl}" if current_impl else ""
                if vis or current_impl:
                    definitions.append((
                        "fn", vis, name,
                        node.start_point[0] + 1,
                        node.end_point[0] + 1,
                        (prefix, params)
                    ))

                # Extract call targets from function body
                for child in node.children:
                    if child.type == "block":
                        targets = extract_call_targets(child)
                        if targets:
                            qualified = f"{current_impl}::{name}" if current_impl else name
                            calls[qualified] = targets
                return  # Don't recurse into function body again

        for child in node.children:
            walk(child, depth + 1, in_test)

    walk(root)

    return {
        "path": str(rel_path),
        "lines": line_count,
        "definitions": definitions,
        "calls": calls,
    }


def generate_source_index(file_data, output_path: Path):
    """Generate source-index.md from parsed file data."""
    with open(output_path, "w") as f:
        f.write("# Source Index\n")
        f.write("<!-- Auto-generated by scripts/generate-index.py — do not edit -->\n\n")
        f.write("Function/type catalog with line numbers. Use this to jump to any definition.\n\n")

        for data in file_data:
            if not data["definitions"]:
                continue

            f.write(f"## {data['path']} ({data['lines']} lines)\n")

            # Group by impl block
            current_impl_block = None
            for kind, vis, name, start, end, extra in data["definitions"]:
                if kind == "fn" and extra:
                    impl_name, params = extra
                    if impl_name and impl_name != current_impl_block:
                        current_impl_block = impl_name
                        f.write(f"  {impl_name}:\n")
                    elif not impl_name and current_impl_block:
                        current_impl_block = None

                    vis_str = f"{vis} " if vis else ""
                    f.write(f"    {vis_str}fn {name}({params}) [L{start}-L{end}]\n")

                elif kind == "struct":
                    current_impl_block = None
                    f.write(f"  {vis} struct {name} [L{start}-L{end}]\n")

                elif kind == "enum":
                    current_impl_block = None
                    if extra:
                        variants_str = ", ".join(extra[:8])
                        if len(extra) > 8:
                            variants_str += f", ... (+{len(extra)-8})"
                        f.write(f"  {vis} enum {name} {{ {variants_str} }} [L{start}-L{end}]\n")
                    else:
                        f.write(f"  {vis} enum {name} [L{start}-L{end}]\n")

                elif kind == "type":
                    current_impl_block = None
                    f.write(f"  {vis} type {name} [L{start}-L{end}]\n")

            f.write("\n")


def generate_call_graph(file_data, output_path: Path):
    """Generate call-graph.md from parsed file data."""
    # Build global function index: name -> (file, line)
    func_index = {}
    for data in file_data:
        for kind, vis, name, start, end, extra in data["definitions"]:
            if kind == "fn":
                # Store both bare name and qualified name
                qualified = None
                if extra and extra[0]:
                    qualified = f"{extra[0].replace('impl ', '')}::{name}"
                    func_index[qualified] = (data["path"], start)
                func_index[name] = (data["path"], start)

    # Build call maps
    all_calls = {}  # qualified_name -> (file, line, [callees])
    called_by = defaultdict(list)  # callee -> [callers]

    for data in file_data:
        for caller, callees in data["calls"].items():
            # Find caller location
            bare_name = caller.split("::")[-1]
            loc = func_index.get(caller) or func_index.get(bare_name)
            if loc:
                all_calls[caller] = (loc[0], loc[1], callees)
                for callee in callees:
                    called_by[callee].append((caller, loc[0], loc[1]))

    with open(output_path, "w") as f:
        f.write("# Call Graph\n")
        f.write("<!-- Auto-generated by scripts/generate-index.py — do not edit -->\n\n")
        f.write("Caller/callee cross-references for public functions.\n\n")

        # Output by file, only public functions with interesting call relationships
        for data in file_data:
            file_entries = []
            for caller, (cfile, cline, callees) in all_calls.items():
                if cfile == data["path"]:
                    # Only include if there are resolved callees
                    resolved = []
                    for callee in callees:
                        if callee in func_index:
                            cloc = func_index[callee]
                            resolved.append(f"{callee} ({cloc[0]}:{cloc[1]})")
                        else:
                            resolved.append(callee)
                    if resolved:
                        file_entries.append((caller, cline, resolved))

            if not file_entries:
                continue

            f.write(f"## {data['path']}\n")
            for caller, line, resolved_callees in sorted(file_entries, key=lambda x: x[1]):
                f.write(f"  {caller} (L{line})\n")
                # Show calls (limit to keep output manageable)
                shown = resolved_callees[:10]
                f.write(f"    calls: {', '.join(shown)}\n")
                if len(resolved_callees) > 10:
                    f.write(f"    ... and {len(resolved_callees) - 10} more\n")
                # Show called_by
                bare = caller.split("::")[-1]
                callers = called_by.get(bare, []) + called_by.get(caller, [])
                if callers:
                    unique_callers = []
                    seen = set()
                    for c, cf, cl in callers:
                        if c not in seen and c != caller:
                            seen.add(c)
                            unique_callers.append(f"{c} ({cf}:{cl})")
                    if unique_callers:
                        f.write(f"    called_by: {', '.join(unique_callers[:5])}\n")
            f.write("\n")


def main():
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <project_dir> <context_dir>", file=sys.stderr)
        sys.exit(1)

    project_dir = Path(sys.argv[1])
    context_dir = Path(sys.argv[2])
    src_dir = project_dir / "src"

    if not src_dir.exists():
        print(f"ERROR: {src_dir} not found", file=sys.stderr)
        sys.exit(1)

    parser = create_parser()
    rs_files = get_rs_files(src_dir)

    print(f"Indexing {len(rs_files)} .rs files...", file=sys.stderr)

    file_data = []
    for filepath in rs_files:
        try:
            data = index_file(parser, filepath, src_dir)
            file_data.append(data)
        except Exception as e:
            print(f"WARNING: Failed to parse {filepath}: {e}", file=sys.stderr)

    generate_source_index(file_data, context_dir / "source-index.md")
    print(f"Generated: {context_dir / 'source-index.md'}", file=sys.stderr)

    generate_call_graph(file_data, context_dir / "call-graph.md")
    print(f"Generated: {context_dir / 'call-graph.md'}", file=sys.stderr)


if __name__ == "__main__":
    main()
