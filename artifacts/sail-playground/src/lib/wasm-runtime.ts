/** The only host capabilities supplied to a compiled Sail module. */
type SailExports = WebAssembly.Exports & {
  memory: WebAssembly.Memory;
  sail_main: () => bigint;
  sail_alloc: (bytes: bigint) => number;
};
type VirtualFile = { path: string; mode: string; open: boolean; opened: boolean; offset: number };

export type Escalation = { status: 'HALT_AND_ESCALATE'; state: string; payload: string };
export type SailRunResult = {
  stdout: string; exitCode: number; durationMs: number; escalation?: Escalation;
};
class EscalationSignal {
  readonly escalation: Escalation;
  constructor(escalation: Escalation) { this.escalation = escalation; }
}

export async function executeSail(
  bytes: Uint8Array,
  stdin: string,
  write: (chunk: string) => void,
): Promise<SailRunResult> {
  const started = performance.now();
  const tokens = stdin.match(/\S+/gu) ?? [];
  let tokenIndex = 0;
  let exports: SailExports;
  let stdout = '';
  const files = new Map<string, string>();
  const handles = new Map<number, VirtualFile>();
  const decoder = new TextDecoder('utf-8', { fatal: true });
  const encoder = new TextEncoder();

  const string = (ptr: number) => {
    const mem = new Uint8Array(exports.memory.buffer);
    if (ptr <= 0 || ptr >= mem.length) throw new Error('Invalid string pointer.');
    let end = ptr;
    while (end < mem.length && mem[end] !== 0) end++;
    if (end === mem.length) throw new Error('Unterminated string.');
    return decoder.decode(mem.subarray(ptr, end));
  };
  const allocateString = (value: string) => {
    const data = encoder.encode(value);
    const ptr = exports.sail_alloc(BigInt(data.length + 1));
    const mem = new Uint8Array(exports.memory.buffer);
    mem.set(data, ptr);
    mem[ptr + data.length] = 0;
    return ptr;
  };
  const output = (text: string) => {
    if (stdout.length + text.length > 65536) throw new Error('Output limit exceeded (64 KiB).');
    stdout += text;
    write(text);
  };
  const token = () => {
    if (tokenIndex >= tokens.length) throw new Error('Standard input exhausted. Add the required values to stdin.');
    return tokens[tokenIndex++];
  };
  const file = (id: number, requireOpen = true) => {
    const f = handles.get(id);
    if (!f || (requireOpen && !f.open)) throw new Error('Invalid or closed virtual file.');
    return f;
  };
  const imports = {
    sail_print_int: (x: bigint) => output(x.toString()),
    sail_print_float: (x: number) => output(Number.isFinite(x) ? String(Number(x.toPrecision(12))) : String(x).toLowerCase()),
    sail_print_bool: (x: number) => output(x ? 'true' : 'false'),
    sail_print_char: (x: bigint) => {
      const cp = Number(x);
      if (cp < 0 || cp > 0x10ffff || (cp >= 0xd800 && cp <= 0xdfff)) throw new Error('Invalid Unicode character.');
      output(String.fromCodePoint(cp));
    },
    sail_print_string: (ptr: number) => output(string(ptr)),
    sail_read_int: () => {
      const s = token();
      if (!/^[+-]?\d+$/.test(s)) throw new Error(`Expected int input, received "${s}".`);
      const n = BigInt(s);
      if (n < -(1n << 63n) || n >= (1n << 63n)) throw new Error('Input integer is outside the signed 64-bit range.');
      return n;
    },
    sail_read_float: () => {
      const s = token();
      if (!/^[+-]?(?:\d+\.?\d*|\.\d+)(?:e[+-]?\d+)?$/i.test(s) || !Number.isFinite(Number(s))) {
        throw new Error(`Expected finite float input, received "${s}".`);
      }
      return Number(s);
    },
    sail_read_bool: () => {
      const s = token();
      if (!['true', 'false', '1', '0'].includes(s)) throw new Error(`Expected bool input, received "${s}".`);
      return s === 'true' || s === '1' ? 1 : 0;
    },
    sail_read_char: () => {
      const s = token();
      if ([...s].length !== 1) throw new Error('Character input must contain exactly one Unicode character.');
      return BigInt(s.codePointAt(0)!);
    },
    sail_read_string: () => allocateString(token()),
    sail_file_new: (path: number, mode: number) => {
      if (handles.size >= 32) throw new Error('Virtual file limit exceeded (32).');
      const f: VirtualFile = { path: string(path), mode: string(mode), open: false, opened: false, offset: 0 };
      if (!/^[rwa+]+$/.test(f.mode)) throw new Error('Invalid virtual file access mode.');
      const id = handles.size + 1;
      handles.set(id, f);
      return id;
    },
    sail_file_open: (id: number) => {
      const f = file(id, false);
      if (f.open) throw new Error('File is already open.');
      if (f.mode.includes('w') && !f.opened) files.set(f.path, '');
      else if (!files.has(f.path) && f.mode.includes('a')) files.set(f.path, '');
      else if (!files.has(f.path)) throw new Error(`Virtual file "${f.path}" does not exist.`);
      f.open = true;
      f.opened = true;
      f.offset = f.mode.includes('a') ? files.get(f.path)!.length : 0;
    },
    sail_file_close: (id: number) => { file(id).open = false; },
    sail_file_eof: (id: number) => {
      const f = file(id);
      return f.offset >= files.get(f.path)!.length ? 1 : 0;
    },
    sail_file_read: (id: number) => {
      const f = file(id);
      if (!f.mode.includes('r') && !f.mode.includes('+')) throw new Error('File is not open for reading.');
      const text = files.get(f.path)!;
      const end = text.indexOf('\n', f.offset);
      const line = text.slice(f.offset, end === -1 ? text.length : end);
      f.offset = end === -1 ? text.length : end + 1;
      return allocateString(line);
    },
    sail_file_write: (id: number, ptr: number) => {
      const f = file(id);
      if (!f.mode.includes('w') && !f.mode.includes('a')) throw new Error('File is not open for writing.');
      const chunk = string(ptr);
      const text = files.get(f.path)!;
      const next = text.slice(0, f.offset) + chunk + text.slice(f.offset + chunk.length);
      let total = next.length;
      for (const [path, value] of files) if (path !== f.path) total += value.length;
      if (total > 65536) throw new Error('Virtual filesystem limit exceeded (64 KiB).');
      files.set(f.path, next);
      f.offset += chunk.length;
    },
  };
  const panic = (code: bigint) => {
    const messages = ['Runtime failure.', 'Heap limit exceeded (8 MiB).', 'Vector index out of bounds.', 'Integer division by zero or overflow.', 'Invalid character literal.', 'File operation failed.', 'Invalid input.', 'Tensor shape mismatch.', 'Empty tensor reduction.', 'NaN in tensor maximum.', 'Tensor multiplication operation budget exceeded.'];
    throw new Error(messages[Number(code)] ?? messages[0]);
  };
  const module = await WebAssembly.compile(bytes as unknown as BufferSource);
  const escalate = (state: number, payload: number): never => {
    throw new EscalationSignal({ status: 'HALT_AND_ESCALATE', state: string(state), payload: string(payload) });
  };
  for (const imp of WebAssembly.Module.imports(module)) {
    if (imp.kind !== 'function' || !(imp.module === 'env' && Object.hasOwn(imports, imp.name) || imp.module === 'sail' && ['panic', 'escalate', 'exp'].includes(imp.name))) {
      throw new Error(`Module requests an unsupported capability: ${imp.module}.${imp.name}.`);
    }
  }
  const instance = await WebAssembly.instantiate(module, { env: imports, sail: { panic, escalate, exp: Math.exp } });
  exports = instance.exports as SailExports;
  if (!(exports.memory instanceof WebAssembly.Memory) || typeof exports.sail_main !== 'function' || typeof exports.sail_alloc !== 'function') {
    throw new Error('Invalid Sail module exports.');
  }
  try {
    const exit = exports.sail_main();
    return { stdout, exitCode: Number(BigInt.asIntN(32, exit)), durationMs: performance.now() - started };
  } catch (signal) {
    if (!(signal instanceof EscalationSignal)) throw signal;
    return { stdout, exitCode: 75, durationMs: performance.now() - started, escalation: signal.escalation };
  }
}