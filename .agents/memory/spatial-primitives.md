---
name: Spatial primitive interpretation
description: Scope and rationale for interpreting the user's multidimensional-array extension.
---

The user's supplied "Multidimensional Spatial Primitives" extension explicitly defines ranked declarations and scalar wildcard fills. Preserve original one-dimensional vector syntax.

**Why:** The supplied extension does not define coordinate syntax, tensor algebra, slicing or NumPy-style shape broadcasting. These must not be implied by implementing its scalar fill operation.

**How to apply:** The chosen coordinate convention is zero-based, comma-separated indexing with row-major storage. Keep wildcard fills assignment-only and evaluate their scalar source once. Changes to this interpretation should be discussed rather than silently changing existing programs.
