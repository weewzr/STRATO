import '../design/explore.css';
import { ExploreControls, ExploreEquation } from '../equation/ExploreEquation';
import { NumericalSections } from '../equation/NumericalSections';
import { ExploreGraphs, useGraphChoices } from '../graph/ExploreGraphs';
import { useHeadingFocus } from './common';

/** The phone layout's bottom-sheet entry belonged to the manual solver, which Explore no longer mounts; the props are kept so the caller needs no change. */
export interface ExploreProps {
  compact?: boolean;
  onOpenControls?: () => void;
}

/**
 * The Explore page in four areas (final directive of 27 Nov 2026, decisions 77 and 78): upper left MODEL
 * EQUATION and, beneath it, OPERATION (two independent groups of buttons); upper centre and right the
 * GOVERNING EQUATION with its legend; lower left NUMERICAL INTERPRETATION (six collapsed sections); lower
 * right INTERACTIVE GRAPHS. One shared selection (state/store.ts) drives them and is written to the address.
 * The numbers come from the wrapper (run_stage, analytical) and from the generated results files; the
 * page computes none. The manual solver lives in advanced/AdvancedSolver.tsx.
 */
export function Explore(props: ExploreProps) {
  void props;
  const h1 = useHeadingFocus();
  const choices = useGraphChoices();
  const applicability = choices.result?.reply.analytical ?? null;
  return (
    <article className="section explore explore--final" data-section="explore">
      <h1 ref={h1} tabIndex={-1} className="visually-hidden">
        Explore
      </h1>
      <div className="xgrid">
        <div className="xgrid__controls">
          <ExploreControls />
        </div>
        <div className="xgrid__equation">
          <ExploreEquation />
        </div>
        <div className="xgrid__numerical">
          <NumericalSections applicability={applicability} />
        </div>
        <div className="xgrid__graphs">
          <ExploreGraphs choices={choices} />
        </div>
      </div>
    </article>
  );
}
