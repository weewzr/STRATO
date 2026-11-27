// The experimental-comparison graph (directive of 26 Nov 2026 on R² and source traceability): the data
// contract, the pure mapping from a file to a plot model, the figure, and the component with its
// caption, citation and link below the graph. The fixtures are invented values marked FIXTURE and are
// mounted only here and in the browser harness; with no comparison file in the build the pages of
// The comparison evidence and the experimental graphs in Scientific Documentation show nothing of this.
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { ExperimentalComparisons } from '../src/graph/Comparisons';
import { comparisonErrors, isComparisonFile, type ComparisonFile } from '../src/graph/contract';
import { COMPARISON_FILE_CHECKS, COMPARISON_FILES } from '../src/graph/files';
import { comparisonFigure } from '../src/graph/figure';
import { COLOURS, toPlotModel } from '../src/graph/plotModel';
import type { GraphDiv } from '../src/plots/plotly';
import { readPlotTheme } from '../src/plots/theme';
import { ComparisonEvidence } from '../src/documentation/ComparisonEvidence';
import { ExperimentalEvidence } from '../src/documentation/evidence';
import both from './fixtures/fixture_r2_both_methods.json';
import absent from './fixtures/fixture_r2_method_b_absent.json';
import negative from './fixtures/fixture_r2_negative.json';
import bounds from './fixtures/fixture_r2_upper_bounds.json';

const THEME = readPlotTheme();
const BOTH = both as unknown as ComparisonFile;
const NEG = negative as unknown as ComparisonFile;
const BOUNDS = bounds as unknown as ComparisonFile;
const ABSENT = absent as unknown as ComparisonFile;
const FIXTURES = [BOTH, NEG, BOUNDS, ABSENT];

const clone = <T,>(x: T): T => structuredClone(x);

describe('fixtures', () => {
  it('are marked FIXTURE in their content and satisfy the contract', () => {
    const dir = join(process.cwd(), 'tests', 'fixtures');
    const names = readdirSync(dir).filter((n) => n.endsWith('.json'));
    expect(names).toHaveLength(4);
    for (const n of names) {
      const text = readFileSync(join(dir, n), 'utf8');
      expect(text, n).toContain('FIXTURE');
      expect(comparisonErrors(JSON.parse(text)), n).toEqual([]);
    }
    for (const f of FIXTURES) expect(f.id.startsWith('FIXTURE-')).toBe(true);
  });
});

describe('data contract', () => {
  it('accepts the fixtures and rejects a value that is not a file', () => {
    for (const f of FIXTURES) expect(isComparisonFile(f)).toBe(true);
    expect(isComparisonFile(null)).toBe(false);
    expect(isComparisonFile({})).toBe(false);
    expect(isComparisonFile([BOTH])).toBe(false);
  });

  it('rejects another metric, and any metric or verdict word in a label or caption', () => {
    const rmse = clone(BOTH);
    (rmse.r2[0] as unknown as { metric: string }).metric = 'rmse';
    expect(comparisonErrors(rmse).join()).toContain('metric');
    for (const word of ['RMSE', 'MAE', 'NRMSE', 'PASS', 'FAIL']) {
      const f = clone(BOUNDS);
      f.r2[0]!.caption_plain = `The ${word} is high`;
      expect(isComparisonFile(f), word).toBe(false);
    }
    const lab = clone(BOTH);
    lab.r2[0]!.label_plain = 'NRMSE = 0.1';
    expect(isComparisonFile(lab)).toBe(false);
  });

  it('needs a label where R² has a value, a caption where it has none, a real address and equal-length arrays', () => {
    const noLabel = clone(BOTH);
    noLabel.r2[0]!.label_plain = '';
    expect(comparisonErrors(noLabel).join()).toContain('label_plain');
    const noCaption = clone(BOUNDS);
    noCaption.r2[0]!.caption_plain = '';
    expect(comparisonErrors(noCaption).join()).toContain('caption_plain');
    const badLink = clone(BOTH);
    badLink.source.link = 'doi:10.0000/none';
    expect(comparisonErrors(badLink).join()).toContain('link');
    const ragged = clone(BOTH);
    ragged.method_a.f = ragged.method_a.f.slice(1);
    expect(comparisonErrors(ragged).join()).toContain('equal length');
    const orphan = clone(NEG);
    orphan.r2.push({ ...clone(NEG.r2[0]!), method: 'B' });
    expect(comparisonErrors(orphan).join()).toContain('Method B does not apply');
  });
});

describe('plot model', () => {
  it('takes the arrays of the file themselves, element by element and by identity', () => {
    for (const f of FIXTURES) {
      const m = toPlotModel(f);
      const a = m.traces.find((t) => t.role === 'method_a')!;
      expect(a.t).toBe(f.method_a.t);
      expect(a.f).toBe(f.method_a.f);
      const b = m.traces.find((t) => t.role === 'method_b');
      if (f.method_b) {
        expect(b?.t).toBe(f.method_b.t);
        expect(b?.f).toBe(f.method_b.f);
      } else expect(b).toBeUndefined();
      const obs = m.traces.filter((t) => t.role === 'measurement' || t.role === 'upper_bound');
      expect(obs).toHaveLength(f.observations.length);
      f.observations.forEach((o, i) => {
        expect(obs[i]?.t).toBe(o.t);
        expect(obs[i]?.f).toBe(o.f);
      });
    }
  });

  it('puts the nuclide and the experimental temperature in the title line and the unit of the file on the time axis', () => {
    expect(toPlotModel(BOTH).title).toBe('Ag-110m, 1500 °C');
    expect(toPlotModel(BOTH).timeUnit).toBe('h');
  });

  it('gives one label per applicable method that holds a value, verbatim, and the caption of the others', () => {
    expect(toPlotModel(BOTH).r2Labels).toEqual([
      { method: 'A', text: BOTH.r2[0]!.label_plain, colour: '#2563EB' },
      { method: 'B', text: BOTH.r2[1]!.label_plain, colour: '#DC2626' },
    ]);
    expect(toPlotModel(BOTH).caption).toBeNull();
    // A negative value is shown as it is.
    expect(toPlotModel(NEG).r2Labels.map((l) => l.text)).toEqual(['R² (Method A: BTCS) = -0.42']);
    // No label where there is no value; one caption, not two copies of it.
    expect(toPlotModel(BOUNDS).r2Labels).toEqual([]);
    expect(toPlotModel(BOUNDS).caption).toBe(BOUNDS.r2[0]!.caption_plain);
    // Method B does not apply: no curve, no label, no caption about it.
    expect(toPlotModel(ABSENT).traces.map((t) => t.role)).toEqual(['method_a', 'measurement']);
    expect(toPlotModel(ABSENT).caption).toBeNull();
  });
});

describe('figure', () => {
  it('draws Method A blue and Method B red as solid lines of equal width, observations as black markers', () => {
    const fig = comparisonFigure(toPlotModel(BOTH), THEME);
    const [a, b, o] = fig.data;
    expect(a?.line).toEqual({ color: '#2563EB', width: 2, dash: 'solid' });
    expect(b?.line).toEqual({ color: '#DC2626', width: 2, dash: 'solid' });
    expect(a?.mode).toBe('lines');
    expect(o?.mode).toBe('markers');
    expect(o?.marker?.color).toBe('#111111');
    expect(o?.marker?.symbol).toBe('circle');
    expect(COLOURS).toEqual({ A: '#2563EB', B: '#DC2626', observations: '#111111' });
    expect(a?.x).toBe(BOTH.method_a.t);
    expect(a?.y).toBe(BOTH.method_a.f);
  });

  it('draws upper bounds as downward triangles', () => {
    const o = comparisonFigure(toPlotModel(BOUNDS), THEME).data.find((t) => t.meta?.id.startsWith('exp-upper-bound'));
    expect(o?.marker?.symbol).toBe('triangle-down');
    expect(o?.marker?.color).toBe('#111111');
  });

  it('names the axes with their units and draws R² only where the record has a value, in the method colour', () => {
    const fig = comparisonFigure(toPlotModel(BOTH), THEME);
    const title = (a: unknown) => (a as { title: { text: string } }).title.text;
    expect(title(fig.layout.xaxis)).toBe('Time (h)');
    expect(title(fig.layout.yaxis)).toBe('Cumulative release fraction (dimensionless)');
    const notes = fig.layout.annotations as { text: string; font: { color: string } }[];
    expect(notes.map((n) => [n.text, n.font.color])).toEqual([
      [BOTH.r2[0]!.label_plain, '#2563EB'],
      [BOTH.r2[1]!.label_plain, '#DC2626'],
    ]);
    expect(comparisonFigure(toPlotModel(BOUNDS), THEME).layout.annotations).toEqual([]);
    expect(JSON.stringify(comparisonFigure(toPlotModel(BOUNDS), THEME).layout)).not.toMatch(/R²|pending|unavailable/i);
  });
});

describe('component', () => {
  it('mounts all four fixtures, with the R² label rule, caption, citation and link below each graph', async () => {
    render(<ExperimentalComparisons files={FIXTURES} />);
    for (const f of FIXTURES) {
      const graph = screen.getByTestId(`exp-plot-${f.id}-graph`) as GraphDiv;
      await waitFor(() => expect(graph.data).toBeDefined());
      const labels = ((graph.layout?.annotations ?? []) as { text: string }[]).map((a) => a.text);
      expect(labels).toEqual(f.r2.filter((r) => r.value !== null).map((r) => r.label_plain));
      const plot = screen.getByTestId(`exp-plot-${f.id}`);
      const fig = screen.getByTestId(`exp-graph-${f.id}`);
      const caption = screen.queryByTestId(`exp-caption-${f.id}`);
      const source = screen.getByTestId(`exp-source-${f.id}`);
      // Nothing about R² inside the plot area; the caption (when there is one) follows the whole plot, then the citation.
      expect(plot.textContent ?? '').not.toMatch(/R²|pending|unavailable|upper bound/i);
      expect(fig.contains(source)).toBe(true);
      expect(source.textContent).toContain(f.source.citation_apa);
      expect(source.textContent).toContain(f.source.location);
      const follows = (a: Node, b: Node) => Boolean(a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING);
      expect(follows(plot, source)).toBe(true);
      if (caption) {
        expect(plot.contains(caption)).toBe(false);
        expect(follows(plot, caption)).toBe(true);
        expect(follows(caption, source)).toBe(true);
        expect(caption.textContent).toBe(f.r2.find((r) => r.value === null)!.caption_plain);
      } else expect(f.r2.every((r) => r.value !== null)).toBe(true);
    }
    expect(screen.queryByTestId('exp-caption-FIXTURE-both')).toBeNull();
    expect(screen.queryByTestId('exp-caption-FIXTURE-negative')).toBeNull();
    expect(screen.getByTestId('exp-caption-FIXTURE-bounds').textContent).toContain('upper bounds');
    expect(document.body.textContent ?? '').not.toMatch(/\b(RMSE|MAE|NRMSE|PASS|FAIL)\b/i);
  });

  it('writes the link as a real anchor with the address of the file, reachable with the keyboard', async () => {
    render(<ExperimentalComparisons files={[BOUNDS]} />);
    const a = screen.getByTestId('exp-link-FIXTURE-bounds') as HTMLAnchorElement;
    expect(a.tagName).toBe('A');
    expect(a.getAttribute('href')).toBe(BOUNDS.source.link);
    expect(a.textContent).toBe(BOUNDS.source.link);
    const user = userEvent.setup();
    await user.tab();
    expect(document.activeElement).toBe(a);
  });

  it('shows no heading and nothing at all for an empty list', () => {
    const { container } = render(<ExperimentalComparisons files={[]} />);
    expect(container.innerHTML).toBe('');
  });
});

describe('mounting in Scientific Documentation (the comparison and the experimental graphs)', () => {
  it('shows nothing of the graph while the build holds no comparison file', () => {
    expect(COMPARISON_FILES).toHaveLength(0);
    for (const page of [<ComparisonEvidence key="c" />, <ExperimentalEvidence key="v" />]) {
      const { unmount } = render(page);
      expect(screen.queryByTestId('exp-comparisons')).toBeNull();
      unmount();
    }
  });

  it('holds no fixture among the comparison files of the build, and every file there satisfies the contract', () => {
    for (const c of COMPARISON_FILE_CHECKS) expect(c.errors, c.path).toEqual([]);
    expect(COMPARISON_FILES.some((f) => f.id.startsWith('FIXTURE'))).toBe(false);
  });
});
