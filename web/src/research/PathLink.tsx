import { createContext, Fragment, useContext, type ReactNode } from 'react';
import { repoFile } from '../build';

// A repository path in a catalogue text (a test file, a data table, a document) is shown as a link titled
// by what it holds, in words, not as a file name or a path (minimal-text rules: no file path or source file
// name in the main interface). The link address and its title keep the full path, so nothing is lost; the
// file names themselves are listed in Scientific Documentation.

const ROOTS = 'src|crates|tests|docs|data|inputs|web|prompt|examples|scripts';
const EXT = 'rs|toml|md|csv|json|ya?ml|tsx?';
// An optional "#A3-05" row or anchor id follows a file.
const PATH_SOURCE = String.raw`(?:\b(?:${ROOTS})\/[\w.\/-]*\w\/?|\b[\w-]+\.(?:${EXT})\b)(?:#[\w-]+)?`;
const WHOLE = new RegExp(`^${PATH_SOURCE}$`);

/** True for a string that is, as a whole, a repository path or a source file name (with an optional #id). */
export function isRepoPath(text: string): boolean {
  return WHOLE.test(text);
}

/** What a file or folder holds, in words, keyed by its path; any other path falls back to its own name in words. */
const TITLES: Record<string, string> = {
  'data/nuclear/decay.csv': 'decay constants register',
  'data/nuclear/fission_yields.csv': 'fission yields register',
  'data/diffusivity/arrhenius.csv': 'diffusivity register',
  'data/geometry/geometry.csv': 'geometry register',
  'data/temperature/temperature_histories.csv': 'temperature histories register',
  'data/experimental/release_measurements.csv': 'release measurements register',
  'data/research/mechanisms.csv': 'mechanism catalogue',
  'data/research/species.csv': 'species catalogue',
  'data/research/gallery.csv': 'gallery catalogue',
  'data/research/references.csv': 'reference catalogue',
  'tests/btcs_verification.rs': 'BTCS verification tests',
  'tests/btcs_regression.rs': 'BTCS regression tests',
  'tests/btcs_properties.rs': 'BTCS property tests',
  'tests/golden/btcs': 'BTCS regression reference files',
  'docs/mathematics/btcs': 'BTCS derivation documents',
  'docs/mathematics/STRATO_Phase2_Derivations.md': 'Phase 2 derivations',
  'docs/mathematics/STRATO_Phase2_Pseudocode.md': 'Phase 2 pseudocode',
  'inputs/synthetic/synthetic_five_layer.toml': 'synthetic five-layer case input',
};

/** A file or folder name as words: "wrapper_native_synthetic.json" becomes "wrapper native synthetic". */
function humanise(part: string): string {
  return part.replace(/\.\w+$/, '').replace(/[_-]+/g, ' ');
}

/** The words that name a path; a "#A3-05" row id is kept after them. */
export function pathLabel(path: string): string {
  const [file, anchor] = path.split('#');
  const clean = (file ?? path).replace(/\/$/, '');
  const parts = clean.split('/').filter(Boolean);
  const isDir = !/\.\w+$/.test(clean);
  const fallback = isDir && parts.length > 1 ? parts.slice(-2).map(humanise).join(' ') : humanise(parts[parts.length - 1] ?? clean);
  const title = TITLES[clean] ?? fallback;
  return anchor ? `${title} ${anchor}` : title;
}

/**
 * How a path is written: "words" in the main interface; "files" in Scientific Documentation, the one page
 * where file names and paths belong (the same links, named by the path).
 */
export const PathStyle = createContext<'words' | 'files'>('words');

export function PathLink({ path }: { path: string }) {
  const style = useContext(PathStyle);
  const [file] = path.split('#');
  const target = file ?? path;
  const dir = target.endsWith('/') || !/\.\w+$/.test(target);
  const href = dir ? repoFile(target.replace(/\/$/, '')).replace('/blob/', '/tree/') : repoFile(path);
  return (
    <a href={href} title={path} className="path-link" rel="noopener noreferrer">
      {style === 'files' ? path : pathLabel(path)}
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
