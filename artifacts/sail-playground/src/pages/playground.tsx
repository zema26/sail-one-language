import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useCompileSail, useGetExamples, type CompilationResult } from '@workspace/api-client-react';
import { runSail } from '@/lib/runner';
import type { Escalation } from '@/lib/wasm-runtime';
import { base64ToBlob, download, usePersisted } from '@/lib/storage';
import { CodeEditor } from '@/components/sail/code-editor';
import { Shell } from '@/components/sail/shell';
import { Skeleton } from '@/components/ui/skeleton';

type Opt = '0' | '1' | '2';
type Tab = 'output' | 'diagnostics' | 'ir';
type Phase = 'idle' | 'compiling' | 'running';
interface RunState { stdout: string; exitCode: number | null; durationMs: number | null; error: string | null; escalation?: Escalation }

const STARTER = `module| Hello
    fun| string args| | > main > int
        'Hello, Sail!\\n' > out
        back| 0 |back
    |fun
|module
`;
const emptyRun: RunState = { stdout: '', exitCode: null, durationMs: null, error: null };

const btn =
  'inline-flex items-center gap-2 rounded border border-border bg-secondary px-3 py-1.5 text-xs font-medium text-foreground transition-all hover:border-primary/60 active:translate-y-px disabled:pointer-events-none disabled:opacity-40';

export default function Playground() {
  const [source, setSource] = usePersisted('sail.source', STARTER);
  const [stdin, setStdin] = usePersisted('sail.stdin', '');
  const [opt, setOpt] = usePersisted<Opt>('sail.opt', '1');
  const [exampleId, setExampleId] = usePersisted('sail.example', '');
  const [tab, setTab] = useState<Tab>('output');
  const [phase, setPhase] = useState<Phase>('idle');
  const [result, setResult] = useState<CompilationResult | null>(null);
  const [compileError, setCompileError] = useState<string | null>(null);
  const [run, setRun] = useState<RunState>(emptyRun);
  const runId = useRef(0);

  const examples = useGetExamples();
  const compile = useCompileSail();
  const compileRef = useRef(compile.mutateAsync);
  compileRef.current = compile.mutateAsync;

  const busy = phase !== 'idle';

  const doCompile = useCallback(
    async (andRun: boolean) => {
      const id = ++runId.current;
      setPhase('compiling');
      setCompileError(null);
      setRun(emptyRun);
      setTab('output');
      let res: CompilationResult;
      try {
        res = await compileRef.current({ data: { source, optimization: opt } });
      } catch (e) {
        if (id !== runId.current) return;
        setResult(null);
        setCompileError(e instanceof Error ? e.message : 'The compile request failed.');
        setPhase('idle');
        return;
      }
      if (id !== runId.current) return;
      setResult(res);
      if (!res.success) {
        setTab('diagnostics');
        setPhase('idle');
        return;
      }
      if (!andRun) {
        setPhase('idle');
        return;
      }
      if (!res.wasmBase64) {
        setRun({ ...emptyRun, error: 'Compilation succeeded but returned no WebAssembly module.' });
        setPhase('idle');
        return;
      }
      setPhase('running');
      try {
        const r = await runSail(res.wasmBase64, stdin, (chunk: string) => {
          if (id === runId.current) setRun((p) => ({ ...p, stdout: p.stdout + chunk }));
        });
        if (id !== runId.current) return;
        setRun({ stdout: r.stdout, exitCode: r.exitCode, durationMs: r.durationMs, error: null, escalation: r.escalation });
      } catch (e) {
        if (id !== runId.current) return;
        setRun((p) => ({ ...p, error: e instanceof Error ? e.message : 'Execution failed.' }));
      }
      setPhase('idle');
    },
    [source, stdin, opt],
  );

  const runRef = useRef(doCompile);
  runRef.current = doCompile;
  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
        e.preventDefault();
        if (phase === 'idle') void runRef.current(true);
      }
    };
    window.addEventListener('keydown', h);
    return () => window.removeEventListener('keydown', h);
  }, [phase]);

  const diags = result?.diagnostics ?? [];
  const errLines = useMemo(() => diags.filter((d) => d.severity === 'error').map((d) => d.line), [diags]);
  const warnLines = useMemo(() => diags.filter((d) => d.severity === 'warning').map((d) => d.line), [diags]);

  const pick = (id: string) => {
    setExampleId(id);
    const ex = examples.data?.find((x) => x.id === id);
    if (ex) {
      runId.current++;
      setPhase('idle');
      setSource(ex.source);
      setStdin(ex.stdin);
      setResult(null);
      setRun(emptyRun);
      setCompileError(null);
    }
  };
  const fresh = () => {
    runId.current++;
    setPhase('idle');
    setSource(STARTER);
    setStdin('');
    setExampleId('');
    setResult(null);
    setRun(emptyRun);
    setCompileError(null);
  };
  const reset = () => {
    const ex = examples.data?.find((x) => x.id === exampleId);
    if (ex) pick(ex.id);
    else fresh();
  };

  const selected = examples.data?.find((x) => x.id === exampleId);
  const featured = [
    { id: 'clairii', title: 'ClairII language features are supported', text: 'Shared module tensors, outer products, string append, and explicit axiom/confidence policies. Includes a small runnable example, not a trained AI.', button: 'Load ClairII example' },
    { id: 'tensor-math', title: 'Tensor signatures and math are supported', text: 'Shaped functions, matrix/vector products, softmax, exp, sum, max, and argmax.', button: 'Load tensor math example' },
    { id: 'neuro-symbolic', title: 'Neuro-Symbolic Branching is supported', text: 'infer| / fall| blocks, confidence ~ threshold, and controlled ^> escalation.', button: 'Load branching example' },
    { id: 'tensors', title: 'Multidimensional arrays are supported', text: 'Ranked shapes, checked coordinates, and scalar broadcast fills using |*|.', button: 'Load tensor example' },
  ].find((feature) => examples.data?.some((example) => example.id === feature.id));

  return (
    <Shell>
      {featured && (
        <section className="flex flex-wrap items-center justify-between gap-3 border-b border-accent/30 bg-accent/5 px-4 py-3" data-testid="tensor-update">
          <div>
            <p className="text-sm font-semibold text-accent">{featured.title}</p>
            <p className="text-xs text-muted-foreground">{featured.text}</p>
          </div>
          <div className="flex flex-wrap items-center gap-3">
            <span className="text-xs text-muted-foreground">Loading an example replaces the editor contents.</span>
            <button type="button" className={btn} onClick={() => pick(featured.id)} disabled={busy} data-testid={`button-${featured.id}-example`}>
              {featured.button}
            </button>
          </div>
        </section>
      )}
      <div className="flex flex-wrap items-center gap-2 border-b border-border px-4 py-2.5">
        <button
          data-testid="button-run"
          onClick={() => void doCompile(true)}
          disabled={busy || !source.trim()}
          className="relative inline-flex items-center gap-2 overflow-hidden rounded bg-primary px-4 py-1.5 text-sm font-bold text-primary-foreground transition-transform hover:brightness-110 active:translate-y-px disabled:opacity-60"
        >
          {busy && <span className="sweep absolute inset-0" />}
          <span className="relative">{phase === 'compiling' ? 'Compiling' : phase === 'running' ? 'Running' : 'Compile and run'}</span>
          {!busy && <kbd className="relative rounded bg-background/20 px-1.5 font-mono text-[10px]">Ctrl+Enter</kbd>}
        </button>
        <button data-testid="button-compile" className={btn} onClick={() => void doCompile(false)} disabled={busy || !source.trim()}>
          Compile only
        </button>
        <label className="ml-2 flex items-center gap-2 text-xs text-muted-foreground">
          Optimization
          <span className="flex overflow-hidden rounded border border-border">
            {(['0', '1', '2'] as Opt[]).map((o) => (
              <button
                key={o}
                data-testid={`button-opt-${o}`}
                onClick={() => setOpt(o)}
                className={`px-2.5 py-1 font-mono transition-colors ${opt === o ? 'bg-accent text-accent-foreground' : 'bg-card hover:bg-secondary'}`}
              >
                O{o}
              </button>
            ))}
          </span>
        </label>
        <div className="ml-auto flex flex-wrap items-center gap-2">
          <select
            data-testid="select-example"
            value={exampleId}
            onChange={(e) => pick(e.target.value)}
            disabled={!examples.data}
            className="max-w-[220px] rounded border border-border bg-card px-2 py-1.5 text-xs outline-none focus:border-primary"
            aria-label="Examples"
          >
            <option value="">{examples.isLoading ? 'Loading examples' : examples.isError ? 'Examples unavailable' : 'Examples'}</option>
            {examples.data?.map((x) => (
              <option key={x.id} value={x.id}>
                {x.category} / {x.title}
              </option>
            ))}
          </select>
          {examples.isError && (
            <button className={btn} onClick={() => examples.refetch()} data-testid="button-retry-examples">Retry</button>
          )}
          <button className={btn} onClick={reset} data-testid="button-reset">Reset</button>
          <button className={btn} onClick={fresh} data-testid="button-new">New</button>
        </div>
      </div>

      <main className="grid flex-1 grid-cols-1 lg:grid-cols-2 lg:h-[calc(100dvh-101px)]">
        <section className="flex min-h-0 flex-col border-b border-border lg:border-b-0 lg:border-r">
          <div className="flex items-center justify-between border-b border-border px-4 py-1.5 text-xs text-muted-foreground">
            <span className="font-mono">main.sl{selected ? ` - ${selected.description}` : ''}</span>
            <span className="font-mono">{source.length.toLocaleString()} / 65,536</span>
          </div>
          <div className="min-h-[320px] flex-1">
            <CodeEditor value={source} onChange={setSource} errorLines={errLines} warnLines={warnLines} />
          </div>
          <div className="border-t border-border">
            <div className="px-4 py-1.5 text-xs uppercase tracking-[0.18em] text-muted-foreground">stdin</div>
            <textarea
              data-testid="input-stdin"
              value={stdin}
              onChange={(e) => setStdin(e.target.value)}
              spellCheck={false}
              placeholder="Input given to the program"
              className="block h-24 w-full resize-y bg-card/50 px-4 py-2 font-mono text-[13px] outline-none placeholder:text-muted-foreground/50 focus:bg-card"
            />
          </div>
        </section>

        <section className="flex min-h-[360px] min-w-0 flex-col">
          <div className="flex items-center border-b border-border">
            {(
              [
                ['output', 'Output'],
                ['diagnostics', `Diagnostics${result ? ` (${diags.length})` : ''}`],
                ['ir', 'LLVM IR'],
              ] as [Tab, string][]
            ).map(([t, label]) => (
              <button
                key={t}
                data-testid={`tab-${t}`}
                onClick={() => setTab(t)}
                className={`relative px-4 py-2.5 text-sm transition-colors ${tab === t ? 'text-foreground' : 'text-muted-foreground hover:text-foreground'}`}
              >
                {label}
                {tab === t && <span className="absolute inset-x-3 bottom-0 h-0.5 bg-accent" />}
              </button>
            ))}
            <div className="ml-auto flex gap-2 px-3">
              <button className={btn} disabled={!source} onClick={() => download('main.sl', new Blob([source], { type: 'text/plain' }))} data-testid="button-download-source">.sl</button>
              <button className={btn} disabled={!result?.llvmIr} onClick={() => result && download('main.ll', new Blob([result.llvmIr], { type: 'text/plain' }))} data-testid="button-download-ir">.ll</button>
              <button className={btn} disabled={!result?.wasmBase64} onClick={() => result && download('main.wasm', base64ToBlob(result.wasmBase64))} data-testid="button-download-wasm">.wasm</button>
            </div>
          </div>

          <div className="min-h-0 flex-1 overflow-auto p-4 font-mono text-[13px] leading-6">
            {tab === 'output' && (
              <div className="rise" data-testid="panel-output">
                {phase === 'compiling' && <Skeleton className="h-5 w-1/2" />}
                {compileError && (
                  <div className="rounded border border-destructive/40 bg-destructive/10 p-3 text-destructive" data-testid="text-compile-error">
                    {compileError}
                    <div className="mt-2">
                      <button className={btn} onClick={() => void doCompile(true)}>Retry</button>
                    </div>
                  </div>
                )}
                {result && !result.success && !compileError && (
                  <p className="text-destructive">Compilation failed. See Diagnostics.</p>
                )}
                {result?.success && phase === 'idle' && run.exitCode === null && !run.error && !run.stdout && (
                  <p className="text-accent">Compiled in {result.durationMs} ms with {result.compilerVersion}. Not run.</p>
                )}
                {(run.stdout || phase === 'running') && (
                  <pre className="whitespace-pre-wrap break-words text-foreground" data-testid="text-stdout">{run.stdout}{phase === 'running' && <span className="animate-pulse text-primary">_</span>}</pre>
                )}
                {run.error && (
                  <div className="mt-3 rounded border border-destructive/40 bg-destructive/10 p-3 text-destructive" data-testid="text-run-error">
                    {run.error}
                  </div>
                )}
                {run.escalation && (
                  <section role="status" className="mt-3 rounded border border-primary/40 bg-primary/10 p-3" data-testid="panel-escalation">
                    <h2 className="font-bold text-primary">HALT_AND_ESCALATE</h2>
                    <p className="mt-1 font-sans text-sm text-muted-foreground">Inference stopped safely. This result is available for human or supervisor review; no external arbiter has been contacted.</p>
                    <dl className="mt-3 space-y-2">
                      <div><dt className="text-xs text-muted-foreground">State</dt><dd className="whitespace-pre-wrap break-words">{run.escalation.state}</dd></div>
                      <div><dt className="text-xs text-muted-foreground">Payload</dt><dd className="whitespace-pre-wrap break-words">{run.escalation.payload || '(empty)'}</dd></div>
                    </dl>
                  </section>
                )}
                {run.exitCode !== null && (
                  <div className="mt-4 flex gap-4 border-t border-border pt-3 text-xs text-muted-foreground" data-testid="text-exit">
                    <span>{run.escalation ? 'handoff' : 'exit'} <b className={run.escalation ? 'text-primary' : run.exitCode === 0 ? 'text-accent' : 'text-destructive'}>{run.exitCode}</b></span>
                    {run.durationMs !== null && <span>{run.durationMs} ms run</span>}
                    {result && <span>{result.durationMs} ms compile</span>}
                  </div>
                )}
                {!result && !compileError && phase === 'idle' && (
                  <div className="py-16 text-center text-muted-foreground">
                    <div className="mx-auto mb-4 h-px w-24 bg-gradient-to-r from-transparent via-primary to-transparent" />
                    Nothing has run yet. Press Ctrl+Enter to compile for WebAssembly and run it here.
                  </div>
                )}
              </div>
            )}

            {tab === 'diagnostics' && (
              <div className="rise space-y-2" data-testid="panel-diagnostics">
                {!result && <p className="text-muted-foreground">Compile to see diagnostics.</p>}
                {result && diags.length === 0 && <p className="text-accent">No diagnostics.</p>}
                {diags.map((d, i) => (
                  <div key={i} className={`flex gap-3 rounded border-l-2 bg-card p-3 ${d.severity === 'error' ? 'border-destructive' : d.severity === 'warning' ? 'border-primary' : 'border-accent'}`}>
                    <span className={`w-14 shrink-0 text-xs uppercase ${d.severity === 'error' ? 'text-destructive' : d.severity === 'warning' ? 'text-primary' : 'text-accent'}`}>{d.severity}</span>
                    <span className="flex-1 whitespace-pre-wrap">{d.message}</span>
                    <span className="shrink-0 text-xs text-muted-foreground">{d.line}:{d.column}</span>
                  </div>
                ))}
              </div>
            )}

            {tab === 'ir' && (
              <div className="rise" data-testid="panel-ir">
                {result?.llvmIr ? (
                  <pre className="whitespace-pre text-foreground/85">{result.llvmIr}</pre>
                ) : (
                  <p className="text-muted-foreground">{result ? 'The compiler returned no LLVM IR.' : 'Compile to inspect the generated LLVM IR.'}</p>
                )}
              </div>
            )}
          </div>
        </section>
      </main>
    </Shell>
  );
}
