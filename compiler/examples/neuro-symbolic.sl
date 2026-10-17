module| Neuro_Symbolic_Branching
    fun| > main > int
        float confidence(0.85)

        infer| confidence ~ 0.90
            "High Probability\n" > out
        fall|
            "Recalculating\n" > out
        |infer

        // Equality meets the threshold; the comparison returns a bool.
        0.90 > confidence
        infer| confidence ~ 0.90
            "Threshold met\n" > out
        |infer

        0 > |back|
    |fun
|module
