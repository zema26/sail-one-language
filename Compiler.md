# Sail compiler — Rust + LLVM

A Rust lexer, parser, typed LLVM IR generator and command-line driver for the attached Sail language manual. The web playground compiles on the server and executes LLVM-generated WebAssembly in a browser worker.

**Status:** core-language release. See `docs/implementation.md` for exact support and remaining features. This is not a complete implementation of Multitex, tracing GC or arbitrary compile-time evaluation.

## Ubuntu 24.04 prerequisites

```sh
sudo apt update
sudo apt install -y build-essential clang-18 lld-18 curl
# Rust: install using the official installer, reviewing it first if required.
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh
sh /tmp/rustup-init.sh
. "$HOME/.cargo/env"

# Ensure the LLVM 18 driver and linker are discoverable as clang / wasm-ld.
mkdir -p "$HOME/.local/bin"
ln -sf /usr/bin/clang-18 "$HOME/.local/bin/clang"
ln -sf /usr/bin/wasm-ld-18 "$HOME/.local/bin/wasm-ld"
export PATH="$HOME/.local/bin:$PATH"
```

No Rust crate dependencies or LLVM Rust bindings are required. LLVM IR is emitted directly, then checked and compiled by Clang/LLVM 18+.

## Build and use

From the repository root:

```sh
cargo build --release --manifest-path compiler/Cargo.toml
cargo test --manifest-path compiler/Cargo.toml

# LLVM IR on stdout
compiler/target/release/sailc compiler/examples/euclidean.sl --emit-llvm

# Native Ubuntu executable; stdin uses whitespace-separated values
compiler/target/release/sailc compiler/examples/euclidean.sl --native -O2 -o /tmp/euclid
printf '48 18\n' | /tmp/euclid

# Browser WebAssembly; requires the Sail host imports supplied by the playground
compiler/target/release/sailc compiler/examples/primes.sl --wasm -O1 -o /tmp/primes.wasm

# Machine-readable diagnostics (external module links disabled)
compiler/target/release/sailc compiler/examples/hello.sl --wasm --json -o /tmp/hello.wasm
```

Both target modes preserve `OUTPUT.ll` so you can inspect the generated LLVM IR.

Linked modules use paths relative to the importing `.sl` file. Shared linked modules must not define duplicate functions. Native file I/O uses the process's working directory and should only be used with trusted programs.

## Webpage on Ubuntu

Install Node.js 24 and pnpm 10, plus the toolchain above, then:

```sh
pnpm install
pnpm --filter @workspace/api-spec run codegen

# Terminal 1 — Express compiler API
PORT=5000 pnpm --filter @workspace/api-server run dev

# Terminal 2 — the dark-themed webpage
PORT=5173 BASE_PATH=/ pnpm --filter @workspace/sail-playground run dev

# Terminal 3 — optional Vite dev proxy to serve the app + API on one Ubuntu URL
node scripts/ubuntu-web.mjs
```

Open `http://localhost:8080`. The Ubuntu helper proxies `/api` to port 5000 and everything else, including Vite HMR, to port 5173. It is for local development, not a hardened production reverse proxy.

For production use a reverse proxy (such as nginx) mapping `/api` to the API server and `/` to the built frontend. Keep LLVM and `compiler/` available to the API process. Never execute submitted native binaries on the web server.

## Structure

- `src/lexer.rs` — tokens with source locations.
- `src/parser.rs` — pipe tags, expressions, declarations, classes and functions.
- `src/ast.rs` — typed source model.
- `src/codegen.rs` — semantic checks, specialization and LLVM IR.
- `src/main.rs` — command-line driver and relative module resolver.
- `runtime/runtime.c` — small native/WebAssembly support runtime.
- `examples/` — executable Sail examples.
- `docs/` — original manual and implementation decisions.