---
applyTo: "**/*.rs"
---
---
# Rust Quality Gatekeeper – Copilot Instructions

Follow these instructions exactly. Do not deviate from them.

## Commands

Run the following commands in the terminal to ensure the code meets the quality standards:
1. `cargo make clippy`
2. `cargo make`

## Error Handling
*   Avoid logging. Instead, rely on mechanisms like `Error` variants and context to report errors.

## Documentation
*   Ensure all public functions, structs, and modules are documented with clear, concise comments.
*   Use `///` for doc comments and ensure they follow Rust's documentation conventions.

## Code Style
*   For imports that conflict with each other, prefer qualified forms instead of imports with aliases. For example, use `std::io::Write;` instead of `use std::io::Write as IoWrite;`.
*   Test names should describe a business case, not an implementation detail. For example, use `can_create_user` instead of `test_create_user_success`.
*   Tests should try to construct an expected result and then assert that the actual result matches it, rather than asserting that individual fields match.

## Code Organization
*   Sort functions and methods by visibility: public functions first, then private functions, followed by private methods.
*   Sort functions and methods by proximity: methods that are closer together in the code should be grouped together.
*   Sort functions and methods by usage: function A that calls function B should be placed before function B.

Only return code that passes all checks above. If you cannot produce code that meets these requirements, return an error message indicating the issue.

Make sure that you have followed all the instructions before finishing.
