module| Tensor_Decoder
    // A small, executable version of the proposed neural pipeline.
    fun| float ctx|1, 2| float weights|2, 2, 3| > Compute_Logits > float|1, 3|
        float logits|1, 3|
        ctx * weights > logits
        logits > |back|
    |fun

    fun| float logits|1, 3| > Softmax > float|1, 3|
        float shifted|1, 3| probs|1, 3|
        // Subtracting the maximum avoids exponential overflow.
        logits - logits.max > shifted
        shifted.exp > probs
        probs / probs.sum > probs
        probs > |back|
    |fun

    fun| float probs|1, 3| > Decode_Token > string
        string vocab|3|
        "other" > vocab|0|
        "candidate" > vocab|1|
        "last" > vocab|2|
        int best_id(0)
        probs.argmax > best_id
        string selected("")
        vocab|best_id| > selected
        infer| probs.max ~ 0.85
            selected > |back|
        fall|
            "TEMPORAL_LOCK" ^> selected
        |infer
    |fun

    fun| > main > int
        float ctx|1, 2| weights|2, 2, 3| logits|1, 3|
        0.0 > ctx|*|
        0.0 > weights|*|
        // stdin 1 accepts the candidate; stdin 0 escalates.
        in > ctx|0|
        4.0 > weights|0, 1|
        ctx weights > Compute_Logits > Softmax > logits
        string next_token("")
        logits > Decode_Token > next_token
        "Output: " + next_token + "\n" > out
        0 > |back|
    |fun
|module
