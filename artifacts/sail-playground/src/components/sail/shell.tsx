import type { ReactNode } from 'react';
import { Link, useLocation } from 'wouter';
import { useGetCompilerStatus } from '@workspace/api-client-react';

function Mark() {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" aria-hidden>
      <path d="M11 2 L11 17 L3 17 Z" fill="hsl(38 92% 62%)" />
      <path d="M13 6 L13 17 L21 17 Z" fill="hsl(172 55% 52%)" />
      <path d="M2 20 H22" stroke="hsl(40 25% 90%)" strokeWidth="1.5" strokeLinecap="round" />
    </svg>
  );
}

export function StatusPill() {
  const { data, isLoading, isError, refetch, isFetching } = useGetCompilerStatus({
    query: { queryKey: ['/api/sail/status'], refetchInterval: (q) => (q.state.data?.ready ? false : 5000) },
  });
  let tone = 'bg-muted-foreground';
  let label = 'checking backend';
  if (isError) {
    tone = 'bg-destructive';
    label = 'backend unreachable';
  } else if (data) {
    tone = data.ready ? 'bg-accent' : 'bg-primary animate-pulse';
    label = data.ready ? `${data.backend} ${data.version}` : 'backend warming up';
    if (data.ready && data.features.some((feature) => feature.includes('multidimensional tensors'))) {
      label += ' · tensors enabled';
    }
  }
  return (
    <button
      type="button"
      data-testid="status-backend"
      onClick={() => refetch()}
      title="Click to re-check"
      className="flex items-center gap-2 rounded-full border border-border bg-card px-3 py-1 text-left font-mono text-[11px] text-muted-foreground transition-colors hover:border-primary/50 hover:text-foreground"
    >
      <span className={`h-1.5 w-1.5 rounded-full ${tone} ${isLoading || isFetching ? 'animate-pulse' : ''}`} />
      {label}
    </button>
  );
}

export function Shell({ children }: { children: ReactNode }) {
  const [loc] = useLocation();
  const nav = [
    { href: '/', label: 'Playground' },
    { href: '/reference', label: 'Reference' },
  ];
  return (
    <div className="flex min-h-[100dvh] flex-col">
      <header className="sticky top-0 z-20 flex flex-wrap items-center gap-x-4 gap-y-1 sm:flex-nowrap border-b border-border bg-background/85 px-4 py-2.5 backdrop-blur">
        <Link href="/" className="group flex items-center gap-2.5" data-testid="link-home">
          <span className="transition-transform group-hover:-rotate-6">
            <Mark />
          </span>
          <span className="text-lg font-extrabold tracking-tight">
            sail<span className="text-primary">.</span>
            <span className="font-normal text-muted-foreground">compiler</span>
          </span>
        </Link>
        <nav className="order-last -mx-2 flex w-full gap-1 overflow-x-auto sm:order-none sm:mx-0 sm:ml-4 sm:w-auto">
          {nav.map((n) => {
            const on = loc === n.href;
            return (
              <Link
                key={n.href}
                href={n.href}
                data-testid={`link-${n.label.toLowerCase()}`}
                className={`relative rounded px-3 py-1.5 text-sm transition-colors ${on ? 'text-foreground' : 'text-muted-foreground hover:text-foreground'}`}
              >
                {n.label}
                {on && <span className="absolute inset-x-3 -bottom-[7px] sm:-bottom-[11px] h-0.5 bg-primary" />}
              </Link>
            );
          })}
        </nav>
        <div className="ml-auto">
          <StatusPill />
        </div>
      </header>
      {children}
    </div>
  );
}
