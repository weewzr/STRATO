import { expect, test } from './fixtures';

// The CI passes the expected commit (EXPECT_COMMIT); locally the check is skipped when it is unset.
const expectCommit = process.env.EXPECT_COMMIT ?? '';

for (const reducedMotion of ['no-preference', 'reduce'] as const) {
  test(`self-test passes with reduced motion ${reducedMotion}`, async ({ page, problems }, info) => {
    void problems;
    await page.emulateMedia({ reducedMotion });
    const q = expectCommit ? `?selftest=1&expect=${encodeURIComponent(expectCommit)}` : '?selftest=1';
    await page.goto(`./${q}`);
    await expect(page.locator('body')).toHaveAttribute('data-selftest', /^(pass|fail)$/, { timeout: 40_000 });
    const verdict = JSON.parse((await page.locator('#selftest-result').textContent()) ?? '{}') as {
      result: string;
      checks: { id: string; status: string; detail: string }[];
    };
    await info.attach(`selftest-${reducedMotion}.json`, { body: JSON.stringify(verdict, null, 2), contentType: 'application/json' });
    const failed = verdict.checks.filter((c) => c.status === 'fail');
    expect(failed, JSON.stringify(failed)).toEqual([]);
    expect(verdict.result).toBe('pass');
    // Web phase 3: the solver checks run and pass (they were not_run before the wasm module existed).
    for (const id of ['wasm_loads', 'native_parity']) {
      expect(verdict.checks.find((c) => c.id === id)?.status, id).toBe('pass');
    }
    if (expectCommit) expect(verdict.checks.find((c) => c.id === 'expected_commit')?.status).toBe('pass');
    if (reducedMotion === 'reduce') expect(verdict.checks.find((c) => c.id === 'reduced_motion')?.status).toBe('pass');
  });
}
