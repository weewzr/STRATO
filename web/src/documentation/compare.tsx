import { findCase } from '../data/case';
import { hrefDocumentation } from '../route';
import { useStrato } from '../state/store';
import type { DocContribution } from './types';

// Scientific material that left the Compare page, moved with its meaning unchanged.

function CompareSource() {
  const caseId = useStrato((s) => s.caseId);
  const caseFile = findCase(caseId)?.provenance.input_file ?? '';
  return (
    <>
      <p>
        The case file of both runs is <code>{caseFile}</code>. The CSV of the comparison, written by the solver in Rust with its
        provenance header, is under <a href={hrefDocumentation('evidence-downloads')}>Data downloads</a>.
      </p>
    </>
  );
}

export const compare: DocContribution = {
  limitations: [
    {
      id: 'compare-limits',
      title: 'What a comparison shows',
      body: (
        <>
          <p>
            Both runs use the verified solver of Method A: BTCS, so the comparison shows the effect of the inputs, not of new physics; this
            holds until generation, decay and temperature dependence are solved.
          </p>
          <p>
            Both runs start from the same synthetic verification case, not physical data (the case file is given under{' '}
            <a href={hrefDocumentation('compare-source')}>Source of a comparison</a>).
          </p>
        </>
      ),
    },
  ],
  verification: [
    {
      id: 'compare-difference-curve',
      title: 'Difference curve of a comparison',
      body: (
        <p>
          The green difference curve is the series that <code>compare</code> returns: the current minus the baseline release fraction,
          computed in Rust at each output time both runs share; a wrapper test checks it point by point. Runs with different output times
          are not compared, because the difference is never interpolated; such a comparison is rejected.
        </p>
      ),
    },
  ],
  interpretation: [
    {
      id: 'compare-and-validate',
      title: 'Comparison and validation are independent',
      body: (
        <p>
          The comparison and the validation layers are independent: the switches of the comparison change only its figure, and the
          measurements and model curves of the experimental comparison on Explore are not drawn there.
        </p>
      ),
    },
  ],
  implementation: [{ id: 'compare-source', title: 'Source of a comparison', body: <CompareSource /> }],
};
