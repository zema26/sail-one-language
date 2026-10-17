// Runtime checks on both real LLVM targets. Build sailc first.
// node compiler/tests/run-tensors.mjs
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { executeSail } from '../../artifacts/sail-playground/src/lib/wasm-runtime.ts';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const compiler = path.join(root, 'compiler/target/release/sailc');
const dir = mkdtempSync(path.join(tmpdir(), 'sail-tensor-tests-'));
const wrap = body => `module| T\nfun| > main > int\n${body}\nback| 0 |back\n|fun\n|module\n`;
const cases = [
  ['dynamic shapes and row-major coordinates',
    wrap("int rows(2) cols(3)\nint t|2, rows, cols|\niterate| int i(0)++ < rows\niterate| int j(0)++ < cols\ni * 10 + j > t|i, j|\n|iterate\n|iterate\nt|0, 2| ' ' t|1, 0| ' ' t|1, 2| ' ' t.length > out"), '2 10 12 6'],
  ['five-dimensional tensor',
    wrap("int t|5, 2, 2, 2, 2, 2|\n7 > t|*|\n42 > t|1, 0, 1, 0, 1|\nt|0, 0, 0, 0, 0| ' ' t|1, 0, 1, 0, 1| ' ' t.length > out"), '7 42 32'],
  ['rank-one and original vector',
    wrap("int t|1, 3|\nint v|3|\n2 > t|*|\n3 > v|*|\nt|2| ' ' v|2| > out"), '2 3'],
  ['float bits, promotion and chained broadcast',
    wrap("float t|2, 2, 3|\n-1.25 > t|*|\nt|1, 2| ' ' > out\n3 > t|*| > out\n'total:' t|0, 0| > out"), '-1.25 3total:3'],
  ['source and coordinates evaluated once',
    wrap("int t|2, 2, 2|\nint i(0) j(0)\n(i++ + 5) > t|*|\n8 > t|j++, 1|\nt|0, 0| ' ' t|0, 1| ' ' i ' ' j > out"), '5 8 1 1'],
  ['boolean broadcast and overwrite',
    wrap("bool t|2, 2, 2|\ntrue > t|*|\nfalse > t|0, 1|\nt|1, 0| ' ' t|0, 1| > out\nfalse > t|*|\n'then:' t|1, 0| > out"), 'true falsethen:false'],
  ['Unicode character broadcast',
    wrap("char t|2, 2, 2|\n'λ' > t|*|\nt|1, 1| > out"), 'λ'],
  ['string defaults and broadcast',
    wrap("string t|2, 2, 2|\nt|1, 1| > out\n'test' > t|*|\n'test' + '!' > t|0, 1|\nt|1, 0| ' ' t|0, 1| > out"), 'test test!'],
  ['input broadcast',
    wrap("float t|2, 2, 2|\nin > t|*|\nt|0, 1| ' ' t|1, 0| > out"), '2.5 2.5', '2.5'],
  ['tensor class fields and methods',
    "module| T\nclass| Layer\nfloat t|2, 2, 3|\nfun| int unused > Fill > int\n4.5 > t|*|\nback| 0 |back\n|fun\n|class\nfun| > main > int\nLayer l\n0 > l.Fill\nl.t|1, 2| > out\nback| 0 |back\n|fun\n|module\n", '4.5'],
  ['empty tensors, including zero after large extents',
    wrap("int t|3, 500000, 500000, 0|\n2 > t|*|\nt.length > out"), '0'],
  ['maximum total cell count',
    wrap("int t|2, 1000, 500|\n12 > t|999, 499|\nt.length ' ' t|999, 499| > out"), '500000 12'],
];
const traps = [
  ['per-axis read bounds', "int t|2, 2, 3|\nt|0, 3| > out", /bounds/],
  ['per-axis write bounds', "int t|2, 2, 3|\n1 > t|2, 0|", /bounds/],
  ['negative coordinate', "int t|2, 2, 3|\nt|-1, 0| > out", /bounds/],
  ['empty tensor access', "int t|2, 0, 3|\nt|0, 0| > out", /bounds/],
  ['negative runtime extent', "int n(-1)\nint t|2, n, 3|", /Heap limit|allocation limit/],
  ['extent overflow', "int t|2, 9223372036854775807, 2|", /Heap limit|allocation limit/],
  ['product overflow', "int t|3, 500000, 500000, 500000|", /Heap limit|allocation limit/],
  ['aggregate allocation budget', "iterate| int i(0)++ < 30\nint t|2, 1000, 500|\n|iterate", /Heap limit|allocation limit/],
];

async function build(source, target, name) {
  const input = path.join(dir, `${name}.sl`);
  const output = path.join(dir, `${name}.${target}`);
  writeFileSync(input, source);
  execFileSync(compiler, [input, `--${target}`, '-O2', '-o', output], {
    timeout: 10000, stdio: 'pipe',
  });
  return output;
}
try {
  for (let i = 0; i < cases.length; i++) {
    const [name, source, output, stdin = ''] = cases[i];
    if (process.argv[2] && !new RegExp(process.argv[2]).test(name)) continue;
    const wasm = await build(source, 'wasm', `success-${i}`);
    const result = await executeSail(new Uint8Array(readFileSync(wasm)), stdin, () => {});
    assert.equal(result.stdout, output, `${name}: WebAssembly output`);
    assert.equal(result.exitCode, 0, `${name}: WebAssembly status`);
    const native = await build(source, 'native', `success-${i}`);
    assert.equal(execFileSync(native, { input: stdin, encoding: 'utf8', timeout: 3000 }), output, `${name}: native output`);
    console.info(`PASS ${name}: WebAssembly + native`);
  }
  for (let i = 0; i < traps.length; i++) {
    const [name, body, error] = traps[i];
    if (process.argv[2] && !new RegExp(process.argv[2]).test(name)) continue;
    const wasm = await build(wrap(body), 'wasm', `trap-${i}`);
    await assert.rejects(() => executeSail(new Uint8Array(readFileSync(wasm)), '', () => {}), error, `${name}: WebAssembly trap`);
    const native = await build(wrap(body), 'native', `trap-${i}`);
    const result = spawnSync(native, { encoding: 'utf8', timeout: 3000 });
    assert.equal(result.status, 70, `${name}: native trap status`);
    assert.match(result.stderr, error, `${name}: native trap message`);
    console.info(`PASS ${name}: WebAssembly + native traps`);
  }
} finally {
  rmSync(dir, { recursive: true, force: true });
}
