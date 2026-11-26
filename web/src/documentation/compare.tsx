import { findCase } from '../data/case';
import { hrefFor } from '../route';
import { useStrato } from '../state/store';
import type { DocContribution } from './types';

// Scientific material that left the Compare page, moved with its meaning unchanged.

function CompareSource() {
  const caseId = useStrato((s) => s.caseId);
  const caseFile = findCase(caseId)?.provenance.input_file ?? '';
  return (
    <>
      <p>
        Both runs of a comparison start from <code>{caseFile}</code>. The CSV of the comparison, written by the solver in Rust with its
        provenance header, is under <a href={hrefFor('data')}>Data</a>.
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
          <p>Both runs start from the same synthetic verification case, not physical data.</p>
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
          computed in Rust at each output time both runs share. Runs with different output times are not compared, because the difference is
          never interpolated.
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
          measurements and predictions of the Validate section are not drawn there.
        </p>
      ),
    },
  ],
  implementation: [{ id: 'compare-source', title: 'Source of a comparison', body: <CompareSource /> }],
};
