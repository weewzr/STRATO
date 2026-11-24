import { hrefFor } from '../route';
import { Placeholder } from '../panels/Placeholder';
import { SectionHeading } from './common';

/**
 * Research section (prompt/DESIGN_SPECIFICATION.md Section 7). It renders the research catalogue
 * from web phase 5.5, read only from the generated catalogue JSON; until then it shows a
 * placeholder and no catalogue content.
 */
export function Research() {
  return (
    <article className="section section--doc" data-section="research">
      <SectionHeading lede="Mechanisms that STRATO does not solve today, organised as research: what is proposed, what evidence exists and what is missing. Nothing in this section is simulated.">
        Research catalogue
      </SectionHeading>
      <Placeholder phase="5.5" title="Research catalogue" testId="research-placeholder">
        <p>
          This section will show the transport regimes, one entry per species, the pending mechanism entries with their candidate equations
          as research concepts, the Materials Studio gallery and the brick roadmap. Every entry and every research status will be read from
          the catalogue files in the repository, and every live-simulation switch in it will stay disabled.
        </p>
        <p>
          Until then, the mechanisms that STRATO does solve are described on <a href={hrefFor('explore')}>Explore</a> and{' '}
          <a href={hrefFor('about')}>About</a>.
        </p>
      </Placeholder>
    </article>
  );
}
