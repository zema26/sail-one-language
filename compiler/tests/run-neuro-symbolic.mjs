// Run with Node 24 after building sailc.
// Real LLVM/native and WebAssembly execution; no mocked compiler or runtime.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { executeSail } from '../../artifacts/sail-playground/src/lib/wasm-runtime.ts';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const compiler = path.join(root, 'compiler/target/release/sailc');
const dir = mkdtempSync(path.join(tmpdir(), 'sail-neuro-tests-'));
const wrap = body => `module| T\nfun| > main > int\n${body}\n0 > |back|\n|fun\n|module\n`;
const example = name => readFileSync(path.join(root, `compiler/examples/${name}.sl`), 'utf8');
const cases = [
  ['document threshold example', example('neuro-symbolic'), '', 'Recalculating\nThreshold met\n'],
  ['decoder high-confidence path', example('token-decoder'), '0.90', 'Output: candidate\n'],
  ['decoder equality path', example('token-decoder'), '0.85', 'Output: candidate\n'],
  ['both operands evaluated once',
    wrap("int score(3) threshold(2)\ninfer| score++ ~ threshold++\n'Y' > out\nfall|\n'N' > out\n|infer\nscore threshold > out"), '', 'Y43'],
  ['inclusive comparison and numeric promotion',
    wrap("(0.85 ~ 0.85) ' ' (0.84 ~ 0.85) ' ' (1 ~ 0.9) ' ' (1.0 ~ 1) > out"), '', 'true false true true'],
  ['integer comparisons retain 64-bit precision',
    wrap("(9007199254740993 ~ 9007199254740994) ' ' (9223372036854775807 ~ 9223372036854775807) > out"), '', 'false true'],
  ['arithmetic and boolean precedence',
    wrap("infer| 0.50 + 0.40 ~ 0.85 and 2 * 2 ~ 4\n'yes' > out\nfall|\n'no' > out\n|infer"), '', 'yes'],
  ['short-circuiting skips failing expressions',
    wrap("infer| true or (1 / 0 ~ 1)\n'A' > out\n|infer\ninfer| false and (1 / 0 ~ 1)\n'bad' > out\nfall|\n'B' > out\n|infer"), '', 'AB'],
  ['NaN fails the confidence threshold',
    wrap("infer| (0.0 / 0.0) ~ 0.5\n'bad' > out\nfall|\n'fallback' > out\n|infer"), '', 'fallback'],
  ['nested infer branches and branch-local variables',
    wrap("infer| 0.9 ~ 0.8\nint n(2)\ninfer| false\n'bad' > out\nfall|\nn > out\n|infer\nfall|\nint n(3)\nn > out\n|infer"), '', '2'],
  ['optional fall and loop control',
    wrap("iterate| int i(0)++ < 5\ninfer| i ~ 3\nbreak\nfall|\ninfer| i == 1\ncontinue\n|infer\ni > out\n|infer\n|iterate\ninfer| false\n'bad' > out\n|infer"), '', '02'],
  ['tensor element confidence',
    wrap("float p|2, 1, 2|\n0.9 > p|*|\ninfer| p|0, 1| ~ 0.85\n'accepted' > out\nfall|\n'bad' > out\n|infer"), '', 'accepted'],
  ['function-pipeline return shorthand',
    "module| T\nfun| int n > Id > int\nn > |back|\n|fun\nfun| > main > int\n7 > Id > |back|\n|fun\n|module\n", '', '', 7],
];
const specialState = 'VETO"\n\\λ';
const specialPayload = 'candidate"\t\\\nλ';
const escalations = [
  ['document decoder fallback', example('token-decoder'), '0.60',
    'HALT_AND_ESCALATE\n', 'TEMPORAL_LOCK', 'candidate'],
  ['nested-call escalation stops the caller',
    "module| T\nfun| int unused > Stop > string\n'inner\\n' > out\n'LOCK' ^> 'review'\n'UNREACHABLE' > |back|\n|fun\nfun| > main > int\nstring token('')\n'outer\\n' > out\n0 > Stop > token\n'UNREACHABLE' > out\n0 > |back|\n|fun\n|module\n", '', 'outer\ninner\n', 'LOCK', 'review'],
  ['escalation inside a loop stops all execution',
    wrap("iterate| int i(0)++ < 5\ninfer| i ~ 2\n'VETO' ^> 'loop'\nfall|\ni > out\n|infer\n|iterate\n'UNREACHABLE' > out"), '', '01', 'VETO', 'loop'],
  ['JSON escaping, unicode and captured payload',
    wrap(`string selected(${JSON.stringify(specialPayload)})\n${JSON.stringify(specialState)} ^> selected\n'UNREACHABLE' > out`), '', '', specialState, specialPayload],
  ['empty state and payload are preserved', wrap("'' ^> ''\n'UNREACHABLE' > out"), '', '', '', ''],
];
function build(source, target, name, optimization) {
  const input = path.join(dir, `${name}.sl`), output = path.join(dir, `${name}.${target}`);
  writeFileSync(input, source);
  execFileSync(compiler, [input, `--${target}`, `-O${optimization}`, '-o', output], { timeout: 15000, stdio: 'pipe' });
  return output;
}
try {
  for (const optimization of [0, 2]) {
    for (let i = 0; i < cases.length; i++) {
      const [name, source, stdin, stdout, exitCode = 0] = cases[i];
      const wasm = build(source, 'wasm', `normal-${i}`, optimization);
      const result = await executeSail(new Uint8Array(readFileSync(wasm)), stdin, () => {});
      assert.equal(result.stdout, stdout, `${name}: Wasm output`);
      assert.equal(result.exitCode, exitCode, `${name}: Wasm status`);
      assert.equal(result.escalation, undefined, `${name}: no unexpected escalation`);
      const native = spawnSync(build(source, 'native', `normal-${i}`, optimization), { input: stdin, encoding: 'utf8', timeout: 3000 });
      assert.equal(native.status, exitCode, `${name}: native status, ${native.stderr}`);
      assert.equal(native.signal, null);
      assert.equal(native.stdout, stdout, `${name}: native output`);
      assert.equal(native.stderr, '');
      console.info(`PASS ${name}: Wasm + native -O${optimization}`);
    }
    for (let i = 0; i < escalations.length; i++) {
      const [name, source, stdin, stdout, state, payload] = escalations[i];
      const wasm = build(source, 'wasm', `escalation-${i}`, optimization);
      const streamed = [];
      const result = await executeSail(new Uint8Array(readFileSync(wasm)), stdin, chunk => streamed.push(chunk));
      assert.equal(result.exitCode, 75, `${name}: Wasm controlled halt`);
      assert.equal(result.stdout, stdout);
      assert.equal(streamed.join(''), stdout, `${name}: output preserved`);
      assert.deepEqual(result.escalation, { status: 'HALT_AND_ESCALATE', state, payload });
      const native = spawnSync(build(source, 'native', `escalation-${i}`, optimization), { input: stdin, encoding: 'utf8', timeout: 3000 });
      assert.equal(native.status, 75, `${name}: native controlled halt, ${native.stderr}`);
      assert.equal(native.signal, null, `${name}: not a crash`);
      assert.equal(native.stdout, stdout);
      assert.deepEqual(JSON.parse(native.stderr), result.escalation, `${name}: identical native handoff`);
      console.info(`PASS ${name}: Wasm + native controlled halt -O${optimization}`);
    }
  }
} finally {
  rmSync(dir, { recursive: true, force: true });
}
