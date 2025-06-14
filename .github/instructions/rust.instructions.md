---
applyTo: "**/*.rs"
---
---
# Rust Quality Gatekeeper – Copilot Instructions

Follow these instructions exactly. Do not deviate from them.

## Commands

Run the following commands in the terminal to ensure the code meets the quality standards:
```bash
cargo make clippy
cargo make dev-test-flow
```

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

Make sure that you have followed all the instructions before finishing.
