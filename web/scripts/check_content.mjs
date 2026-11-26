// Content check of the built site (owner decisions M3 and M5, prompt/DESIGN_SPECIFICATION.md Section 11).
// Usage: node scripts/check_content.mjs dist
// Fails (exit 1) if the build contains a file under prompt/, a link to the prompt/ folder, the README
// sentence about the internal comparison or an agreement figure of that sentence, or a local
// absolute path. It also renders the pages in a browser and applies the mechanical guards of the
// minimal-text rules (scripts/minimal_text_rules.mjs): no build hash or file path in the main interface,
// short figure captions, a limit on prose before the first control, each registered disclaimer once.
// Prints one PASS or FAIL line per rule.
// Options: --snapshots <file.json> uses page snapshots from a file instead of rendering the build;
//          --no-render skips the rendered guards (static rules only).
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import { evaluateSnapshots, TEMPORARY_EXCEPTIONS } from './minimal_text_rules.mjs';

const args = process.argv.slice(2);
const flagValue = (name) => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
};
const root = args.find((a, i) => !a.startsWith('--') && args[i - 1] !== '--snapshots');
if (!root) {
  console.error('usage: node scripts/check_content.mjs <dist directory> [--snapshots file.json | --no-render]');
  process.exit(2);
}
const noRender = args.includes('--no-render');
const snapshotsFile = flagValue('--snapshots');

function walk(dir, out = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) walk(p, out);
    else out.push(p);
  }
  return out;
}

const files = walk(root);
const textFiles = files.filter((f) => /\.(html|js|css|json|csv|txt|svg|md)$/i.test(f));
// Stored encoded, so this script does not carry the strings it forbids.
const forbidden = ['U05SU0k=', 'd2l0aGluIDIl'].map((b) => Buffer.from(b, 'base64').toString('utf8'));

const rules = [];
const rule = (name, failures) => rules.push({ name, failures });

rule(
  'no file of the prompt/ folder in the build',
  files.map((f) => relative(root, f).split(sep).join('/')).filter((p) => p === 'prompt' || p.startsWith('prompt/')),
);

const linkFailures = [];
const textFailures = [];
const pathFailures = [];
for (const f of textFiles) {
  const text = readFileSync(f, 'utf8');
  const rel = relative(root, f).split(sep).join('/');
  for (const m of text.matchAll(/(?:href|src)\s*=\s*["']([^"']*)["']/gi)) {
    if (/(^|\/)prompt\//.test(m[1] ?? '')) linkFailures.push(`${rel}: ${m[1]}`);
  }
  if (/github\.com\/weewzr\/STRATO\/(?:blob|tree)\/[^"'\s]+\/prompt\//.test(text)) linkFailures.push(`${rel}: repository link into prompt/`);
  for (const s of forbidden) if (text.includes(s)) textFailures.push(`${rel}: contains a forbidden string`);
  if (/[A-Za-z]:\\Users\\|\/home\/[a-z]|\/Users\/[A-Za-z]/.test(text)) pathFailures.push(`${rel}: local absolute path`);
}
rule('no test fixture and no test harness in the build', [
  ...files.map((f) => relative(root, f).split(sep).join('/')).filter((p) => /harness|fixture/i.test(p)),
  ...textFiles.filter((f) => readFileSync(f, 'utf8').includes('FIXTURE')).map((f) => `${relative(root, f)}: contains FIXTURE`),
]);
rule('no link to the prompt/ folder', linkFailures);
rule('no internal-comparison text or agreement figure', textFailures);
rule('no local absolute path', pathFailures);
rule('the build has an index.html and a build.json', ['index.html', 'build.json'].filter((n) => !files.some((f) => relative(root, f) === n)));

let notes = [];
if (!noRender) {
  let snapshots = null;
  try {
    if (snapshotsFile) snapshots = JSON.parse(readFileSync(snapshotsFile, 'utf8'));
    else snapshots = await (await import('./render_pages.mjs')).collectSnapshots(root);
  } catch (e) {
    rule('the pages render for the minimal-text guards', [e instanceof Error ? e.message : String(e)]);
  }
  if (snapshots) {
    const evaluated = evaluateSnapshots(snapshots, TEMPORARY_EXCEPTIONS);
    for (const r of evaluated.results) {
      const label = r.excepted.length > 0 ? `${r.name} [temporary exceptions: ${r.excepted.join(', ')}]` : r.name;
      rule(label, r.failures);
    }
    notes = evaluated.notes;
  }
}

let failed = 0;
for (const r of rules) {
  if (r.failures.length === 0) console.log(`PASS ${r.name}`);
  else {
    failed++;
    console.log(`FAIL ${r.name}`);
    for (const x of r.failures) console.log(`     ${x}`);
  }
}
for (const n of notes) console.log(`NOTE ${n}`);
console.log(`${textFiles.length} text files of ${files.length} checked`);
process.exit(failed ? 1 : 0);
