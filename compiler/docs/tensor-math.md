# Tensor signatures and math

Tensor-valued function parameters and return values, numeric tensor arithmetic, leading-axis slices, matrix/vector products, exponentials, and reductions are implemented on WebAssembly and native Linux.

## Shaped function signatures

```Sail
fun| float ctx|1, 2| float weights|2, 2, 3| > Compute_Logits > float|1, 3|
    float logits|1, 3|
    ctx * weights > logits
    logits > |back|
|fun
```

The syntax is `type name|rank, dimensions...|` for parameters and `type|rank, dimensions...|` for returns. Dimensions may be nonnegative integer literals, int scalar parameters, or pure integer arithmetic on those parameters:

```Sail
fun| int n float values|1, n| > Identity > float|1, n|
    values > |back|
|fun
```

Ranks must be literals from 1 to 128. Element type and rank mismatches are compile-time errors; dimensions are checked at function entry and return. Return dimensions are captured at entry, so modifying a scalar parameter inside the function does not change its return contract. Every path of a tensor-returning function must return a tensor or explicitly halt/escalate.

The original `string args||` / vector signatures still work. Tensors use region-owned storage and can safely outlive the function that created them, until the run ends. Tensor assignment shares a reference, rather than copying cells, and now checks that the destination's shape matches. Indexed mutation through a parameter is visible to the caller; rebinding a parameter itself is local.

## Numeric arithmetic and multiplication

- Tensor `+`, `-`, and `/` tensor are element-wise and require identical shapes. No NumPy-style shape broadcasting is implied.
- Tensor with scalar, or scalar with tensor, supports `+`, `-`, `*`, `/`, with scalar broadcasting and the written operand order preserved.
- Operands must have int or float elements. Mixed arithmetic produces a float tensor; int-only operations retain int64 semantics, including checked integer division and wrapping addition/multiplication. Function signatures and whole-tensor assignment still require identical element types.
- Tensor `*` tensor is a linear algebra product: vector · vector returns a scalar dot product; matrix × matrix returns a matrix; vector × matrix and matrix × vector return vectors. Inner dimensions must match. Batched products above rank two are not supported.
- Arithmetic and slices allocate new tensors and do not mutate inputs.

```Sail
float a|2, 2, 3| b|2, 3, 2| product|2, 2, 2|
1.0 > a|*|
2.0 > b|*|
a * b > product
product|0, 1| > out  // 6
```

## Slices and probability operations

`weights|0|` on a matrix returns a copy of its first row as a rank-one tensor. In general, supplying fewer leading coordinates than the rank returns a lower-rank copy; full coordinates return a scalar. Partial-index writes are not supported. Coordinates are zero-based and checked.

- `.exp` maps the exponential function over int or float cells and returns a same-shape float tensor. The browser uses the pure Math.exp host function; native Linux uses libm.
- `.sum` reduces all cells, returning the element type. An empty tensor sums to zero.
- `.max` returns the largest element; `.argmax` returns its zero-based row-major flat index as int. Ties choose the earliest cell. Empty tensors and NaN elements in max/argmax raise explicit runtime errors.
- `.length` remains the total cell count, not the row count. For a matrix argmax, derive row/column from the flat index and known column extent.

A stable softmax is expressed using the implemented primitives:

```Sail
logits - logits.max > shifted
shifted.exp > probs
probs / probs.sum > probs
```

## Runnable decoder and limits

Choose **Tensor math & neural pipeline** in the Examples menu. This small decoder now uses real matrix multiplication, shaped tensor functions, a stable softmax, and max/argmax token selection. stdin `1` produces `Output: candidate`; stdin `0` halts with `TEMPORAL_LOCK`.

The original proposal is preserved in Reference, but its exact large decoder is not a valid model: a 4096 × 4096 tensor exceeds the existing 500,000-cell and 8 MiB region limits, and its vector-times-row expression cannot produce the declared 100,000 logits. Module-level data is now supported by the ClairII extension. Use the dimensionally consistent runnable example rather than treating the proposal as a trained LLM.

Memory limits are unchanged. A single product is also limited to 50 million scalar multiply-accumulate steps to avoid unbounded inference work. Tensor results are region-owned temporaries, not reclaimed during the run; large or repeated intermediate allocations may reach the existing memory budget. Tensor literals, general slice assignment, batched matrix multiplication, automatic differentiation, and model loading are not implemented.
