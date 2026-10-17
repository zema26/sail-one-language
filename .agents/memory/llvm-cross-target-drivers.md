---
name: LLVM cross-target drivers
description: Why the Linux native compiler wrapper must not be used for wasm32 linking on Nix.
---

Use the native Clang wrapper for Linux binaries, and the underlying unwrapped LLVM driver for WebAssembly.

**Why:** Native Nix wrappers supply libc headers and linker setup needed by Linux, but also inject ELF dynamic-linker flags that wasm-ld rejects. An unwrapped driver alone compiles WebAssembly but cannot find native libc headers.

**How to apply:** Preserve target-dependent driver selection when changing the toolchain. Do not fix one target by globally replacing Clang for both targets. Native Nix builds must prefer a wrapper even when PATH puts the unwrapped driver first; package changes can reorder PATH. Ordinary Ubuntu Clang does not require this split.