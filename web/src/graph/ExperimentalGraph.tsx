import { useMemo } from 'react';
import { Plot } from '../plots/Plot';
import { usePlotTheme } from '../plots/theme';
import type { ComparisonFile } from './contract';
import { comparisonFigure } from './figure';
import { toPlotModel } from './plotModel';

/**
 * One experimental-comparison graph: nuclide and experimental temperature in the title line, the
 * curves and observations in the plot, then (outside the plot area) the caption that says why R² is
 * unavailable, where it is, and the full APA citation with the direct publication link.
 */
export function ExperimentalGraph({ file }: { file: ComparisonFile }) {
  const theme = usePlotTheme();
  const model = useMemo(() => toPlotModel(file), [file]);
  const figure = useMemo(() => comparisonFigure(model, theme), [model, theme]);
  const names = model.traces.map((t) => t.name).join(', ');
  const r2 = model.r2Labels.map((l) => l.text).join('; ');
  const label = `Cumulative release fraction against time (${model.timeUnit}) for ${model.title}: ${names}${r2 ? `; ${r2}` : ''}.`;
  return (
    <figure className="figure plotfig expfig" data-testid={`exp-graph-${file.id}`} data-dataset={file.id}>
      <div className="figure__head">
        <h3 className="expfig__title" data-testid={`exp-title-${file.id}`}>
          {model.title}
        </h3>
      </div>
      <Plot figure={figure} testId={`exp-plot-${file.id}`} height={360} label={label} />
      {model.caption ? (
        <p className="expfig__caption" data-testid={`exp-caption-${file.id}`}>
          {model.caption}
        </p>
      ) : null}
      <p className="expfig__source" data-testid={`exp-source-${file.id}`}>
        <span data-testid={`exp-citation-${file.id}`}>{model.source.citation_apa}</span> <span className="muted">({model.source.location})</span>{' '}
        <a className="expfig__link" href={model.source.link} target="_blank" rel="noopener noreferrer" data-testid={`exp-link-${file.id}`}>
          {model.source.link}
        </a>
      </p>
    </figure>
  );
}
