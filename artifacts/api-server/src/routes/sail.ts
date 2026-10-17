import { Router, type IRouter } from "express";
import { readFile } from "node:fs/promises";
import path from "node:path";
import {
  CompileSailBody, CompileSailResponse, GetCompilerStatusResponse,
  GetExamplesResponse, GetLanguageReferenceResponse,
} from "@workspace/api-zod";
import { compileProgram, compilerRoot, compilerVersion, toolchainStatus } from "../lib/sail-compiler";

const router: IRouter = Router();
const examples = [
  { id: "hello", title: "Hello, Sail", description: "Your first left-to-right output pipeline.", category: "Getting started", stdin: "" },
  { id: "euclidean", title: "Euclidean algorithm", description: "The specification's GCD example. Supply two positive integers.", category: "Functions & input", stdin: "48 18" },
  { id: "primes", title: "Sieve of Eratosthenes", description: "The specification's sieve with vectors, nested loops, and break.", category: "Control flow", stdin: "" },
  { id: "vectors", title: "Vectors & strings", description: "Vector initialization, indexed access, iteration, and string concatenation.", category: "Collections", stdin: "" },
  { id: "tensors", title: "Matrices & tensors", description: "Ranked shapes, checked multidimensional indexing, and scalar broadcast fills.", category: "Collections", stdin: "" },
  { id: "neuro-symbolic", title: "Neuro-Symbolic Branching", description: "infer/fall blocks with inclusive fuzzy confidence thresholds (~).", category: "Control flow", stdin: "" },
  { id: "token-decoder", title: "Token decoder & escalation", description: "Simulated token selection with ^> safety handoff. stdin 0.60 escalates; 0.90 succeeds.", category: "Control flow", stdin: "0.60" },
  { id: "tensor-math", title: "Tensor math & neural pipeline", description: "Shaped tensor functions, matrix multiplication, stable softmax, max/argmax, and escalation. stdin 1 succeeds; 0 escalates.", category: "Tensor math", stdin: "1" },
  { id: "clairii", title: "ClairII: liquid memory & explicit policies", description: "Module tensors, outer products, mutable liquid memory, explicit axiom/confidence functions and a tiny vocabulary encoder. stdin hello succeeds; veto or uncertain escalates. Not a trained AI.", category: "Tensor math", stdin: "hello" },
  { id: "classes", title: "Compile-time classes", description: "The specification's GCD class, data fields, and member-function pipeline.", category: "Object-oriented", stdin: "84 30" },
  { id: "generics", title: "Type & operator binding", description: "The specification's &T and &Op numeric specialization.", category: "Compile time", stdin: "" },
  { id: "files", title: "File round trip", description: "Write and read a virtual text file. Browser files exist only for one run.", category: "File I/O", stdin: "" },
];
router.get("/sail/status", async (_req, res): Promise<void> => {
  const toolchain = await toolchainStatus();
  res.json(GetCompilerStatusResponse.parse({
    version: compilerVersion, ...toolchain, target: "WebAssembly (browser) / Linux native (CLI)",
    features: [
      "Rust lexer, parser, semantic checks and direct LLVM IR generation",
      "int (64-bit), float (64-bit), bool, char, string",
      "Left-to-right assignment, input, function and output pipelines",
      "when/other, repeat, iterate, break, continue, back",
      "Bound-checked vectors, vector initialization and string operations",
      "Ranked multidimensional tensors, checked coordinates and scalar broadcast fills",
      "Shaped tensor parameters and returns, checked dimensions, leading-axis slices",
      "Numeric tensor/scalar arithmetic, vector/matrix multiplication, exp, sum, max, argmax",
      "Module-level scalar/vector/tensor data, shared mutation and checked initialization",
      "ClairII outer products (.cross), parenthesized calls, namespaces and explicit tensor policies",
      "Typed pipeline destinations and string.append with checked string rebinding",
      "Neuro-Symbolic Branching: infer/fall, inclusive fuzzy thresholds (~), controlled escalation (^>)",
      "Classes, fields, object instances and member functions",
      "Compile-time numeric &T / &Op function specialization",
      "Text-file I/O (virtual browser filesystem or native Linux files)",
      "CLI module linking, LLVM IR, native executables, WebAssembly",
    ],
    limitations: [
      "Multitex concurrency is not implemented; rejected with diagnostics.",
      "Memory is region-owned with an 8 MiB allocation budget, not a tracing garbage collector.",
      "comp blocks support definitions and generic specialization, not arbitrary compile-time execution.",
      "Browser compilation is single-module; external links are CLI-only.",
      "No inheritance, object-valued fields, generic methods, or binary/hex file encoding.",
      "Browser runs are limited to 3 seconds, 64 KiB output and a per-run virtual filesystem.",
      "Escalation reports a structured handoff; no external arbiter is connected.",
      "No built-in axiom verifier, calibrated confidence or pretrained tokenizer: these require explicit Sail functions. No trained AI, snapshot/resume or supervisor injection is provided.",
      "No batched tensor products, automatic differentiation, or model loading. Tensors are limited to 500,000 cells; matrix products to 50 million multiply-accumulate steps.",
    ],
  }));
});
router.get("/sail/examples", async (_req, res): Promise<void> => {
  const list = await Promise.all(examples.map(async (example) => ({
    ...example, source: await readFile(path.join(compilerRoot, `examples/${example.id}.sl`), "utf8"),
  })));
  res.json(GetExamplesResponse.parse(list));
});
router.get("/sail/reference", async (_req, res): Promise<void> => {
  const [markdown, notes, spatial, neuro, math, clair] = await Promise.all([
    readFile(path.join(compilerRoot, "docs/language.md"), "utf8"),
    readFile(path.join(compilerRoot, "docs/implementation.md"), "utf8"),
    readFile(path.join(compilerRoot, "docs/spatial-primitives.md"), "utf8"),
    readFile(path.join(compilerRoot, "docs/neuro-symbolic-branching.md"), "utf8"),
    readFile(path.join(compilerRoot, "docs/tensor-math.md"), "utf8"),
    readFile(path.join(compilerRoot, "docs/clairii.md"), "utf8"),
  ]);
  res.json(GetLanguageReferenceResponse.parse({ markdown: `${clair}\n\n${math}\n\n${neuro}\n\n${markdown}`, notes: `${notes}\n\n${spatial}` }));
});
router.post("/sail/compile", async (req, res): Promise<void> => {
  const body = CompileSailBody.safeParse(req.body);
  if (!body.success) {
    res.status(400).json({ error: "Source must be 1–65,536 characters; optimization must be 0, 1 or 2." });
    return;
  }
  res.json(CompileSailResponse.parse(await compileProgram(body.data.source, body.data.optimization)));
});
export default router;