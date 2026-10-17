import { useGetCompilerStatus, useGetLanguageReference } from '@workspace/api-client-react';
import { Shell } from '@/components/sail/shell';
import { Markdown } from '@/components/sail/markdown';
import { Skeleton } from '@/components/ui/skeleton';

function Retry({ msg, onRetry }: { msg: string; onRetry: () => void }) {
  return (
    <div className="rounded-md border border-destructive/40 bg-destructive/10 p-4 text-sm">
      <p className="text-destructive">{msg}</p>
      <button onClick={onRetry} className="mt-2 rounded border border-border px-3 py-1 text-foreground hover:border-primary" data-testid="button-retry">
        Retry
      </button>
    </div>
  );
}

export default function Reference() {
  const ref = useGetLanguageReference();
  const st = useGetCompilerStatus();
  return (
    <Shell>
      <main className="mx-auto grid w-full max-w-6xl gap-10 px-4 py-10 lg:grid-cols-[1fr_320px]">
        <article className="rise min-w-0" data-testid="section-reference">
          {ref.isLoading && (
            <div className="space-y-3">
              <Skeleton className="h-10 w-2/3" />
              <Skeleton className="h-4 w-full" />
              <Skeleton className="h-4 w-5/6" />
              <Skeleton className="h-32 w-full" />
            </div>
          )}
          {ref.isError && <Retry msg="Could not load the language reference." onRetry={() => ref.refetch()} />}
          {ref.data && (
            <>
              <section className="mb-6 rounded-md border border-accent/30 bg-accent/5 p-4" data-testid="clairii-reference-summary">
                <h2 className="text-lg font-semibold text-accent">ClairII language support</h2>
                <p className="mt-2 text-sm text-muted-foreground">Shared module tensors, outer products, calls, string append, typed pipeline destinations, and explicit tensor policies. The example demonstrates mechanics, not a trained model or universal safety verifier.</p>
                <a href="#clairii-language-support" className="mt-3 inline-block text-sm text-accent underline underline-offset-4">Read the new syntax, policy definitions, and limitations</a>
              </section>
              <section className="mb-6 rounded-md border border-accent/30 bg-accent/5 p-4" data-testid="tensor-math-reference-summary">
                <h2 className="text-lg font-semibold text-accent">Tensor signatures and math</h2>
                <p className="mt-2 text-sm text-muted-foreground">Shaped function parameters and returns, matrix/vector products, scalar arithmetic, slices, exponentials, and reductions are available now.</p>
                <a href="#tensor-signatures-and-math" className="mt-3 inline-block text-sm text-accent underline underline-offset-4">Read tensor syntax and the runnable neural pipeline</a>
              </section>
              <section className="mb-6 rounded-md border border-primary/30 bg-primary/5 p-4" data-testid="neuro-reference-summary">
                <h2 className="text-lg font-semibold text-primary">Neuro-Symbolic Branching</h2>
                <p className="mt-2 text-sm text-muted-foreground">Available now: infer| / fall|, inclusive confidence thresholds (~), and controlled escalation (^&gt;). The full supplied document and implementation details follow below.</p>
                <a href="#neuro-symbolic-branching" className="mt-3 inline-block text-sm text-primary underline underline-offset-4">Read branching syntax and the original document</a>
              </section>
              <section className="mb-8 rounded-md border border-accent/30 bg-accent/5 p-4" data-testid="tensor-reference-summary">
                <h2 className="text-lg font-semibold text-accent">Multidimensional arrays &amp; broadcast fills</h2>
                <p className="mt-2 text-sm text-muted-foreground">Ranked tensors are available in the compiler. Existing one-dimensional vectors still work.</p>
                <pre className="mt-3 overflow-x-auto rounded bg-background p-3 font-mono text-xs leading-6">{'float weights|2, 64, 64|\\n0.0 > weights|*|\\n1.5 > weights|3, 7|\\nweights|3, 7| > out'.replaceAll('\\n', '\n')}</pre>
                <a href="#implementation-notes" className="mt-3 inline-block text-sm text-accent underline underline-offset-4">Read implementation notes and tensor usage</a>
              </section>
              <Markdown source={ref.data.markdown} />
              {ref.data.notes && (
                <aside id="implementation-notes" className="mt-8 scroll-mt-24 rounded-md border-l-2 border-primary bg-card p-4 text-sm leading-6 text-muted-foreground" data-testid="text-notes">
                  <h2 className="mb-4 text-xl font-bold text-foreground">Implementation notes</h2>
                  <Markdown source={ref.data.notes} />
                </aside>
              )}
            </>
          )}
        </article>
        <aside className="space-y-6 lg:sticky lg:top-20 lg:self-start" data-testid="section-coverage">
          <div className="text-xs font-bold uppercase tracking-[0.2em] text-muted-foreground">Implementation coverage</div>
          {st.isLoading && <Skeleton className="h-40 w-full" />}
          {st.isError && <Retry msg="Could not load compiler status." onRetry={() => st.refetch()} />}
          {st.data && (
            <>
              <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 rounded-md border border-border bg-card p-4 font-mono text-xs">
                <dt className="text-muted-foreground">version</dt>
                <dd>{st.data.version}</dd>
                <dt className="text-muted-foreground">backend</dt>
                <dd>{st.data.backend}</dd>
                <dt className="text-muted-foreground">target</dt>
                <dd>{st.data.target}</dd>
                <dt className="text-muted-foreground">ready</dt>
                <dd className={st.data.ready ? 'text-accent' : 'text-primary'}>{st.data.ready ? 'yes' : 'not yet'}</dd>
              </dl>
              <div>
                <h3 className="mb-2 text-sm font-semibold text-accent">Implemented</h3>
                <ul className="space-y-1.5 text-sm">
                  {st.data.features.map((f) => (
                    <li key={f} className="flex gap-2"><span className="mt-2 h-1 w-3 shrink-0 bg-accent" />{f}</li>
                  ))}
                  {st.data.features.length === 0 && <li className="text-muted-foreground">Nothing reported.</li>}
                </ul>
              </div>
              <div>
                <h3 className="mb-2 text-sm font-semibold text-primary">Limitations</h3>
                <ul className="space-y-1.5 text-sm text-muted-foreground">
                  {st.data.limitations.map((f) => (
                    <li key={f} className="flex gap-2"><span className="mt-2 h-1 w-3 shrink-0 bg-primary" />{f}</li>
                  ))}
                  {st.data.limitations.length === 0 && <li>None reported.</li>}
                </ul>
              </div>
            </>
          )}
        </aside>
      </main>
    </Shell>
  );
}
