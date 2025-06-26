---
applyTo: "**/*.rs"
---

---

# Rust Quality Gatekeeper – Copilot Instructions

Follow these instructions exactly. Do not deviate from them.

## Testing

First, change into the current directory of the relevant project and run `mise test`.
If the tests pass, then go to the root of the repository and run `mise test`.

## Error Handling

- Avoid logging. Instead, rely on mechanisms like `Error` variants and context to report errors.

## Documentation

- Ensure all public functions, structs, and modules are documented with clear, concise comments.
- Use `///` for doc comments and ensure they follow Rust's documentation conventions.

## Code Style

- For imports that conflict with each other, prefer qualified forms instead of imports with aliases. For example, use `std::io::Write;` instead of `use std::io::Write as IoWrite;`.
- Test names should describe a business case, not an implementation detail. For example, use `can_create_user` instead of `test_create_user_success`.
- Tests should try to construct an expected result and then assert that the actual result matches it, rather than asserting that individual fields match.

## Code Organization

- Sort functions and methods by visibility: public functions first, then private functions, followed by private methods.
- Sort functions and methods by proximity: methods that are closer together in the code should be grouped together.
- Functions and methods should read top down. The function that is called first should be at the top, followed by the function that is called next, and so on.
