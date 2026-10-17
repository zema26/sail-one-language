---
name: Neuro-symbolic interpretation
description: Rationale for the confidence threshold and controlled escalation contracts in the user's branching extension.
---

Keep the existing interpretation of `~` as an inclusive confidence threshold, not approximate equality. Escalation captures the left-hand state string and right-hand payload string, stops the entire current run, and exposes a supervisor handoff; it is not a payload assignment or automatic network delivery.

**Why:** The supplied proposal describes confidence thresholds but specifies neither an approximation tolerance nor an arbiter destination or transport. These choices were made explicit rather than inventing a tolerance, silently contacting an external party, or pretending that the illustrative decoder is real neural inference.

**How to apply:** Preserve this contract for existing programs. Any approximate-proximity mode needs an explicit tolerance contract; actual arbiter delivery needs a configured destination. The user's requested branching extension is not, by itself, authorization to add the decoder's full tensor algebra, expand runtime memory limits, or implement model inference.

ClairII's axiom checking, confidence and tokenizer operations must have explicit application definitions, not built-in approval or fabricated model scores.

**Why:** The supplied architecture names these operations without supplying axioms, a confidence calculation, vocabulary or trained weights. Its claims about truth and ethics cannot be established by a compiler primitive.

**How to apply:** Let Sail application functions define those policies and algorithms, reject undefined hooks, and label small numerical examples as demonstrations rather than trained AI. Escalation still does not imply persisted snapshot/resume or supervisor context injection.
