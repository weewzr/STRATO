// Compare and Validate after the minimal-text clean-up: the pages hold controls, graphs and one status
// line; the scientific statements that left them are in Scientific Documentation and the links reach them.
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { allItemIds } from '../src/documentation';
import { documentationAnchorFromHash } from '../src/route';
import { Documentation } from '../src/sections/Documentation';
import { Validate } from '../src/sections/Validate';

describe('Validate page', () => {
  it('shows the disabled switch, one status line and the mount point of the experimental graph', () => {
    const { container } = render(<Validate />);
    const sw = screen.getByTestId('toggle-validation');
    expect(sw.getAttribute('aria-disabled')).toBe('true');
    expect(sw.getAttribute('aria-checked')).toBe('false');
    expect(document.getElementById(sw.getAttribute('aria-describedby') ?? '')?.textContent).toContain(
      'model predictions need generation, decay and temperature-dependent diffusion',
    );
    expect(container.querySelector('[data-mount="experimental-comparison-graph"]')).not.toBeNull();
    const link = screen.getByRole('link', { name: 'Why' });
    expect(allItemIds()).toContain(documentationAnchorFromHash(link.getAttribute('href') ?? ''));
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
    expect(text('validate-status')).toContain('It is a different question from verification');
    expect(text('validate-status')).toContain('What the Validate page will show');
    // The R² rule (decisions 69 and 74) replaced the sentence on error metrics: R² is the only displayed agreement metric.
    expect(text('validate-status')).toContain('R² is the only displayed agreement metric');
    expect(text('validate-status')).toContain('computed in Rust per release-versus-time curve');
    expect(text('validate-status')).toContain('a caption below the graph says why');
    expect(text('validate-status')).not.toContain('never typed in');
  });
});
