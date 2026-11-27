// Compare and Validate left the application: what they held is in Scientific Documentation, and each old address
// leads to it. The experimental graphs keep their mount point there (graph_mount.test.tsx).
import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { allItemIds } from '../src/documentation';
import { ExperimentalEvidence } from '../src/documentation/evidence';
import { redirectFromHash } from '../src/route';
import { Documentation } from '../src/sections/Documentation';

describe('the experimental comparison in Scientific Documentation', () => {
  it('keeps the mount point of the experimental graph, and the old Validate address leads to its statement', () => {
    const { container } = render(<ExperimentalEvidence />);
    expect(container.querySelector('[data-mount="experimental-comparison-graph"]')).not.toBeNull();
    const target = redirectFromHash('#/validate') ?? '';
    expect(allItemIds()).toContain(target.replace('#/documentation/', ''));
  });
});

describe('what left Compare and Validate is in Scientific Documentation', () => {
  it('holds the moved statements once, under their anchors', () => {
    render(<Documentation />);
    const text = (id: string) => document.getElementById(`doc-${id}`)?.textContent ?? '';
    expect(text('compare-limits')).toContain('the comparison shows the effect of the inputs, not of new physics');
    expect(text('compare-difference-curve')).toContain('Runs with different output times are not compared');
    expect(text('compare-and-validate')).toContain('independent');
    expect(text('validate-limits')).toContain('A constant-temperature result of Method A: BTCS is never compared with heating-test data');
    expect(text('validate-limits')).toContain('No comparison is shown yet.');
    expect(text('validate-status')).toContain('It is a different question from verification');
    expect(text('validate-status')).toContain('What the experimental comparison will show');
    // R² is the only displayed agreement metric (decisions 69 and 74).
    expect(text('r2-requirements')).toContain('R² is the only displayed agreement metric');
    expect(text('r2-requirements')).toContain('computed in Rust, per release-versus-time curve');
    expect(text('r2-requirements')).toContain('a one-sentence caption below the graph says why');
    expect(text('r2-requirements')).toContain('No registered dataset supports one today');
    expect(text('validate-status')).not.toContain('only displayed agreement metric');
    expect(text('validate-status')).not.toContain('never typed in');
    expect(text('r2-requirements')).toContain('Seventeen ordered checks');
  });
});