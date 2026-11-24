/// <reference types="vitest/config" />
import { execSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import react from '@vitejs/plugin-react';
import { defineConfig, type Plugin } from 'vite';

const repoRoot = fileURLToPath(new URL('..', import.meta.url));

function git(args: string): string {
  try {
    return execSync(`git ${args}`, { cwd: repoRoot, encoding: 'utf8' }).trim();
  } catch {
    return '';
  }
}

/** Build stamp: commit of the tree the site was built from, and the STRATO crate version. */
function buildStamp() {
  const fromCi = process.env.GITHUB_SHA ?? '';
  const commit = (fromCi || git('rev-parse HEAD')).slice(0, 7) || 'unknown';
  const dirty = fromCi ? false : git('status --porcelain') !== '';
  const cargo = readFileSync(new URL('../Cargo.toml', import.meta.url), 'utf8');
  const version = /^version\s*=\s*"([^"]+)"/m.exec(cargo)?.[1] ?? 'unknown';
  return { schema: 1, commit, dirty, strato_version: version };
}

const stamp = buildStamp();

/** Writes build.json next to index.html and puts the commit into a meta tag. */
function stampPlugin(): Plugin {
  return {
    name: 'strato-build-stamp',
    transformIndexHtml(html) {
      return html.replace('__STRATO_COMMIT__', stamp.commit);
    },
    generateBundle() {
      this.emitFile({
        type: 'asset',
        fileName: 'build.json',
        source: JSON.stringify(stamp, null, 2) + '\n',
      });
    },
  };
}

export default defineConfig({
  base: '/STRATO/',
  plugins: [react(), stampPlugin()],
  define: {
    __STRATO_BUILD__: JSON.stringify(stamp),
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    sourcemap: false,
  },
  test: {
    environment: 'jsdom',
    include: ['tests/**/*.test.{ts,tsx}'],
    setupFiles: ['tests/setup.ts'],
  },
});
