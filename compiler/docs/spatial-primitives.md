# Multidimensional spatial primitives

The attached extension adds ranked shapes and scalar broadcast assignment to Sail's existing vector syntax.

```sail
int n(8)
int primes|n|                  // Original one-dimensional vector
float weights|2, 64, 64|       // Rank-2, 64 by 64 tensor
0.0 > weights|*|               // Fill every cell, evaluating 0.0 once
1.5 > weights|3, 7|            // Set row 3, column 7
weights|3, 7| > out            // Read one scalar
int volume|3, 2, 3, 4|         // Any positive literal rank is supported
7 > volume|*|
volume|1, 2, 3| > out
```

## Declarations and shapes

- `type name|rank, extent1, ..., extentN|` requires a positive integer literal rank and exactly that many extents.
- Extents can be runtime `int` expressions, including variables. They are evaluated once at declaration.
- `int name|n|` remains an ordinary vector, without changing its interpretation. `int name|1, n|` is an explicit rank-one tensor.
- Supported cell types are `int`, `float`, `bool`, `char` and `string`. Defaults are zero, false or an empty string.
- Tensor declarations work in local scopes and as class fields. Tensor-valued function parameter syntax is not defined by this extension.
- Zero extents produce an empty tensor. Negative extents, extents above 500,000, or more than 500,000 total cells trap before allocation. The existing 8 MiB aggregate allocation budget still applies.

## Coordinates and broadcast

The supplied description defines declarations and broadcast fills but not coordinate syntax. This implementation uses **zero-based, comma-separated coordinates** in the same brackets: `weights|row, column|`. Contiguous storage is row-major: the last coordinate varies fastest. Every coordinate must be an `int`; the coordinate count must match rank, and every coordinate is checked against its own extent.

`scalar > collection|*|` fills all cells without a source-level loop. It works for original vectors as well as tensors. Normal scalar assignment conversions apply (for example, `int` promotes to `float`); incompatible values are diagnosed. The source is evaluated once, including side effects. Chained pipelines retain that converted scalar value; `in > weights|*|` reads one scalar and fills all cells with it. Wildcards are assignment destinations only, not array-valued expressions or slice reads.

`.length` returns the **total number of scalar cells**, not the number of rows. Tensors have fixed shapes after allocation; assigning tensors of the same scalar type and rank shares the collection reference, as with existing vectors.

Use broadcast or individual indexed assignments to initialize tensors. A later tensor-math extension adds read-only leading-axis slice copies, matrix/vector products, numeric elementwise arithmetic, reductions, and shaped function signatures; see **Tensor signatures and math**. Tensor literals, slice assignment, and NumPy-style shape broadcasting remain unsupported. Whole-tensor reference assignment checks matching dimensions as well as rank and scalar type.

Choose **Matrices & tensors** in the playground's Examples menu for a complete runnable program. Both WebAssembly and the server compiler's native development target use the same bounds-checked runtime.
