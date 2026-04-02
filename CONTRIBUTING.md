# Contributing to Agnoshi

Thank you for your interest in contributing to Agnoshi!

## Getting Started

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `make check` to verify
5. Submit a pull request

## Development

```bash
make check    # fmt + clippy + test + audit
make bench    # Run benchmarks
make coverage # Generate coverage report
```

## Code Standards

- `cargo fmt` — no exceptions
- `cargo clippy -D warnings` — no warnings
- All public types must be `Serialize + Deserialize`
- `#[non_exhaustive]` on all public enums
- `#[must_use]` on all pure functions
- Zero `unwrap()`/`panic!()` in library code

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0-only.
