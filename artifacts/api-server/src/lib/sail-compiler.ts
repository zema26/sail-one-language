import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { mkdtemp, readFile, writeFile, rm, access } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";

const exec = promisify(execFile);
export const compilerRoot = path.resolve(import.meta.dirname, "../../../compiler");
// Bundled runtime lives in artifacts/api-server/dist, exactly three levels below root.
export const compilerBinary = path.join(compilerRoot, "target/release/sailc");
export const compilerVersion = "sailc 0.1.1";
let active = 0;

export async function toolchainStatus() {
  try {
    await access(compilerBinary);
    await exec("wasm-ld", ["--version"], { timeout: 3000, maxBuffer: 8192 });
    const { stdout } = await exec("clang", ["--version"], { timeout: 3000, maxBuffer: 8192 });
    return { ready: true, backend: stdout.split("\n")[0] };
  } catch {
    return { ready: false, backend: "LLVM / Clang (toolchain unavailable)" };
  }
}
export async function compileProgram(source: string, optimization: string) {
  const started = performance.now();
  const base = { compilerVersion, llvmIr: "", wasmBase64: "" };
  if (active >= 2) {
    return { ...base, success: false, durationMs: 0, diagnostics: [{ severity: "error", line: 1, column: 1, message: "Compiler busy. Please retry shortly." }] };
  }
  active++;
  let dir: string | undefined;
  try {
    dir = await mkdtemp(path.join(tmpdir(), "sail-"));
    const input = path.join(dir, "program.sl");
    const output = path.join(dir, "program.wasm");
    await writeFile(input, source, "utf8");
    // Arguments are fixed/validated. Source is never passed to a shell.
    await exec(compilerBinary, [input, "--wasm", "--json", `-O${optimization}`, "-o", output], {
      timeout: 10000,
      killSignal: "SIGKILL",
      maxBuffer: 1024 * 1024,
      env: { PATH: process.env.PATH, HOME: tmpdir() },
    });
    const [wasm, llvmIr] = await Promise.all([readFile(output), readFile(`${output}.ll`, "utf8")]);
    return { success: true, diagnostics: [], llvmIr, wasmBase64: wasm.toString("base64"), durationMs: performance.now() - started, compilerVersion };
  } catch (error) {
    const err = error as Error & { stdout?: string; killed?: boolean; code?: string };
    let message = "Compiler toolchain failed. Check that Rust, Clang and LLD are installed.";
    let line = 1, column = 1;
    try {
      const diagnostic = JSON.parse(err.stdout ?? "");
      if (typeof diagnostic.message === "string") {
        message = diagnostic.message.replace(/\/(?:home|tmp)\/[^\s:]+/g, "<compiler>");
        line = diagnostic.line ?? 1; column = diagnostic.column ?? 1;
      }
    } catch {
      if (err.killed) message = "Compilation exceeded its 10-second limit.";
    }
    return { ...base, success: false, durationMs: performance.now() - started, diagnostics: [{ severity: "error", message, line, column }] };
  } finally {
    active--;
    if (dir) await rm(dir, { recursive: true, force: true });
  }
}