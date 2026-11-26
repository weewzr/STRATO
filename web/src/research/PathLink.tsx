import { Fragment, type ReactNode } from 'react';
import { repoFile } from '../build';

// A repository path in a catalogue text (a test file, a data table, a document) is shown as a short link
// named after the file, not as a path (minimal-text rules: no file path in the main interface). The link
// address and its title keep the full path, so nothing is lost.

const ROOTS = 'src|crates|tests|docs|data|inputs|web|prompt|examples|scripts';
const EXT = 'rs|toml|md|csv|json|ya?ml|tsx?';
const PATH_SOURCE = String.raw`\b(?:${ROOTS})\/[\w.\/-]*\w\/?|\b[\w-]+\.(?:${EXT})\b`;
const WHOLE = new RegExp(`^(?:${PATH_SOURCE})$`);

/** True for a string that is, as a whole, a repository path or a source file name. */
export function isRepoPath(text: string): boolean {
  return WHOLE.test(text);
}

/** The words that name a path: its last part without the extension, a directory with its parent. */
export function pathLabel(path: string): string {
  if (path === 'data/nuclear/decay.csv') return 'nuclear-data register';
  const parts = path.split('/').filter(Boolean);
  const isDir = path.endsWith('/') || !/\.\w+$/.test(parts[parts.length - 1] ?? '');
  const last = (parts[parts.length - 1] ?? path).replace(/\.\w+$/, '');
  const named = isDir && parts.length > 1 ? `${parts[parts.length - 2]} ${last}` : last;
  return named.replace(/_/g, ' ');
}

export function PathLink({ path }: { path: string }) {
  const dir = path.endsWith('/') || !/\.\w+$/.test(path);
  const href = dir ? repoFile(path.replace(/\/$/, '')).replace('/blob/', '/tree/') : repoFile(path);
  return (
    <a href={href} title={path} className="path-link" rel="noopener noreferrer">
      {pathLabel(path)}
    </a>
  );
}

/** A note: plain text in which every repository path is shown as a PathLink. */
export function NoteText({ text }: { text: string }) {
  const re = new RegExp(PATH_SOURCE, 'g');
  const out: ReactNode[] = [];
  let last = 0;
  for (const m of text.matchAll(re)) {
    const at = m.index ?? 0;
    out.push(<Fragment key={`t${at}`}>{text.slice(last, at)}</Fragment>);
    out.push(<PathLink key={`p${at}`} path={m[0]} />);
    last = at + m[0].length;
  }
  out.push(<Fragment key="end">{text.slice(last)}</Fragment>);
  return <>{out}</>;
}
