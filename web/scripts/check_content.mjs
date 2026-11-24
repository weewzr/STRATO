// Content check of the built site (owner decisions M3 and M5, prompt/DESIGN_SPECIFICATION.md Section 11).
// Usage: node scripts/check_content.mjs dist
// Fails (exit 1) if the build contains a file under prompt/, a link to the prompt/ folder, the README
// sentence about the internal comparison or an agreement figure of that sentence, or a local
// absolute path. Prints one PASS or FAIL line per rule. Reads files only.
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';

const root = process.argv[2];
if (!root) {
  console.error('usage: node scripts/check_content.mjs <dist directory>');
  process.exit(2);
}

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
rule('no link to the prompt/ folder', linkFailures);
rule('no internal-comparison text or agreement figure', textFailures);
rule('no local absolute path', pathFailures);
rule('the build has an index.html and a build.json', ['index.html', 'build.json'].filter((n) => !files.some((f) => relative(root, f) === n)));

let failed = 0;
for (const r of rules) {
  if (r.failures.length === 0) console.log(`PASS ${r.name}`);
  else {
    failed++;
    console.log(`FAIL ${r.name}`);
    for (const x of r.failures) console.log(`     ${x}`);
  }
}
console.log(`${textFiles.length} text files of ${files.length} checked`);
process.exit(failed ? 1 : 0);
