// Runs scripts/web/check_catalogue.py (prompt/TECHNICAL_ARCHITECTURE.md Section 5.2) before every
// build, so a failing catalogue check stops the build. Python standard library only. The interpreter
// is the first of python3 and python that reports Python 3 (on Windows, python3 may be a store stub).
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../..', import.meta.url));
const args = ['-I', '-X', 'utf8', 'scripts/web/check_catalogue.py'];
for (const exe of ['python3', 'python']) {
  const probe = spawnSync(exe, ['-c', 'import sys; print(sys.version_info[0])'], { encoding: 'utf8' });
  if (probe.error || probe.status !== 0 || probe.stdout.trim() !== '3') continue;
  const r = spawnSync(exe, args, { cwd: root, stdio: 'inherit' });
  process.exit(r.status ?? 1);
}
console.error('check_catalogue: no Python interpreter found (python3 or python)');
process.exit(1);
