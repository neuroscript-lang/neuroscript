# Security Guide

## Overview

NeuroScript packages use Ed25519 digital signatures and SHA-256 checksums for integrity verification. Packages are distributed via git repos; checksums are verified automatically on fetch.

## Key Management

### Generate Keys

```bash
neuroscript keygen my-package
```

Creates:
- `~/.neuroscript/keys/my-package.key` — private signing key (0600 permissions)
- `~/.neuroscript/keys/my-package.pub` — public verification key

### Key Format

- **Private key**: 32-byte Ed25519 seed stored as raw hex
- **Public key**: 32-byte Ed25519 public key stored as raw hex
- **In Axon.toml**: `ED25519:<hex>` prefix format

### Key Discovery

When running `neuroscript publish`, the tool looks for keys in this order:
1. Explicit `--key <path>` flag
2. `~/.neuroscript/keys/<package-name>.key`
3. Any `.key` file in `~/.neuroscript/keys/`

## Signing Workflow

### 1. Generate keypair (once per publisher)
```bash
neuroscript keygen my-package
```

### 2. Develop your neurons
Edit `.ns` files in `src/`.

### 3. Publish (sign + checksum)
```bash
neuroscript publish
```

This:
1. Scans all `.ns` files in the package
2. Computes SHA-256 hash for each file
3. Computes overall checksum from sorted file hashes (deterministic via BTreeMap)
4. Signs the overall checksum with Ed25519
5. Writes `[security]` section to Axon.toml

### 4. Commit and push
The updated Axon.toml (with security section) is committed to git.

## Verification

### Manual verification
```bash
neuroscript verify
```

Checks:
- Each `.ns` file's SHA-256 matches its recorded checksum
- Overall checksum matches the deterministic hash of all file checksums
- Signature (if present) is valid against the publisher key

### Verification on fetch
When `neuroscript fetch` downloads a git dependency that has a `[security]` section, it automatically verifies:
- **Checksum mismatch**: Hard error — fetch fails
- **Signature failure**: Warning — fetch continues but alerts user

### Verification Report

The verification report tracks:
- `checksums_valid`: all per-file checksums match
- `overall_checksum_valid`: aggregate checksum matches
- `signature_valid`: signature verification result (None if no signature)
- `failed_files`: files whose checksums don't match
- `extra_files`: `.ns` files not in the checksum list
- `missing_files`: checksum entries with no corresponding file

## Security Formats

| Item | Format | Size |
|------|--------|------|
| Publisher key | `ED25519:<64 hex chars>` | 32 bytes |
| Signature | `ED25519:<128 hex chars>` | 64 bytes |
| Checksum | `sha256:<64 hex chars>` | 32 bytes |
| Key file | raw hex | 32 bytes |

## Best Practices

1. **Never share private keys** — keep `.key` files secure
2. **Commit Axon.lock** — ensures reproducible builds
3. **Re-publish after changes** — checksums must match current files
4. **Verify before deploying** — run `neuroscript verify` in CI
5. **Pin git dependencies** — use `tag` or `rev` instead of `branch` for production
