# Omitl developer task runner
# Usage: just <recipe>

# Show available recipes
default:
    @just --list

# Compile and run in development mode (hot workflow)
run *ARGS:
    cargo run -- {{ARGS}}

# Compile release binary
build:
    cargo build --release

# Type-check without compiling a binary (fast)
check:
    cargo check

# Run the test suite
test:
    cargo test

# Run tests with output visible
test-verbose:
    cargo test -- --nocapture

# Format source code
fmt:
    cargo fmt

# Lint with clippy
lint:
    cargo clippy -- -D warnings

# Generate docs from a sample contract (requires a built binary)
example:
    cargo run -- generate \
        --input examples/api_contract.json \
        --brand examples/brand.json \
        --format pdf \
        --output /tmp/omitl_example.pdf
    @echo "Output: /tmp/omitl_example.pdf"

# Clean build artifacts
clean:
    cargo clean

# Full pre-commit pipeline: fmt + lint + test
ci: fmt lint test
    @echo "All checks passed"
