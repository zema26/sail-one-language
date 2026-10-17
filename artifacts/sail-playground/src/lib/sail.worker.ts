import { executeSail } from './wasm-runtime';

const workerScope = self as unknown as {
  onmessage: ((event: MessageEvent<{ wasmBase64: string; stdin: string }>) => void) | null;
  postMessage: (message: unknown) => void;
};
workerScope.onmessage = async ({ data }) => {
  let chunk = '';
  try {
    const bytes = Uint8Array.from(atob(data.wasmBase64), (c) => c.charCodeAt(0));
    // Batch output so loops cannot flood the browser with tiny messages.
    const result = await executeSail(bytes, data.stdin, (text) => {
      chunk += text;
      if (chunk.length >= 1024) { workerScope.postMessage({ type: 'output', chunk }); chunk = ''; }
    });
    if (chunk) workerScope.postMessage({ type: 'output', chunk });
    workerScope.postMessage({ type: 'done', result });
  } catch (error) {
    if (chunk) workerScope.postMessage({ type: 'output', chunk });
    workerScope.postMessage({ type: 'error', message: error instanceof Error ? error.message : 'WebAssembly execution failed.' });
  }
};