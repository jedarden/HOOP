# TestRepo

A test repository for HOOP integration testing.

## Overview

This is a synthetic Rust project designed to provide a realistic file tree for testing HOOP's integration test suite. It contains:

- ~500 source files representing a typical Rust workspace
- Multiple binary targets
- Comprehensive test suites
- Documentation
- Configuration examples

## Structure

```
testrepo/
├── src/           # Library source code
├── tests/         # Integration tests
├── benches/       # Criterion benchmarks
├── examples/      # Example usage
├── docs/          # Additional documentation
└── assets/        # Test attachments
```

## Usage

This repository is intended for testing purposes only.

```bash
cargo build
cargo test
cargo run --bin testrepo-cli
```

## License

MIT
