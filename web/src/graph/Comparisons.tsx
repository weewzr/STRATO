import type { ComparisonFile } from './contract';
import { ExperimentalGraph } from './ExperimentalGraph';
import { COMPARISON_FILES } from './files';

/**
 * The experimental comparisons of the build, mounted by Compare and Validate. Behind a data check:
 * with no comparison file in the build it renders nothing, so the pages are as they were. Tests pass
 * `files` through the harness; the pages pass nothing.
 */
export function ExperimentalComparisons({ files = COMPARISON_FILES }: { files?: readonly ComparisonFile[] }) {
  if (files.length === 0) return null;
  return (
    <section aria-labelledby="exp-comparisons-heading" data-testid="exp-comparisons">
      <h2 id="exp-comparisons-heading">Comparison with published experiments</h2>
      {files.map((f) => (
        <ExperimentalGraph key={f.id} file={f} />
      ))}
    </section>
  );
}
