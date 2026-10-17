module| ClairII
    // Small executable mechanics example, not a trained AI model.
    float static_weights|2, 2, 2|
    float liquid_memory|2, 2, 2|
    int updates(0)

    fun| float current_state|1, 2| float new_perception|1, 2| > Update_Liquid_Memory > float|2, 2, 2|
        float memory_delta|2, 2, 2|
        current_state.cross(new_perception) * 0.01 > memory_delta
        liquid_memory + memory_delta > liquid_memory
        updates + 1 > updates
        liquid_memory > |back|
    |fun

    fun| float latent_input|1, 2| > Generate_Thought_Vector > float|1, 2|
        float system1_heuristic|1, 2| system2_analytical|1, 2| synthesized_thought|1, 2|
        latent_input * static_weights > system1_heuristic
        latent_input * liquid_memory > system2_analytical
        system1_heuristic + system2_analytical > synthesized_thought
        synthesized_thought > |back|
    |fun

    // Explicit example policy: accept only nonnegative first-coordinate
    // thoughts with a zero second coordinate. No universal truth verification.
    fun| float thought|1, 2| > Tensor.verify_axioms > float
        when| thought|0| >= 0.0 && thought|1| == 0.0
            1.0 > |back|
        other|
            0.0 > |back|
        |other
    |fun

    // Explicit example score, not a calibrated model confidence:
    // largest component divided by total nonnegative activation.
    fun| float thought|1, 2| > Tensor.confidence > float
        when| thought.sum > 0.0
            thought.max / thought.sum > |back|
        other|
            0.0 > |back|
        |other
    |fun

    // Deliberately tiny vocabulary encoder: "veto" exercises the axiom rule,
    // "uncertain" exercises the confidence rule, any other word is accepted.
    fun| string prompt|| > Tokenizer.encode > float|1, 2|
        float embeds|1, 2|
        1.0 > embeds|0|
        when| prompt|0| == "veto"
            0.0 > embeds|0|
            2.0 > embeds|1|
        |when
        when| prompt|0| == "uncertain"
            1.0 > embeds|1|
        |when
        embeds > |back|
    |fun

    fun| float thought|1, 2| > Tokenizer.decode_step > string
        when| updates == 0
            "accepted " > |back|
        other|
            "<EOT>" > |back|
        |other
    |fun

    fun| float thought_vector|1, 2| float confidence > EDL_Arbiter > bool
        float logical_coherence(0.0)
        thought_vector.verify_axioms > logical_coherence
        infer| confidence ~ 0.90
            infer| logical_coherence ~ 0.99
                true > |back|
            fall|
                "VETO: example axiom violated\n" > out
                false > |back|
            |infer
        fall|
            "HALT_AND_ESCALATE: confidence below threshold\n" > out
            false > |back|
        |infer
    |fun

    fun| string prompt|| > Execute_ClairII > string
        float token_embeds|1, 2| thought_vector|1, 2|
        string generated_output("")
        prompt > Tokenizer.encode > token_embeds
        iterate| int cycle(0)++ < 8
            token_embeds > Generate_Thought_Vector > thought_vector
            thought_vector thought_vector.confidence > EDL_Arbiter > bool is_valid
            when| is_valid
                thought_vector > Tokenizer.decode_step > string new_token
                generated_output.append(new_token)
                thought_vector token_embeds > Update_Liquid_Memory > liquid_memory
                when| new_token == "<EOT>"
                    break
                |when
            other|
                "TEMPORAL_LOCK" ^> generated_output
                break
            |other
        |iterate
        generated_output > |back|
    |fun

    fun| > main > int
        // Identity static weights; liquid memory starts at zero each run.
        1.0 > static_weights|0, 0|
        1.0 > static_weights|1, 1|
        string prompt|1|
        in > prompt|0|
        prompt > Execute_ClairII > string result
        result "\nUpdates: " updates "\nLiquid memory[0,0]: " liquid_memory|0, 0| "\n" > out
        0 > |back|
    |fun
|module
