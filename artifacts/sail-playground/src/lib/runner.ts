import type { SailRunResult } from './wasm-runtime';

export function runSail(
  wasmBase64: string,
  stdin: string,
  onOutput?: (chunk: string) => void,
): Promise<SailRunResult> {
  if (stdin.length > 65536) return Promise.reject(new Error('Standard input limit exceeded (64 KiB).'));
  return new Promise((resolve, reject) => {
    const worker = new Worker(new URL('./sail.worker.ts', import.meta.url), { type: 'module' });
    const timer = setTimeout(() => {
      worker.terminate();
      reject(new Error('Execution exceeded 3 seconds. Check for an infinite loop or use a smaller input.'));
    }, 3000);
    const finish = () => { clearTimeout(timer); worker.terminate(); };
    worker.onmessage = ({ data }) => {
      if (data.type === 'output') onOutput?.(data.chunk);
      if (data.type === 'done') { finish(); resolve(data.result); }
      if (data.type === 'error') { finish(); reject(new Error(data.message)); }
    };
    worker.onerror = (event) => { finish(); reject(new Error(event.message || 'Worker failed to start.')); };
    worker.postMessage({ wasmBase64, stdin });
  });
}