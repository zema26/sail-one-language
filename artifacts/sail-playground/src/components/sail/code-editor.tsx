import { useMemo, useRef, type KeyboardEvent, type ReactElement, type UIEvent } from 'react';

const TOKEN = /(\/\/[^\n]*)|("(?:[^"\\\n]|\\.)*")|(\b\d[\d_]*(?:\.\d+)?\b)|(->|=>|\|>|<-|[+\-*/%=<>!&|^~:;,.?@$]+)|([A-Za-z_][A-Za-z0-9_]*)/g;

function highlight(src: string) {
  const out: ReactElement[] = [];
  let last = 0;
  let k = 0;
  for (const m of src.matchAll(TOKEN)) {
    const i = m.index ?? 0;
    if (i > last) out.push(<span key={k++}>{src.slice(last, i)}</span>);
    const cls = m[1] ? 'tok-c' : m[2] ? 'tok-s' : m[3] ? 'tok-n' : m[4] ? 'tok-o' : 'tok-i';
    out.push(
      <span key={k++} className={cls}>
        {m[0]}
      </span>,
    );
    last = i + m[0].length;
  }
  if (last < src.length) out.push(<span key={k++}>{src.slice(last)}</span>);
  return out;
}

interface Props {
  value: string;
  onChange: (v: string) => void;
  errorLines?: number[];
  warnLines?: number[];
}

export function CodeEditor({ value, onChange, errorLines = [], warnLines = [] }: Props) {
  const gutter = useRef<HTMLDivElement>(null);
  const pre = useRef<HTMLPreElement>(null);
  const lines = value.split('\n').length;
  const hl = useMemo(() => highlight(value), [value]);

  const sync = (e: UIEvent<HTMLTextAreaElement>) => {
    const t = e.currentTarget;
    if (gutter.current) gutter.current.scrollTop = t.scrollTop;
    if (pre.current) {
      pre.current.style.transform = `translate(${-t.scrollLeft}px, ${-t.scrollTop}px)`;
    }
  };

  const onKey = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Tab') {
      e.preventDefault();
      const t = e.currentTarget;
      const s = t.selectionStart;
      onChange(value.slice(0, s) + '  ' + value.slice(t.selectionEnd));
      requestAnimationFrame(() => t.setSelectionRange(s + 2, s + 2));
    }
  };

  return (
    <div className="relative flex h-full min-h-[320px] overflow-hidden font-mono text-[13px] leading-6">
      <div
        ref={gutter}
        aria-hidden
        className="w-12 shrink-0 select-none overflow-hidden border-r border-border bg-card/60 py-3 pr-3 text-right text-muted-foreground/60"
      >
        {Array.from({ length: lines }, (_, i) => {
          const n = i + 1;
          const err = errorLines.includes(n);
          const warn = warnLines.includes(n);
          return (
            <div
              key={n}
              className={err ? 'font-bold text-destructive' : warn ? 'font-bold text-primary' : ''}
            >
              {n}
            </div>
          );
        })}
      </div>
      <div className="relative flex-1 overflow-hidden">
        <pre
          ref={pre}
          aria-hidden
          className="pointer-events-none absolute left-0 top-0 m-0 whitespace-pre p-3 will-change-transform"
          style={{ font: 'inherit', tabSize: 2 }}
        >
          {hl}
          {'\n'}
        </pre>
        <textarea
          data-testid="input-source"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onScroll={sync}
          onKeyDown={onKey}
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
          wrap="off"
          aria-label="Sail source"
          className="absolute inset-0 h-full w-full resize-none overflow-auto whitespace-pre bg-transparent p-3 text-transparent caret-primary outline-none selection:bg-primary/25"
          style={{ font: 'inherit', tabSize: 2 }}
        />
      </div>
    </div>
  );
}
