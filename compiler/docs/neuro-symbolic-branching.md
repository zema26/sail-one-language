# Neuro-Symbolic Branching

Sail supports the proposal's `~` confidence operator, `infer| ... fall| ... |infer` blocks, and `^>` escalation operator on both WebAssembly and native Linux targets.

## Implemented semantics

- `score ~ threshold` returns a bool: **score is greater than or equal to threshold**. Equality passes; there is no implicit tolerance, random sampling, or approximate-equality epsilon.
- Both operands must be int or float scalars. Mixed int/float comparisons use the existing numeric promotion rules. NaN comparisons are false, just like an ordered floating-point comparison. Values are not clamped to the probability interval.
- A tensor element can be compared directly, for example `token_probs|1| ~ 0.85`. Whole tensors must be indexed or explicitly reduced first.
- `infer|` requires a bool condition. It executes exactly one branch. `fall|` is optional and uses the same closing `|infer`; nested infer blocks, boolean combinations, loops, and returns work normally. Branch-local variables remain scoped to their branch.
- `state ^> payload` accepts two strings, evaluates each once, captures them, and **immediately stops the entire current run**, including callers. Statements after escalation never run. It does not assign to or mutate the payload variable.
- The captured status is `HALT_AND_ESCALATE`. In the proposal, the state string is `TEMPORAL_LOCK` and the payload is the selected token. The lock describes a halted inference run, not an operating-system or multi-process lock.
- The browser displays a separate, non-crash handoff panel with state and payload and preserves output written before the halt. The host result contains `escalation: { status, state, payload }` and exit code 75.
- Native executables emit one JSON handoff record on stderr and exit normally with code 75; stdout remains available. A parent supervisor can inspect the record. No human arbiter or external AI service is automatically contacted.
- The proposal's `value > |back|` return shorthand is supported, including after a function pipeline. The original `back| value |back` syntax remains supported.

## Run the examples

In the playground's Examples menu, select **Neuro-Symbolic Branching** for the threshold example. The first comparison is `0.85 ~ 0.90`, so it prints `Recalculating`; the second demonstrates an inclusive threshold and prints `Threshold met`.

Select **Token decoder & escalation** for a runnable simulation of the proposal's safety step. Leave stdin at `0.60` to see the `HALT_AND_ESCALATE` result with state `TEMPORAL_LOCK` and payload `candidate`. Change stdin to `0.90` and run again to see `Output: candidate` with exit code 0.

The simulation uses a small probability tensor, an explicit argmax loop, string vocabulary, and a decoder function. It is not an LLM implementation.

## Scope of the supplied decoder

The complete supplied document is reproduced below, including its illustrative decoder. Tensor-valued function signatures, matrix multiplication, `.exp`, `.sum`, `.max`, and `.argmax` are now implemented; see **Tensor signatures and math** and the **Tensor math & neural pipeline** example.

**The exact original decoder remains illustrative, not a currently executable Sail program.** Its 4096 × 4096 weights exceed the current 500,000-cell and 8 MiB limits, and its context-vector-times-weight-row expression cannot produce the declared 100,000-element logits. Module-level data is now supported by the ClairII extension. The runnable decoder uses dimensionally consistent small tensors and function-local data.

These limitations do not affect the new branching, fuzzy scalar threshold, or escalation features. Use the two runnable examples above to exercise those features today.

## Original document

Neuro-Symbolic Branching
Standard symbolic logic (when| / other|) requires absolute booleans.
An AI-optimized language benefits from heuristic evaluation operators that handle floating-point probabilities natively, allowing the language to compile generative behaviors.
 * Syntax: infer| / fall|
 * Fuzzy Operator (~): Evaluates proximity or confidence thresholds.
float confidence(0.85)

```Sail
infer| confidence ~ 0.90
    // Executes if the confidence tensor meets the acceptable threshold
    "High Probability" > out
fall|
    // Acts as a fallback for sub-threshold evaluations
    "Recalculating" > out
|infer
```

Here is a 50-line implementation of an LLM token decoding pipeline in Sail, utilizing `infer|` and `fall|` to apply neuro-symbolic branching based on the confidence tensor of the generated token.

Notes:
Escalation Operator (^>): Triggers a HALT_AND_ESCALATE state. If a task fails or yields a veto, the pipeline routes the global state output upward to a human arbiter or superior cognitive tier rather than crashing.

```Sail
module| LLM_Decoder

    // Tensors for neural weights and token vocabulary
    float model_weights|2, 4096, 4096|
    string vocab|100000|

    // Simulated neural feedforward calculation
    fun| float ctx|1, 4096| >  Compute_Logits > float|1, 100000|
        float logits|1, 100000|
        // Tensor matrix multiplication
        ctx * model_weights|0| > logits
        logits > |back|
    |fun

    // Simulated Softmax to extract probability distribution
    fun| float logits|1, 100000| >  Softmax > float|1, 100000|
        float probs|1, 100000|
        logits.exp / logits.exp.sum > probs
        probs > |back|
    |fun

    // Neuro-symbolic token selection with threshold branching
    fun| float context_vector|1, 4096| >  Decode_Token > string
        float token_probs|1, 100000|
        float max_conf(0.0)
        int best_id(0)
        string selected("")

        // Fluid left-to-right neural pipeline
        context_vector > Compute_Logits > Softmax > token_probs

        // Extract maximum probability (argmax)
        token_probs.max > max_conf
        token_probs.argmax > best_id
        vocab|best_id| > selected

        // Neuro-Symbolic Branching
        infer| max_conf ~ 0.85
            // High confidence threshold met: stream token directly
            selected > |back|

        fall|
            // Confidence failed threshold: trigger systemic safety
            // Invoke global TEMPORAL_LOCK to freeze inference state
            "HALT_AND_ESCALATE" > out
            "TEMPORAL_LOCK" ^> selected

            // Route to reasoning arbiter instead of hallucinating
            selected > |back|
        |infer
    |fun

    // Main execution entry point
    fun| string args|| >  main > int
        float current_ctx|1, 4096|
        string next_token("")

        // Broadcast assignment to initialize tensor
        0.5 > current_ctx|*|

        // Execute the decoder step
        current_ctx > Decode_Token > next_token

        "Output: " + next_token > out
        0 > |back|
    |fun

|module

```
