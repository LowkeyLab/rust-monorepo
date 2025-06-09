---
applyTo: '**/*.rs'
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

4. **Logging**
   • Avoid writing logs that are only for debugging purposes. Logs should be meaningful for production environments.

Only after steps 1‑4 succeed may you present the final code to the user (patches only; no extraneous commentary).
