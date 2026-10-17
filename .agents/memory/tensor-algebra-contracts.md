---
name: Tensor algebra contracts
description: Rationale for tensor multiplication, ownership, and signature dimension contracts.
---

For tensor operands, `*` means the rank-one/rank-two linear algebra product, not elementwise multiplication. Scalar multiplication still broadcasts.

**Why:** The supplied decoder explicitly labels its `*` expression as matrix multiplication. Choosing an elementwise interpretation would not provide the feedforward operation the user requested. The original expression itself has inconsistent output dimensions, so supporting the primitive does not make that exact example a valid model.

**How to apply:** Preserve the documented distinction when adding further operators or batching. Use dimensionally consistent small examples rather than silently rewriting the original proposal or claiming that a trained LLM is implemented.

ClairII's `.cross(...)` means an outer product of two rank-one tensors, not the usual geometric cross product.

**Why:** The ClairII proposal explicitly says the operation creates a state-by-perception memory matrix. A conventional three-component cross product would produce the wrong rank and learning update.

**How to apply:** Preserve the proposal-specific outer-product meaning; use a different, explicitly specified operation if geometric cross products are added.

Tensor function returns retain region-owned storage; slices and arithmetic create independent results. Dimension contracts use pure expressions of scalar parameters, with return extents captured at entry.

**Why:** This avoids dangling returned arrays, ambiguous slice mutation, side effects in annotations, and return contracts that change when a function mutates its parameters. Existing aggregate memory limits apply to all intermediates.

**How to apply:** Extend tensor functionality without silently introducing view aliasing, altering signature dimensions mid-call, or raising memory limits to accommodate the proposal's oversized weights.
