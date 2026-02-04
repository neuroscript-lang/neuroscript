# NeuroScript Package Management

This document describes the package management system for NeuroScript, modeled after Cargo.

## Current Status: Phase 2 - Registry Infrastructure ✅

### Phase 1: Local Package Support ✅ (Complete)
- Axon.toml manifest parsing
- Package initialization (`neuroscript init`)
- Data structures for dependencies

### Phase 2: Registry Infrastructure ✅ (Complete)
- Lockfile (Axon.lock) generation and management
- Dependency resolution with semver constraints
- Git dependency fetching (URL, branch, tag, rev)
- Path dependency resolution for local development
- Cache management (~/.neuroscript/)
- `neuroscript add` command
- `neuroscript fetch` command

## Architecture

### Manifest Format (Axon.toml)

Packages are described using TOML format (not YAML) for consistency with Cargo:

```toml
[package]
name = "attention-mechanisms"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
license = "MIT"
description = "Self-attention neurons for transformer architectures"
repository = "https://github.com/user/attention-mechanisms"

# What neurons this package provides
neurons = [
    "MultiHeadAttention",
    "ScaledDotProductAttention",
    "CrossAttention"
]

[dependencies]
# Git repos with version constraints (Cargo-style)
core-primitives = "1.2.0"
residual-blocks = { version = "0.3", git = "https://github.com/org/residual" }

# Or local path for development
# residual-blocks = { path = "../residual-blocks" }

[python-runtime]
# Python dependencies this package needs (for impl: references)
requires = ["torch>=2.0", "einops>=0.6"]

[security]
# Optional: cryptographic hash of neurons for verification
# Generated during `axon publish`
checksum = "sha256:abc123..."
```

### Package Structure

A typical NeuroScript package has this structure:

```
my-package/
├── Axon.toml              # Package manifest
├── README.md              # Package documentation
├── .gitignore             # Git ignore rules
└── src/
    ├── neuron1.ns         # NeuroScript source files
    └── neuron2.ns
```

With `--bin` flag:
```
my-package/
├── Axon.toml
├── README.md
├── .gitignore
├── src/
│   └── *.ns
└── examples/
    └── usage.py           # Example Python usage
```

## Implementation Details

### Module Structure

```
src/package/
├── mod.rs         # Module declarations and re-exports
├── manifest.rs    # Axon.toml parsing and validation
└── init.rs        # Package initialization logic
```

### Key Types

**`Manifest`** - Root structure of Axon.toml
- `package: PackageMetadata` - Package metadata
- `neurons: Vec<String>` - Neurons provided by this package
- `dependencies: HashMap<String, Dependency>` - Dependencies
- `python_runtime: Option<PythonRuntime>` - Python requirements
- `security: Option<Security>` - Security metadata

**`Dependency`** - Dependency specification
- `Simple(String)` - Version string like "1.2.0"
- `Detailed(DependencyDetail)` - Full spec with git/path/version

**`PackageMetadata`** - Package information
- Name, version, authors, license, description
- Repository, homepage, documentation URLs
- Keywords and categories for discoverability

### CLI Commands

#### `neuroscript init`

Initialize a new NeuroScript package:

```bash
# Create new package
neuroscript init my-neurons

# With options
neuroscript init attention-mechanisms \
  --author "Your Name <you@example.com>" \
  --license MIT \
  --version 0.1.0

# Create with examples directory
neuroscript init my-neurons --bin

# Create in specific directory
neuroscript init my-neurons --path ./packages/my-neurons
```

This creates:
- `Axon.toml` with package metadata
- `README.md` with basic documentation
- `.gitignore` with common ignore patterns
- `src/` directory with example neuron
- `examples/` directory (if `--bin` flag used)

### Validation

The manifest parser validates:
- **Package name**: lowercase, alphanumeric + hyphens, no leading/trailing/consecutive hyphens
- **Version**: Must be valid semver (e.g., "0.1.0", "1.2.3-alpha")
- **Dependencies**: Version requirements must be valid semver expressions
- **Consistency**: Can't specify both git and path for same dependency

Examples:
```toml
# Valid package names
name = "core-primitives"
name = "attention123"
name = "my-cool-neurons"

# Invalid package names
name = "Invalid_Name"        # No underscores
name = "-starts-with-hyphen" # No leading hyphen
name = "double--hyphen"      # No consecutive hyphens
```

## Future Phases

### Phase 2: Registry Infrastructure ✅ (Complete)

Implemented:
- ✅ Cache mechanism (~/.neuroscript/)
- ✅ `neuroscript add` command
- ✅ `neuroscript fetch` command
- ✅ Lockfile generation (Axon.lock)
- ✅ Dependency resolution framework with semver
- ✅ Git dependency fetching (URL, branch, tag, rev)
- ✅ Path dependency resolution

Not yet implemented (deferred to Phase 3):
- ⏳ Central registry index (git-based like crates.io-index)
- ⏳ Registry package fetching (currently skipped)
- ⏳ Full semver resolution with version selection

### Phase 3: Publishing & Security (Planned)

- Code signing with Ed25519
- Checksum verification (SHA-256)
- `neuroscript publish`, `neuroscript verify` commands
- Central registry server (Rust web service)
- Package verification on download

### Phase 4: Advanced Features (Planned)

- `neuroscript audit` (vulnerability database)
- Private registries
- Workspace support (multiple packages)
- Build caching
- Pre-compiled neuron caching

## Design Decisions

### Why TOML over YAML?

- Cargo uses TOML - consistency for Rust developers
- TOML is unambiguous (YAML has footguns like Norway problem)
- Better error messages
- Excellent Rust support (`toml` + `serde`)

### Why "Axon" for the manifest?

- Neural network terminology (axons connect neurons)
- Represents a package/bundle of neurons
- Domain-specific branding distinct from generic terms

### Why model after Cargo?

- Proven at scale (crates.io has 100k+ packages)
- Excellent UX that developers love
- Security built in (checksums, signatures)
- Git-based index is efficient and auditable
- Semver resolution is well-understood

Key differences from npm:
- Central registry not required (git deps work)
- Lockfiles are committed (reproducibility)
- No dependency hell (flat dependency tree)
- Security by default (verify checksums)

## Examples

### Creating a Package

```bash
# Initialize package
$ neuroscript init attention-mechanisms --author "You <you@example.com>"
✓ Created new package 'attention-mechanisms' at attention-mechanisms

Next steps:
  cd attention-mechanisms
  # Edit src/*.ns with your neuron definitions
  neuroscript build
```

### Package Manifest

```toml
[package]
name = "attention-mechanisms"
version = "0.1.0"
authors = ["You <you@example.com>"]
license = "MIT"
description = "A NeuroScript package"

neurons = []

[dependencies]

[python-runtime]
requires = ["torch>=2.0"]
```

### Generated Neuron

```neuroscript
// Example neuron definition
// Replace this with your actual neuron implementation

neuron AttentionMechanisms:
  in input: [*shape]
  out output: [*shape]
  graph:
    input -> Identity() -> output
```

### Adding Dependencies (Phase 2)

```bash
# Add a version dependency (for future registry)
$ neuroscript add core-primitives --version "^1.0"
✓ Added dependency: core-primitives
  Updated: ./Axon.toml

# Add a git dependency
$ neuroscript add attention-blocks \
    --git "https://github.com/user/attention.git" \
    --branch main
✓ Added dependency: attention-blocks
  Updated: ./Axon.toml

# Add a local path dependency (for development)
$ neuroscript add local-dev --path ../local-dev
✓ Added dependency: local-dev
  Updated: ./Axon.toml

Run `neuroscript fetch` to download dependencies
```

### Fetching Dependencies (Phase 2)

```bash
# Fetch all dependencies
$ neuroscript fetch
Fetching 3 dependencies...
  ✓ attention-blocks -> /Users/you/.neuroscript/git/a1b2c3
  ✓ local-dev -> /Users/you/projects/local-dev
Skipping registry dependency: core-primitives (not yet implemented)

✓ All dependencies fetched successfully

# Fetch with verbose output
$ neuroscript fetch --verbose
Loading manifest from ./Axon.toml
Package: my-package v0.1.0
Cache directory: /Users/you/.neuroscript
Fetching 2 dependencies...
  ✓ attention-blocks -> /Users/you/.neuroscript/git/a1b2c3
  ✓ local-dev -> /Users/you/projects/local-dev

✓ Generated Axon.lock

✓ All dependencies fetched successfully

# Update dependencies to latest compatible versions
$ neuroscript fetch --update
```

### Lockfile (Phase 2)

After running `neuroscript fetch`, an `Axon.lock` file is generated:

```toml
# This file is @generated by neuroscript
# It is not intended for manual editing

version = 1

[[package]]
name = "attention-blocks"
version = "0.3.1"
source = "git+https://github.com/user/attention.git?rev=abc123"

[[package]]
name = "local-dev"
version = "0.1.0"
source = "path+/Users/you/projects/local-dev"
```

This lockfile pins exact versions and commit hashes for reproducible builds.

## Testing

Run package management tests:
```bash
cargo test package::manifest
cargo test package::init
```

Test the CLI:
```bash
# Build
cargo build --release

# Test init command
./target/release/neuroscript init test-package --author "Test <test@example.com>"

# Verify structure
ls test-package/
cat test-package/Axon.toml
```

## Dependencies Added

### Phase 1
```toml
[dependencies]
toml = "0.8"           # TOML parsing for Axon.toml
semver = "1.0"         # Semantic versioning
# (serde already present)
```

### Phase 2
```toml
[dependencies]
git2 = "0.18"          # Git operations for dependency fetching
sha2 = "0.10"          # SHA-256 checksums
dirs = "5.0"           # Cross-platform directory paths

[dev-dependencies]
tempfile = "3.8"       # Temporary directories for testing
```

## Security Considerations

### Phase 1 & 2
- Package name validation prevents path traversal
- Version validation ensures valid semver
- Git URL validation
- SHA-256 infrastructure ready for checksums
- Isolated cache directory (~/.neuroscript/)

### Future (Phase 3)
- Ed25519 code signing
- SHA-256 checksums for all files
- Signature verification on download
- Sandboxed compilation
- Audit database for known vulnerabilities

## Contributing

When adding package management features:
1. Update this document
2. Add tests for new functionality
3. Update CLI help text
4. Consider security implications
5. Follow Cargo conventions when applicable

## References

- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Semantic Versioning](https://semver.org/)
- [TOML Specification](https://toml.io/)
