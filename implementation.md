# Implementation notes

This is Sail compiler 0.1.0, a working implementation of the core language and the manual's Euclidean, sieve, GCD-class and numeric-generic examples. It is **not a complete implementation of every feature in the manual**.

## Compiler pipeline

Sail source → Rust lexer → Rust parser / AST → type checking and specialization → textual LLVM IR → Clang / LLVM → WebAssembly or Linux native executable.

There is no Sail-to-JavaScript interpreter or fabricated output. The browser executes the actual LLVM-produced WebAssembly.

## Implemented

- Pipe-delimited modules, functions, conditions, loops and returns.
- Left-to-right value assignment, function calls, chained input and output.
- Signed 64-bit integers, IEEE 64-bit floats, booleans, Unicode character values and UTF-8 strings.
- Whitespace-separated declarations, default initialization and initializer expressions.
- Dynamic-size typed vectors, vector literals, checked indexing, `.length`.
- Arithmetic, comparisons, unary minus, `not` / `!`, short-circuit `and` / `or` / `&&` / `||`.
- `repeat`, `iterate` with `++` / `--`, nested `when` / `other`, `break` / `continue`.
- Class fields and methods, object creation, member pipelines.
- `comp` definitions and numeric `&T` / `&Op` specialization. Both arguments determine the bound type; int/float mixtures promote to float.
- Text files: declaration, open/close, read/write and EOF.
- CLI intermodular links resolved relative to the importing file. The standard `in2out.sl` / `.sln` link is intrinsic.
- LLVM IR output, O0/O1/O2 compilation, WebAssembly and native executable output.

## Deliberate interpretations of the manual

- Newlines (or semicolons) end statements; indentation has no meaning.
- `>` is a comparison in conditions, bounds, returns and parenthesized expressions; outside those contexts it separates pipeline stages. Write `(a > b) > result` to store a comparison.
- Arguments to a function are separate expressions: `a b > Euclid`. Output concatenates values without inserting spaces or newlines.
- Double- and single-quoted literals are strings. A string assigned to `char` must contain exactly one Unicode character.
- Uninitialized scalars are zero/false/empty; vector elements are zeroed.
- `string args| |` is accepted for main. It currently receives an empty vector; command-line argument forwarding is not implemented.
- Functions that fall through return zero/false/empty (or no value for `void`).
- Integer arithmetic wraps in 64 bits; integer division by zero and minimum-int / -1 overflow trap. Float division follows IEEE rules.
- Strings are immutable. String indexing and `.length` operate on UTF-8 bytes. Character literals/input/output use Unicode code points.
- `iterate| i(0)++ < n` reuses an existing int variable. `iterate| int k(0)++ < n` creates one if absent, scoped to the loop.
- File access is whitespace-separated, for example `file f("Readme.md" txt wr+)`. The first open with `w` truncates; subsequent opens read existing contents. String reads return one line without its newline.
- Browser files are private, in-memory, and discarded after each run. They never access the server or the user's computer.
- The subtraction-based GCD examples require **positive** inputs; zero or negative inputs can cause an infinite loop. The runner stops these after three seconds.
- The generics example's `1.41 * 1.73` result is `2.4393`, not the rounded `2.44` shown in the manual.

## Not yet implemented

- Multitex task vectors and concurrency.
- A tracing garbage collector. This release uses a bounded, zero-initialized allocation region, reclaimed at the end of each program run.
- Arbitrary code execution inside `comp` blocks; only definitions and type/operator specialization are supported.
- Generic member functions, inheritance and object-valued fields.
- Binary/hex file encodings.
- Forward class declarations and arbitrary source-level overloaded operators.

Unsupported syntax is diagnosed. The manual describes some of these at a conceptual level without enough details for a compatible implementation; this release does not invent their semantics.

## Browser limits

- Source: 65,536 characters; compilation: 10 seconds; at most two concurrent compiler jobs.
- Execution: three seconds in a dedicated Web Worker, terminated on timeout.
- Allocation region: 8 MiB; linear memory maximum: 32 MiB.
- Vector length: 500,000 elements; indices are checked at runtime.
- Output/stdin: 64 KiB; virtual files: 32 handles and 64 KiB combined contents.
- Programs receive only numeric/string I/O and virtual-file imports. No network, shell, environment or server filesystem capabilities are passed to WebAssembly.
- Native binaries are **not sandboxed**. Run only trusted source natively.

## Ubuntu 24.04

See the project's compiler README for setup and command-line usage. The online host uses its own Linux package manager; the compiler source and native target are portable to Ubuntu.