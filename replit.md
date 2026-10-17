# Sail Compiler

Rust/LLVM compiler for the uploaded Sail language manual, with a dedicated dark-themed editor and WebAssembly runner.

## Run & verify

- Managed workflow `artifacts/api-server: API Server` builds Rust and starts the Express API.
- Managed workflow `artifacts/sail-playground: web` starts the React/Vite webpage at `/`.
- `pnpm run typecheck` — workspace TypeScript checks.
- `cargo test --manifest-path compiler/Cargo.toml` — compiler tests.
- `cargo build --release --manifest-path compiler/Cargo.toml` — build sailc.
- `node compiler/tests/run-programs.mjs` — compile/execute all examples with LLVM WebAssembly and Linux native, plus runtime assertions.
- `node compiler/tests/run-tensors.mjs` — multidimensional arrays, broadcast fills, scalar types, dynamic shapes and runtime traps on both targets.
- `pnpm --filter @workspace/api-spec run codegen` — regenerate clients after API changes.
- `compiler/README.md` covers building the compiler from source and local webpage serving on Ubuntu 24.04.

## Stack and structure

- `compiler/` — Rust frontend, direct LLVM IR generation, Clang driver, C runtime, examples and documentation.
- `compiler/docs/language.md` — original uploaded manual.
- `compiler/docs/implementation.md` — support matrix and explicit language interpretations.
- `artifacts/api-server/src/routes/sail.ts` — compilation, examples, reference and status API.
- `artifacts/sail-playground/` — React dark-themed editor, diagnostics and LLVM IR.
- `artifacts/sail-playground/src/lib/wasm-runtime.ts` — restricted I/O and in-memory filesystem imports.
- `lib/api-spec/openapi.yaml` — API source of truth; Orval generates client hooks and Zod schemas.
- No database, user authentication, or third-party integrations are needed for this product.

## Safety and scope

- Never execute web-submitted native programs on the API server. Server compiles only; browser workers run WebAssembly with a three-second timeout.
- Native CLI is for trusted local source. It intentionally has native file I/O.
- Do not imply the whole manual is implemented: Multitex, tracing GC and arbitrary compile-time execution remain unsupported and are documented in the app.
- The `comp` implementation specializes definitions, not general compile-time statements.
- User requested Rust, LLVM, Ubuntu compatibility, and a dedicated dark-themed webpage.
- User requested continuation after disconnection and a credit-based cutoff. Agent cannot inspect credit balance or enforce a custom spending cap; account-level usage limits must be managed in Replit.
