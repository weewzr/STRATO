// The comparison files of this build: web/src/generated/comparison/*.json, written by the Rust
// pipeline. Today the folder does not exist, so the list is empty and Compare and Validate show what
// they showed before. A file that does not satisfy the contract is left out here and reported by the
// test "every comparison file of the build satisfies the contract", which fails the build checks.
import { comparisonErrors, isComparisonFile, type ComparisonFile } from './contract';

const modules = import.meta.glob<unknown>('../generated/comparison/*.json', { eager: true, import: 'default' });

/** Every file of the folder with the reasons it fails the contract (empty when it satisfies it). */
export const COMPARISON_FILE_CHECKS: { path: string; errors: string[] }[] = Object.entries(modules).map(([path, x]) => ({ path, errors: comparisonErrors(x) }));

export const COMPARISON_FILES: readonly ComparisonFile[] = Object.values(modules).filter(isComparisonFile);
