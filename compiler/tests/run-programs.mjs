// Backend/runtime integration checks, not browser/UI automation.
// Run with Node 24: node compiler/tests/run-programs.mjs
// Native-only release check (Node 18+; no frontend or Rust needed):
// node compiler/tests/run-programs.mjs /path/to/sailc --native-only --examples-dir /path/to/bundle/examples
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const args = process.argv.slice(2);
let compiler = path.join(root, 'compiler/target/release/sailc');
let examples = path.join(root, 'compiler/examples'), nativeOnly = false;
while (args.length) {
  const arg = args.shift();
  if (arg === '--native-only') nativeOnly = true;
  else if (arg === '--examples-dir') {
    const value = args.shift();
    assert.ok(value && !value.startsWith('--'), '--examples-dir requires a directory');
    examples = path.resolve(value);
  } else {
    assert.ok(!arg.startsWith('-'), `Unknown argument: ${arg}`);
    compiler = path.resolve(arg);
  }
}
const executeSail = nativeOnly ? undefined
  : (await import('../../artifacts/sail-playground/src/lib/wasm-runtime.ts')).executeSail;
const dir = mkdtempSync(path.join(tmpdir(), 'sail-program-tests-'));
const expectedPrimes = '2 3 5 7 11 13 17 19 23 29 31\n 37 41 43 47 53 59 61 67 71 73\n 79 83 89 97\nnumber: 24\n';
const cases = [
  ['hello', '', 'Hello, Sail!\n'],
  ['euclidean', '48 18', '6\n'],
  ['classes', '84 30', '6\n'],
  ['generics', '', '3.14\n3\n2.4393\n2\n'],
  ['vectors', '', 'Sum: 10\nHello, Sail!\n'],
  ['files', '', 'Sail writes from left to right.\n'],
  ['primes', '', expectedPrimes],
  ['tensors', '', 'Matrix: 1.5 2.5\nElements: 6\nVolume: 7 9\nVector: 4\n'],
  ['neuro-symbolic', '', 'Recalculating\nThreshold met\n'],
  ['token-decoder', '0.90', 'Output: candidate\n'],
  ['tensor-math', '1', 'Output: candidate\n'],
  ['clairii', 'hello', 'accepted <EOT>\nUpdates: 2\nLiquid memory[0,0]: 0.0201\n'],
];
try {
  for (const [name, stdin, expected] of cases) {
    const source = path.join(examples, `${name}.sl`);
    if (!nativeOnly) {
      const wasm = path.join(dir, `${name}.wasm`);
      execFileSync(compiler, [source, '--wasm', '-O1', '-o', wasm], { timeout: 10000 });
      const result = await executeSail(new Uint8Array(readFileSync(wasm)), stdin, () => {});
      assert.equal(result.stdout, expected, `${name} WebAssembly output`);
      assert.equal(result.exitCode, 0, `${name} exit status`);
    }
    const native = path.join(dir, `${name}.native`);
    execFileSync(compiler, [source, '--native', '-O2', '-o', native], { timeout: 10000 });
    const output = execFileSync(native, { input: stdin, encoding: 'utf8', timeout: 3000, cwd: dir });
    assert.equal(output, expected, `${name} native output`);
    console.info(`PASS ${name}: ${nativeOnly ? '' : 'LLVM WebAssembly + '}Linux native`);
  }
  if (!nativeOnly) {
    const wrap = (body) => `module| T\nfun| > main > int\n${body}\nback| 0 |back\n|fun\n|module\n`;
    const execute = async (body, stdin = '') => {
      const source = path.join(dir, 'check.sl'), wasm = path.join(dir, 'check.wasm');
      writeFileSync(source, wrap(body));
      execFileSync(compiler, [source, '--wasm', '-O2', '-o', wasm], { timeout: 10000 });
      return executeSail(new Uint8Array(readFileSync(wasm)), stdin, () => {});
    };
    assert.equal((await execute("when| false and (1 / 0 > 2)\n'bad' > out\n|when\n'good' > out")).stdout, 'good');
    assert.equal((await execute("int n(0)\niterate| int i(0)++ < 10\nwhen| i == 2\ncontinue\n|when\nwhen| i == 5\nbreak\n|when\nn + i > n\n|iterate\nn > out")).stdout, '8');
    assert.equal((await execute("char c('λ')\nc > out")).stdout, 'λ');
    assert.equal((await execute("bool b\nin > b\nb > out", 'true')).stdout, 'true');
    assert.equal((await execute("int a(0) b(0)\niterate| a(0)++ < 3\niterate| b(0)++ < 2\n|iterate\n|iterate\na ' ' b > out")).stdout, '3 2');
    await assert.rejects(() => execute("int a|2|\na|2| > out"), /bounds/);
    await assert.rejects(() => execute("1 / 0 > out"), /division/);
    await assert.rejects(() => execute("int a\nin > a", 'not-a-number'), /Expected int/);
    await assert.rejects(() => execute("int a|500001|"), /Heap limit/);
    console.info('PASS runtime traps, short-circuit evaluation, loop control, input, characters');
  }
} finally {
  rmSync(dir, { recursive: true, force: true });
}