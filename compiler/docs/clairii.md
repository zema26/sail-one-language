# ClairII language support

The ClairII proposal uses several language constructs beyond the earlier tensor-math extension. Those constructs are now implemented in the Rust/LLVM compiler for WebAssembly and native Linux. This is language/runtime support, not an implementation or validation of the proposal's claims about intelligence, ethics, truth, or hallucination elimination.

## Shared module data

Scalar, vector and tensor declarations may appear outside functions:

```Sail
module| Memory
    int width(2)
    float static_weights|2, width, width|
    float liquid_memory|2, width, width|

    fun| float state|1, 2| float perception|1, 2| > Update > float|2, 2, 2|
        state.cross(perception) * 0.01 + liquid_memory > liquid_memory
        liquid_memory > |back|
    |fun
|module
```

All functions in a compiled program see the same module slots. Indexed writes mutate the tensor; whole-tensor assignment rebinds the shared slot after checking element type, rank and extents. Old aliases still refer to the old allocation. Parameters and local declarations may shadow module variables without replacing the shared slot.

Initialization runs once before the body of `main`, in declaration order, and starts fresh for every execution. Initializer and dimension expressions may use literals, earlier module variables and pure arithmetic. Forward references, calls and side effects in module initializers are rejected. CLI-linked dependencies initialize before their importer; browser source remains self-contained. Module names are organizational wrappers, not isolated data namespaces: duplicate module data names are rejected.

## Outer product

`a.cross(b)` is the **outer product specified in ClairII**, not the 3D geometric cross product. Numeric rank-one tensors of lengths M and N produce an independent rank-two M×N tensor with `result|i, j| = a|i| * b|j|`. Int/float mixtures promote to float; integer multiplication wraps in 64 bits. Empty inputs produce an empty matrix. Allocation limits apply before writing results.

```Sail
float state|1, 2| perception|1, 3| delta|2, 2, 3|
state.cross(perception) * 0.01 > delta
```

Existing `*` tensor multiplication retains its vector/matrix linear algebra meaning. Global memory updates use ordinary tensor addition and assignment.

## Calls, namespaces, and explicit tensor policies

Both `a b > Function > result` and `Function(a, b) > result` are supported. Parenthesized arguments are comma-separated. Calls can appear inside expressions.

Functions can have qualified names, such as `Tokenizer.encode` or `Tokenizer.decode_step`, and use the normal typed calling convention. A qualified function is a Sail definition, **not an automatic connection to a tokenizer library**.

The proposal supplies no axioms, confidence definition, vocabulary, model weights, or tokenization algorithm. Sail therefore does not fabricate those operations. Define them explicitly:

```Sail
// Example application policy: components are nonnegative and sum to one.
// This checks only this numerical rule, not truth or ethics.
fun| float thought|1, 2| > Tensor.verify_axioms > float
    when| thought|0| >= 0.0 && thought|1| >= 0.0 && thought.sum == 1.0
        1.0 > |back|
    other|
        0.0 > |back|
    |other
|fun
```

`thought.verify_axioms` calls `Tensor.verify_axioms(thought)`; `thought.confidence` similarly calls `Tensor.confidence(thought)`. Other application-defined tensor properties can use this convention too. Parenthesized methods pass the receiver first and any explicit arguments afterward. Ordinary type and shape contracts still apply. The existing built-in tensor math properties retain their meaning. Undefined policies produce an explicit diagnostic, never a default approval or confidence score.

The original `EDL_Arbiter` parameter `float thought_vector` is a scalar declaration. It must be corrected to a shaped tensor parameter, such as `float thought_vector|1, 2|` in the small example.

## Typed pipeline destinations and append

```Sail
thought_vector thought_vector.confidence > EDL_Arbiter > bool is_valid
thought_vector > Tokenizer.decode_step > string new_token
generated_output.append(new_token)
```

A typed destination creates a new variable in the current scope, with compile-time type checking. Shaped inline tensor destinations check their runtime dimensions. Redeclaring a local variable in the same scope is an error.

`string.append(string)` concatenates and rebinds its writable receiver, returning the new string. It accepts a variable, a string-valued vector element, or a string field. Existing string aliases are unchanged: string storage remains immutable. There is no in-place mutation of literal strings.

The ClairII `when| ... other| ... |other` ending is accepted, as are the existing `|when` and `|other` followed by `|when` endings. Adjacent `|other` / `|when` is interpreted as the legacy double ending; for nested blocks, include an explicit inner `|when` when necessary to avoid ambiguous ownership of the following close tag. Indentation still has no meaning.

## Run the ClairII example

Choose **ClairII: liquid memory & explicit policies** from Examples, or **Load ClairII example**.

- stdin **`hello`**: two verified steps generate `accepted <EOT>`, perform two liquid-memory updates, and show `Liquid memory[0,0]: 0.0201`.
- stdin **`veto`**: the explicitly defined example axiom rejects the thought before decoding; execution escalates with state `TEMPORAL_LOCK`.
- stdin **`uncertain`**: the example confidence score is below 0.90; execution escalates before decoding.

This example uses two-dimensional activations, a hand-written vocabulary encoder, and an explicitly documented numerical policy. It is not a trained model or a general-purpose tokenizer. Its score is not calibrated confidence. The example cannot establish universal logical coherence or ethical alignment.

The exact uploaded architecture is still incomplete: it has no `main` entry point or definitions for its AI operations, and its Arbiter declares the thought as a scalar. Its 8192×8192 tensors each require 67,108,864 cells, exceeding the unchanged **500,000-cell / 8 MiB allocation limits** before any reasoning can run. Intermediate tensors and appended strings consume the same per-run region; long cycles may exhaust it.

`^>` still terminates execution with a structured state/payload handoff. `TEMPORAL_LOCK` is a reported state label, not a persisted process snapshot. Automatic supervisor contact, context injection, resumption, trained weights and model loading remain outside this compiler.
