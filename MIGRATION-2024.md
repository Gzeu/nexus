# Rust 2024 Edition Migration Guide for NEXUS

**Migration Date:** November 7, 2025  
**Previous Edition:** 2021  
**New Edition:** 2024  
**Minimum Rust Version:** 1.85.0

## Overview

This document outlines the migration of NEXUS from Rust Edition 2021 to Edition 2024, which was released on February 20, 2025 with Rust 1.85.0.

## Changes Made

### Configuration Updates

1. **Cargo.toml** (Workspace)
   - `edition = "2024"` (updated from `"2021"`)
   - `rust-version = "1.85"` (updated from `"1.82"`)

2. **rustfmt.toml**
   - `edition = "2024"`
   - `style_edition = "2021"` (added to preserve existing formatting)

## Next Steps for Local Development

After pulling this branch, you'll need to:

### 1. Update Rust Toolchain

```bash
# Update to latest stable (must be 1.85.0 or newer)
rustup update stable
rustup default stable

# Verify version
rustc --version  # Should show 1.85.0 or higher
```

### 2. Run Automated Migration Fixes

```bash
# This will automatically fix edition-specific issues
cargo fix --edition --all-features --allow-dirty

# Review the changes made by cargo fix
git diff
```

### 3. Build and Test

```bash
# Clean build
cargo clean
cargo build --all-features

# Run tests
cargo test --all-features

# Run clippy
cargo clippy --all-features -- -D warnings

# Check formatting
cargo fmt --all -- --check
```

### 4. Address Manual Fixes (if needed)

Some changes may require manual intervention:

#### Temporary Lifetime Changes

```rust
// If you encounter lifetime issues, you may need to add explicit scopes
{
    if let Some(x) = get_option() {
        use_reference(&x);
    }
}
```

#### Pattern Matching

`cargo fix --edition` may aggressively convert `if let` to `match`. Review these:

```rust
// Original (might be preferred)
if let Some(value) = option {
    process(value);
}

// Auto-generated (review if this is better)
match option {
    Some(value) => process(value),
    None => {}
}
```

## New Features Available in Rust 2024

### 1. Async Closures

You can now use async closures directly:

```rust
// New in 2024!
let async_closure = async || {
    fetch_data().await
};

// Useful in agent system
agent.map(async |data| {
    process_async(data).await
})
```

### 2. RPIT Lifetime Capture

Improved `impl Trait` return position lifetime capture:

```rust
pub trait BlockchainClient {
    async fn get_balance(&self, address: Address) 
        -> impl Future<Output = Result<Balance>> + use<'_>;
}
```

### 3. Reserved Keywords

- `gen` - Reserved for future generators
- `async gen` - Reserved for async generators

Ensure you don't use these as identifiers.

### 4. Enhanced Unsafe Blocks

Unsafe extern blocks now require explicit `unsafe` keyword:

```rust
// 2024 edition requires
unsafe extern "C" {
    fn external_function();
}
```

## CI/CD Updates Required

Update GitHub Actions workflows to use Rust 1.85+:

```yaml
- name: Install Rust
  uses: actions-rs/toolchain@v1
  with:
    toolchain: 1.85.0  # or 'stable'
    profile: minimal
    override: true
```

## Formatting Style

We've chosen to preserve 2021 formatting style (`style_edition = "2021"`) during the initial migration. This prevents unnecessary whitespace-only commits.

### Future: Adopting 2024 Formatting

When ready to adopt 2024 formatting improvements:

1. Remove or update `style_edition` in `rustfmt.toml`
2. Run `cargo fmt --all`
3. Commit formatting changes separately

## Breaking Changes Summary

1. **Temporary Scopes**: Changes to `if let` and tail expression temporary scopes
2. **Pattern Matching**: More aggressive pattern matching suggestions
3. **Unsafe Blocks**: Stricter requirements for unsafe extern blocks
4. **Reserved Keywords**: Cannot use `gen` as identifier
5. **Lifetime Inference**: More precise RPIT lifetime capture

## Benefits for NEXUS

- **Async Closures**: Simplifies AI agent orchestration
- **Better Lifetime Inference**: Cleaner Web3 trait implementations
- **Enhanced Safety**: Improved unsafe block requirements
- **Future-Ready**: Prepared for upcoming async generators
- **Modern Patterns**: Latest Rust idioms and best practices

## Rollback Procedure

If issues arise:

```bash
# Switch back to main branch
git checkout main

# Or revert edition in Cargo.toml
edition = "2021"
rust-version = "1.82"
```

## Testing Checklist

- [ ] All crates build successfully
- [ ] All tests pass (`cargo test --all-features`)
- [ ] Clippy checks pass (`cargo clippy --all-features`)
- [ ] Documentation builds (`cargo doc --all-features --no-deps`)
- [ ] Formatting is consistent (`cargo fmt --all -- --check`)
- [ ] CI/CD pipeline passes
- [ ] Local development workflow works
- [ ] Plugin system functions correctly
- [ ] AI engine integration works
- [ ] Web3 operations succeed

## Resources

- [Rust 2024 Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/)
- [Rust 1.85.0 Announcement](https://blog.rust-lang.org/2025/02/19/Rust-1.85.0.html)
- [Edition Migration Guide](https://doc.rust-lang.org/edition-guide/editions/transitioning-an-existing-project-to-a-new-edition.html)
- [Cargo Fix Documentation](https://doc.rust-lang.org/cargo/commands/cargo-fix.html)

## Questions or Issues?

If you encounter problems during migration:

1. Check this guide for known issues
2. Review `cargo fix --edition` output carefully
3. Run `cargo check` to see detailed error messages
4. Open an issue on GitHub with:
   - Error message
   - Affected crate/file
   - Rust version (`rustc --version`)
   - Steps to reproduce

---

**Status:** ✅ Configuration files updated  
**Next:** Run `cargo fix --edition` locally and test thoroughly