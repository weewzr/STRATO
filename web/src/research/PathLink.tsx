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

/** The words that name a path: the file name with its extension; a folder by its last part with a slash. */
export function pathLabel(path: string): string {
  const parts = path.split('/').filter(Boolean);
  const last = parts[parts.length - 1] ?? path;
  const isDir = path.endsWith('/') || !/\.\w+$/.test(last);
  return isDir ? `${last}/` : last;
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
