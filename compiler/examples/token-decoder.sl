module| Token_Decoder
    // A runnable simulation of the document's token-selection safety step.
    // stdin: 0.60 escalates; change it to 0.90 for a successful decode.
    fun| float confidence string selected > Decode_Token > string
        infer| confidence ~ 0.85
            selected > |back|
        fall|
            "HALT_AND_ESCALATE\n" > out
            "TEMPORAL_LOCK" ^> selected
            // This return is never reached after escalation.
            selected > |back|
        |infer
    |fun

    fun| > main > int
        float token_probs|1, 3|
        0.10 > token_probs|0|
        in > token_probs|1|
        0.05 > token_probs|2|
        string vocab|3|
        "other" > vocab|0|
        "candidate" > vocab|1|
        "last" > vocab|2|

        float max_conf(0.0)
        int best_id(0)
        iterate| int i(0)++ < token_probs.length
            when| token_probs|i| > max_conf
                token_probs|i| > max_conf
                i > best_id
            |when
        |iterate

        string selected("") next_token("")
        vocab|best_id| > selected
        max_conf selected > Decode_Token > next_token
        "Output: " + next_token + "\n" > out
        0 > |back|
    |fun
|module
