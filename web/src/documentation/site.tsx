import type { ReactNode } from 'react';
import { BUILD, REPO_URL, repoFile } from '../build';
import { useStrato } from '../state/store';
import { TableBox } from '../sections/common';
import type { DocContribution } from './types';

// Material that belongs to the site as a whole (no single source page): the rendered derivation
// documents and the technical implementation and build information. The build hash, the solver version
// and the source-code paths of the interface live here and nowhere else.

/** `page`: the page that the documents tool renders from the same file, with the mathematics typeset. */
const DOCUMENTS: { path: string; title: string; page?: string }[] = [
  {
    path: 'docs/mathematics/btcs/00_original_documents.md',
    title: 'Document 0: the original FTCS documents, an account and an algebra check',
    page: 'docs/mathematics/btcs/00_original_documents.html',
  },
  {
    path: 'docs/mathematics/btcs/01_from_ftcs_to_five_layer_btcs.md',
    title: 'Document 1: from the explicit scheme to the five-layer BTCS scheme',
    page: 'docs/mathematics/btcs/01_from_ftcs_to_five_layer_btcs.html',
  },
  {
    path: 'docs/mathematics/btcs/02_final_formulation.md',
    title: 'Document 2: the final BTCS formulation (normative)',
    page: 'docs/mathematics/btcs/02_final_formulation.html',
  },
  {
    path: 'docs/mathematics/btcs/03_rust_pseudocode.md',
    title: 'Document 3: Rust pseudocode',
    page: 'docs/mathematics/btcs/03_rust_pseudocode.html',
  },
  {
    path: 'docs/mathematics/btcs/04_equation_to_code_traceability.md',
    title: 'Document 4: equation to code traceability',
    page: 'docs/mathematics/btcs/04_equation_to_code_traceability.html',
  },
  { path: 'data/reference/STRATO_Parameter_Register.md', title: 'Parameter register', page: 'data/reference/STRATO_Parameter_Register.html' },
];

function DerivationDocuments() {
  return (
    <>
      <ul className="prose-list" data-testid="documentation-derivations">
        {DOCUMENTS.map((d) => (
          <li key={d.path}>
            {d.page ? (
              <>
                <a href={`${import.meta.env.BASE_URL}${d.page}`}>{d.title}</a> (<a href={repoFile(d.path)}>Markdown source</a>)
              </>
            ) : (
              <a href={repoFile(d.path)}>{d.title}</a>
            )}
          </li>
        ))}
      </ul>
      <p>
        <a href={`${import.meta.env.BASE_URL}docs/index.html`}>All documents rendered on this site</a>
      </p>
    </>
  );
}

/** Build stamp of the page and of the solver, from the build and from the WebAssembly wrapper. */
function BuildTable() {
  const solver = useStrato((s) => s.solver);
  const info = solver.build;
  const unavailable = solver.state === 'failed' ? 'solver not available' : 'loading';
  const rows: { item: string; value: ReactNode }[] = [
    {
      item: 'Site build',
      value: (
        <>
          <a href={`${REPO_URL}/commit/${BUILD.commit}`}>
            <code>{BUILD.commit}</code>
          </a>
          {BUILD.dirty ? ' (uncommitted changes)' : ''}
        </>
      ),
    },
    { item: 'STRATO version', value: <code>{BUILD.strato_version}</code> },
    { item: 'Solver version', value: info ? <code>{info.btcs.solver.version}</code> : unavailable },
    { item: 'WebAssembly wrapper', value: info ? <code>{`${info.crate} ${info.wrapper_version}, build ${info.commit.slice(0, 7)}`}</code> : unavailable },
  ];
  return (
    <TableBox label="Build of this site and of its solver">
      <table className="table" data-testid="documentation-build">
        <caption className="visually-hidden">Build of this site and of its solver</caption>
        <thead>
          <tr>
            <th scope="col">Item</th>
            <th scope="col">Value</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.item}>
              <th scope="row">{r.item}</th>
              <td>{r.value}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </TableBox>
  );
}

const tree = (path: string) => repoFile(path).replace('/blob/', '/tree/');

const SOURCE: { part: string; path: string }[] = [
  { part: 'BTCS solver library (Rust)', path: 'src' },
  { part: 'WebAssembly wrapper', path: 'crates/strato-wasm' },
  { part: 'Web demonstration (React and TypeScript)', path: 'web' },
  { part: 'Verification tests and regression references', path: 'tests' },
];

function SourceCode() {
  return (
    <TableBox label="Source code of STRATO">
      <table className="table" data-testid="documentation-source">
        <caption className="visually-hidden">Source code of STRATO</caption>
        <thead>
          <tr>
            <th scope="col">Part</th>
            <th scope="col">Location in the repository</th>
          </tr>
        </thead>
        <tbody>
          {SOURCE.map((s) => (
            <tr key={s.path}>
              <th scope="row">{s.part}</th>
              <td>
                <a href={tree(s.path)}>
                  <code>{s.path}/</code>
                </a>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </TableBox>
  );
}

export const site: DocContribution = {
  derivations: [{ id: 'derivation-documents', title: 'Derivation and register documents', body: <DerivationDocuments /> }],
  implementation: [
    { id: 'build', title: 'Build', body: <BuildTable /> },
    { id: 'source-code', title: 'Source code', body: <SourceCode /> },
  ],
};
