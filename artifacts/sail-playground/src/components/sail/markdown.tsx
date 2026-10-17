import type { ReactNode } from 'react';

function inline(t: string): ReactNode[] {
  return t.split(/(`[^`]+`|\*\*[^*]+\*\*)/g).map((p, i) => {
    if (p.startsWith('`') && p.endsWith('`') && p.length > 1)
      return (
        <code key={i} className="rounded bg-secondary px-1.5 py-0.5 font-mono text-[0.85em] text-primary">
          {p.slice(1, -1)}
        </code>
      );
    if (p.startsWith('**') && p.endsWith('**') && p.length > 4)
      return <strong key={i}>{p.slice(2, -2)}</strong>;
    return p;
  });
}

export function Markdown({ source }: { source: string }) {
  const lines = source.split('\n');
  const out: ReactNode[] = [];
  let i = 0;
  let k = 0;
  while (i < lines.length) {
    const l = lines[i];
    if (l.startsWith('```')) {
      const buf: string[] = [];
      i++;
      while (i < lines.length && !lines[i].startsWith('```')) buf.push(lines[i++]);
      i++;
      out.push(
        <pre key={k++} className="my-4 overflow-x-auto rounded-md border border-border bg-card p-4 font-mono text-[13px] leading-6 text-foreground/90">
          {buf.join('\n')}
        </pre>,
      );
    } else if (/^#{1,4}\s/.test(l)) {
      const lvl = l.match(/^#+/)![0].length;
      const txt = l.replace(/^#+\s*/, '');
      const cls =
        lvl === 1
          ? 'mt-2 mb-4 text-4xl font-extrabold tracking-tight'
          : lvl === 2
            ? 'mt-10 mb-3 border-b border-border pb-2 text-2xl font-bold'
            : 'mt-6 mb-2 text-lg font-semibold text-primary';
      out.push(
        <div key={k++} id={txt.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '')} role="heading" aria-level={lvl} className={`scroll-mt-36 ${cls}`}>
          {inline(txt)}
        </div>,
      );
      i++;
    } else if (/^\s*[-*]\s/.test(l)) {
      const items: string[] = [];
      while (i < lines.length && /^\s*[-*]\s/.test(lines[i])) items.push(lines[i++].replace(/^\s*[-*]\s/, ''));
      out.push(
        <ul key={k++} className="my-3 space-y-1.5 pl-5">
          {items.map((t, j) => (
            <li key={j} className="list-disc marker:text-accent">
              {inline(t)}
            </li>
          ))}
        </ul>,
      );
    } else if (l.trim() === '') {
      i++;
    } else {
      const buf: string[] = [];
      while (i < lines.length && lines[i].trim() !== '' && !/^(```|#{1,4}\s|\s*[-*]\s)/.test(lines[i])) buf.push(lines[i++]);
      out.push(
        <p key={k++} className="my-3 leading-7 text-foreground/85">
          {inline(buf.join(' '))}
        </p>,
      );
    }
  }
  return <div>{out}</div>;
}
