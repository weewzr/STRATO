/// <reference types="vitest/config" />
import { execSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
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

/**
 * Serves the files of the Materials Studio gallery folder (docs/research/gallery/, listed in
 * data/research/gallery.csv; prompt/TECHNICAL_ARCHITECTURE.md Section 5.1) at research/gallery/ of
 * the site: copied into the build, and served by the development server. Today the folder does not
 * exist (no screenshot has been supplied), so nothing is copied.
 */
function galleryPlugin(): Plugin {
  const dir = join(repoRoot, 'docs', 'research', 'gallery');
  const list = (d: string, out: string[] = []): string[] => {
    if (!existsSync(d)) return out;
    for (const name of readdirSync(d)) {
      const p = join(d, name);
      if (statSync(p).isDirectory()) list(p, out);
      else out.push(p);
    }
    return out;
  };
  return {
    name: 'strato-research-gallery',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const m = /^\/STRATO\/research\/gallery\/([^?#]+)/.exec(req.url ?? '');
        const file = m ? join(dir, decodeURIComponent(m[1] ?? '')) : '';
        if (!m || !file.startsWith(dir) || !existsSync(file)) return next();
        res.end(readFileSync(file));
      });
    },
    generateBundle() {
      for (const p of list(dir)) {
        this.emitFile({ type: 'asset', fileName: `research/gallery/${relative(dir, p).split(sep).join('/')}`, source: readFileSync(p) });
      }
    },
  };
}

export default defineConfig({
  base: '/STRATO/',
  plugins: [react(), stampPlugin(), galleryPlugin()],
  define: {
    __STRATO_BUILD__: JSON.stringify(stamp),
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    sourcemap: false,
    // KaTeX (web phase 2) in its own chunk, so the application chunk stays small and cacheable.
    rolldownOptions: {
      output: {
        codeSplitting: {
          groups: [{ name: 'katex', test: /node_modules[\\/]katex/ }],
        },
      },
    },
  },
  test: {
    environment: 'jsdom',
    include: ['tests/**/*.test.{ts,tsx}'],
    setupFiles: ['tests/setup.ts'],
  },
});
