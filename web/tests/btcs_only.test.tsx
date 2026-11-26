// The page runs the BTCS solver and nothing else (coordinator decisions 29 and 52; directive section
// 16: no silent fallback to the retired finite-volume solver). Three tests keep that so:
//  1. no source file of the page, the tests or the browser tests names an export of the retired
//     solver, so no call to one can be written;
//  2. the worker, which is the only code that calls the module, calls the BTCS functions only: every
//     request kind is sent to a module whose retired exports fail the test when they are called;
//  3. a solver that cannot be loaded is shown as a failure, never as numbers.
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { act, render, screen } from '@testing-library/react';
import { beforeAll, describe, expect, it, vi } from 'vitest';
import { RunProblemNote, SolverStatus } from '../src/panels/SimResults';
import type { RequestKind, WorkerRequest } from '../src/solver/protocol';
import { useStrato } from '../src/state/store';

const RETIRED = ['run_v1', 'compare_v1', 'analytical_release', 'validate_config'];

const calls = vi.hoisted(() => ({ retired: [] as string[], btcs: [] as string[] }));

vi.mock('../src/solver/pkg/strato_wasm.js', () => {
  const retired = (name: string) => () => {
    calls.retired.push(name);
    throw new Error(`finite-volume export ${name} called`);
  };
  const btcs = (name: string, reply: unknown) => () => {
    calls.btcs.push(name);
    return JSON.stringify(reply);
  };
  return {
    default: () => Promise.resolve(),
    abi_version: () => 3,
    build_info: btcs('build_info', { btcs: { capabilities: ['diffusion_btcs_layered'] } }),
    validate: btcs('validate', { schema: 2, ok: true, errors: [], warnings: [] }),
    run: btcs('run', { schema: 2, status: 'ok' }),
    compare: btcs('compare', { schema: 2, status: 'ok' }),
    export_run: btcs('export_run', { schema: 2, status: 'ok', text: '' }),
    run_v1: retired('run_v1'),
    compare_v1: retired('compare_v1'),
    analytical_release: retired('analytical_release'),
    validate_config: retired('validate_config'),
  };
});

function sourceFiles(dir: string, out: string[] = []): string[] {
  for (const name of readdirSync(dir)) {
    if (name === 'node_modules' || name === 'pkg' || name === 'dist' || name === 'generated') continue;
    const path = join(dir, name);
    if (statSync(path).isDirectory()) sourceFiles(path, out);
    else if (/\.(ts|tsx|mjs|js)$/.test(name)) out.push(path);
  }
  return out;
}

describe('no call to a finite-volume export can be written', () => {
  it('names no retired export in the sources of the page or in any test', () => {
    const root = process.cwd();
    const files = ['src', 'tests', 'e2e'].flatMap((d) => sourceFiles(join(root, d))).filter((f) => !f.includes('btcs_only.test'));
    expect(files.length).toBeGreaterThan(20);
    const offenders: string[] = [];
    for (const f of files) {
      const text = readFileSync(f, 'utf8');
      for (const name of RETIRED) if (text.includes(name)) offenders.push(`${f}: ${name}`);
    }
    expect(offenders).toEqual([]);
  });
});

describe('the worker calls the BTCS functions only', () => {
  const posted: unknown[] = [];

  beforeAll(async () => {
    // jsdom's window is the worker scope here: the worker assigns onmessage and posts its replies to it.
    (self as unknown as { postMessage: (m: unknown) => void }).postMessage = (m) => {
      posted.push(m);
    };
    await import('../src/solver/worker');
    await new Promise((r) => setTimeout(r, 20));
  });

  it('answers every kind of request through a BTCS function and never reaches a retired one', async () => {
    const kinds: RequestKind[] = ['run', 'validate', 'compare', 'export_run', 'build_info'];
    const scope = self as unknown as { onmessage: ((ev: MessageEvent<WorkerRequest>) => void) | null };
    kinds.forEach((kind, i) => scope.onmessage?.({ data: { id: i + 1, kind, payload: '{}' } } as MessageEvent<WorkerRequest>));
    await new Promise((r) => setTimeout(r, 20));
    expect(calls.retired).toEqual([]);
    expect(calls.btcs).toEqual(expect.arrayContaining(['build_info', 'run', 'validate', 'compare', 'export_run']));
    const replies = posted.filter((m): m is { kind: string; ok: boolean } => typeof m === 'object' && m !== null && (m as { kind?: string }).kind === 'response');
    expect(replies).toHaveLength(kinds.length);
    expect(replies.every((r) => r.ok)).toBe(true);
  });
});

describe('a solver that cannot be loaded is a failure, not numbers', () => {
  it('shows the failure and no result, and says that there is no other source of numbers', () => {
    act(() => {
      useStrato.setState({ solver: { state: 'failed', build: null, error: 'CompileError: bad magic number', restarts: 0 }, run: null });
    });
    render(
      <>
        <SolverStatus />
        <RunProblemNote />
      </>,
    );
    expect(screen.getByTestId('solver-status-line').dataset.state).toBe('failed');
    expect(screen.getByTestId('solver-failed').textContent).toContain('could not be loaded');
    expect(screen.getByTestId('solver-failed').textContent).toContain('CompileError');
    expect(screen.queryByTestId('release-value')).toBeNull();
    expect(useStrato.getState().run).toBeNull();
  });
});
