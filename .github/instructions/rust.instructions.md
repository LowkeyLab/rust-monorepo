---
applyTo: "**/*.rs"
---

# Rust Quality Gatekeeper – Copilot Instructions

Before returning any code:

1. **Build check**  
   Run `cargo check --all-targets --all-features`.  
   • If it fails, fix the code until the command succeeds.

2. **Lint**  
   Run `cargo clippy --all-targets --all-features -- -D warnings`.  
   • Resolve every warning; treat warnings as errors.  
   • Prefer idiomatic, safe Rust and avoid `unsafe` unless absolutely required; document any remaining `unsafe` blocks.

3. **Test**  
   Run `cargo test --all-features --all-targets`.  
   • All unit, integration, and doc tests must pass.

4. **Error Handling**
   • Avoid logging. Instead, rely on mechanisms like `Error` variants and context to report errors.

5. **Documentation**  
   • Ensure all public functions, structs, and modules are documented with clear, concise comments.  
   • Use `///` for doc comments and ensure they follow Rust's documentation conventions.

6. **Code Style**
   • Run `cargo fmt` to format the code consistently.
   • Sort functions and methods by visibility: public functions first, then private functions, followed by private methods.
   • Sort functions and methods by proximity: methods that are closer together in the code should be grouped together.

Only after steps 1‑4 succeed may you present the final code to the user (patches only; no extraneous commentary).
