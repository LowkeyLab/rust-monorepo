---
applyTo: "**/*.rs"
---
---
# Rust Quality Gatekeeper – Copilot Instructions

Before returning any code:

## Build check
Run `cargo check --all-targets --all-features`.
*   If it fails, fix the code until the command succeeds.

## Lint
Run `cargo clippy --all-targets --all-features -- -D warnings`.
*   Resolve every warning; treat warnings as errors.
*   Prefer idiomatic, safe Rust and avoid `unsafe` unless absolutely required; document any remaining `unsafe` blocks.

## Test
Run `cargo test --all-features --all-targets`.
*   All unit, integration, and doc tests must pass.

## Error Handling
*   Avoid logging. Instead, rely on mechanisms like `Error` variants and context to report errors.

## Documentation
*   Ensure all public functions, structs, and modules are documented with clear, concise comments.
*   Use `///` for doc comments and ensure they follow Rust's documentation conventions.

## Code Style
*   Run `cargo fmt` to format the code consistently.
*   For imports that conflict with each other, prefer qualified forms instead of imports with aliases. For example, use `std::io::Write;` instead of `use std::io::Write as IoWrite;`.

## Code Organization
*   Sort functions and methods by visibility: public functions first, then private functions, followed by private methods.
*   Sort functions and methods by proximity: methods that are closer together in the code should be grouped together.
*   Sort functions and methods by usage: function A that calls function B should be placed before function B.

Only return code that passes all checks above. If you cannot produce code that meets these requirements, return an error message indicating the issue.
