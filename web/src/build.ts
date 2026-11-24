// Build stamp injected by vite.config.ts (also written to build.json and the strato-build meta tag).
declare const __STRATO_BUILD__: { schema: number; commit: string; dirty: boolean; strato_version: string };

export const BUILD = __STRATO_BUILD__;

export const REPO_URL = 'https://github.com/weewzr/STRATO';

/** Link to a file in the repository at the commit this site was built from. */
export function repoFile(path: string): string {
  const ref = BUILD.commit && BUILD.commit !== 'unknown' && !BUILD.dirty ? BUILD.commit : 'main';
  return `${REPO_URL}/blob/${ref}/${path}`;
}
