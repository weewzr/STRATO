import type { ReactNode } from 'react';
import { repoFile } from '../build';
import { Blocks } from '../research/Blocks';
import { useCatalogue, type Block } from '../research/catalogue';
import { GALLERY_ENTRY, introBlocks, ROADMAP_ENTRY } from '../research/entries';
import { PathLink, PathStyle } from '../research/PathLink';
import { STATUS_INFO, STATUSES } from '../research/status';
import { GalleryPlannedDetail } from '../research/parts';
import { StatusChip } from '../research/StatusChip';
import { TableBox } from '../sections/common';
import type { DocContribution } from './types';

// Scientific material that left the Research pages, moved with its meaning unchanged: the definitions of
// the research statuses, the statements of what the catalogue does not do, the rules of the gallery and
// the sources of the catalogue.

function StatusScale() {
  return (
    <>
      <TableBox label="Research statuses and their definitions">
        <table className="table" data-testid="research-status-scale">
          <caption className="visually-hidden">Research statuses and their definitions</caption>
          <thead>
            <tr>
              <th scope="col">Status</th>
              <th scope="col">Definition</th>
            </tr>
          </thead>
          <tbody>
            {STATUSES.map((s) => (
              <tr key={s}>
                <td>
                  <StatusChip status={s} legend />
                </td>
                <td>{STATUS_INFO[s].definition}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </TableBox>
      <p>
        Shown as a word and a shape, never by colour alone. A research status describes a mechanism; the labels on numbers and figures (for
        example COMPUTED v2.0 or PRECOMPUTED) describe a result.
      </p>
      <p>
        Each mechanism shows its research status on a five-level scale, assigned from the evidence in the repository and read from the research
        catalogue files.
      </p>
    </>
  );
}

/** Scientific Documentation is the page where file names and paths belong: the research links are named by their path here. */
function Files({ children }: { children: ReactNode }) {
  return <PathStyle.Provider value="files">{children}</PathStyle.Provider>;
}

function GalleryRules() {
  const cat = useCatalogue();
  return (
    <Files>
      <div className="prose" data-testid="research-gallery-rules">
        <Blocks blocks={introBlocks(cat.entries[GALLERY_ENTRY])} catalogue={cat} />
      </div>
    </Files>
  );
}

function GalleryPlannedCalculations() {
  const cat = useCatalogue();
  const items = cat.gallery.filter((g) => g.kind === 'placeholder');
  return (
    <Files>
    <div className="prose" data-testid="research-gallery-planned">
      <p>
        This calculation has not been performed. It is planned for a future phase of the project. (This applies to each of the {items.length}{' '}
        planned calculations below; no picture exists.)
      </p>
      {items.map((it) => (
        <GalleryPlannedDetail key={it.item_id} item={it} catalogue={cat} />
      ))}
    </div>
    </Files>
  );
}

/**
 * The roadmap introduction is the text of the roadmap page; here it stands on another page, so the words
 * that point at "this page" and "below" name the roadmap instead. Nothing else changes.
 */
function onRoadmap(blocks: Block[]): Block[] {
  return JSON.parse(JSON.stringify(blocks), (key: string, value: unknown) =>
    key === 'v' && typeof value === 'string'
      ? value.replace(/This page lists/g, 'The brick roadmap lists').replace(/on this page/g, 'on the roadmap').replace(/The brick status below/g, 'The brick status on the roadmap')
      : value,
  ) as Block[];
}

function RoadmapIntro() {
  const cat = useCatalogue();
  return (
    <Files>
      <div className="prose" data-testid="research-roadmap-intro">
        <Blocks blocks={onRoadmap(introBlocks(cat.entries[ROADMAP_ENTRY]))} catalogue={cat} />
      </div>
    </Files>
  );
}

function CatalogueSources() {
  const cat = useCatalogue();
  const files = Object.keys(cat.entries).sort();
  return (
    <>
      <ul className="prose-list">
        <li>
          Entry texts: <a href={repoFile('docs/research').replace('/blob/', '/tree/')}>docs/research/</a>, one file per entry:{' '}
          <span data-testid="research-entry-files">
            <Files>
              {files.map((f, i) => (
                <span key={f}>
                  {i > 0 ? ', ' : ''}
                  <PathLink path={f} />
                </span>
              ))}
            </Files>
          </span>
          .
        </li>
        <li>
          Statuses and rows: <a href={repoFile('data/research/mechanisms.csv')}>data/research/mechanisms.csv</a>.
        </li>
        <li>
          References: <a href={repoFile('data/research/references.csv')}>data/research/references.csv</a>. It lists every source the catalogue
          cites, with the date on which its link was opened. A link that has not been opened says so.
        </li>
        <li>
          Gallery items: <a href={repoFile('data/research/gallery.csv')}>data/research/gallery.csv</a>.
        </li>
      </ul>
      <p>
        The catalogue file was written by {cat.generator} from commit <code>{cat.commit.slice(0, 7)}</code>. In the catalogue pages a file
        named in a text (a test, a data table, a document) is shown as a link named after the file; the link address is the file in the
        repository.
      </p>
    </>
  );
}

export const research: DocContribution = {
  limitations: [
    {
      id: 'research-limits',
      title: 'What the research catalogue is',
      body: (
        <>
          <p>
            Mechanisms of fission-product transport in a TRISO particle, organised as research: what STRATO solves, what has been derived but
            not implemented, and what is only proposed, with its evidence and what is missing.
          </p>
          <p>Nothing in the research catalogue is simulated.</p>
          <p>Each row&apos;s live-simulation switch is shown disabled until the solver implements and verifies it.</p>
          <p>
            On the research entry pages: nothing on this page is a STRATO result, and every equation on it is a research concept that STRATO
            does not solve. STRATO does not simulate silver, caesium, europium, iodine, krypton, palladium, ruthenium, strontium or xenon
            today.
          </p>
          <p>
            No mechanism is switched on for a species automatically: a species is simulated only when a row of its own is implemented and
            verified.
          </p>
        </>
      ),
    },
  ],
  // The definitions and the texts of the research catalogue are part of its provenance, not model limitations.
  provenance: [
    { id: 'research-status-scale', title: 'Research status scale', body: <StatusScale /> },
    { id: 'research-roadmap', title: 'Brick roadmap: introduction', body: <RoadmapIntro /> },
    { id: 'research-gallery', title: 'Materials Studio gallery: rules for the planned calculations', body: <GalleryRules /> },
    { id: 'research-gallery-planned', title: 'Materials Studio gallery: the planned calculations', body: <GalleryPlannedCalculations /> },
    { id: 'research-sources', title: 'Sources of the research catalogue', body: <CatalogueSources /> },
    {
      id: 'research-species-records',
      title: 'Species records',
      body: (
        <p>
          Half-lives and decay constants are those of <code>data/nuclear/decay.csv</code>; a nuclide outside that register is marked, not filled
          in.
        </p>
      ),
    },
  ],
};
