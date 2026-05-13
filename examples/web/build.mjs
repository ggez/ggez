#!/usr/bin/env node
// Build ggez examples for wasm32-unknown-unknown and serve them using Vite from
// examples/web/public/
//
// Usage:
//   node build.mjs                      # build every example
//   node build.mjs 04_snake             # build a single example
//   node build.mjs 04_snake bunnymark   # build several
//   node build.mjs --release            # build in release mode (much faster runtime)

import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync, mkdirSync, cpSync, existsSync, readdirSync, rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildStoredZip } from './src/zip.js';

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO = resolve(HERE, '..', '..');
const PUBLIC = join(HERE, 'public');
const EXAMPLES_OUT = join(PUBLIC, 'examples');
const RESOURCES_OUT = join(PUBLIC, 'resources');

const argv = process.argv.slice(2);
const flags = new Set(argv.filter(a => a.startsWith('--')));
const targets = argv.filter(a => !a.startsWith('--'));
const release = flags.has('--release');

const allExamples = readdirSync(join(REPO, 'examples'))
  .filter(f => f.endsWith('.rs'))
  .map(f => f.replace(/\.rs$/, ''))
  .sort();

const wanted = targets.length ? targets : allExamples;
for (const name of wanted) {
  if (!allExamples.includes(name)) {
    console.error(`unknown example: ${name}`);
    process.exit(1);
  }
}

// Check that wasm-bindgen-cli matches the locked wasm-bindgen version.

const lock = readFileSync(join(REPO, 'Cargo.lock'), 'utf8');
const lockMatch = lock.match(/name = "wasm-bindgen"\nversion = "([^"]+)"/);
if (!lockMatch) {
  console.error('could not find wasm-bindgen version in Cargo.lock');
  process.exit(1);
}
const requiredBindgen = lockMatch[1];

function run(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, { stdio: 'inherit', cwd: REPO, ...opts });
  if (r.status !== 0) process.exit(r.status ?? 1);
  return r;
}

const bindgenProbe = spawnSync('wasm-bindgen', ['--version'], { stdio: 'pipe', cwd: REPO });
const installedBindgen = bindgenProbe.status === 0
  ? bindgenProbe.stdout.toString().trim().split(/\s+/).pop()
  : null;

if (installedBindgen !== requiredBindgen) {
  console.error(`wasm-bindgen ${requiredBindgen} required (have ${installedBindgen ?? 'none'}).`);
  console.error(`install with: cargo install --locked --version ${requiredBindgen} wasm-bindgen-cli`);
  process.exit(1);
}

const profileDir = release ? 'release' : 'debug';
const profileArgs = release ? ['--release'] : [];

mkdirSync(EXAMPLES_OUT, { recursive: true });

const cargoArgs = [
  'build', '--target', 'wasm32-unknown-unknown', ...profileArgs,
  '--features', '3d',
  ...wanted.flatMap(n => ['--example', n]),
];
console.log(`\n→ cargo ${cargoArgs.join(' ')}`);
run('cargo', cargoArgs);

// Run wasm-bindgen on each produced .wasm

for (const name of wanted) {
  const wasmIn = join(REPO, 'target', 'wasm32-unknown-unknown', profileDir, 'examples', `${name}.wasm`);
  if (!existsSync(wasmIn)) {
    console.error(`expected ${wasmIn} but it is missing — did cargo build fail?`);
    process.exit(1);
  }
  const outDir = join(EXAMPLES_OUT, name);
  rmSync(outDir, { recursive: true, force: true });
  mkdirSync(outDir, { recursive: true });
  console.log(`→ wasm-bindgen ${name}`);
  run('wasm-bindgen', [
    '--target', 'web',
    '--no-typescript',
    '--out-dir', outDir,
    '--out-name', name,
    wasmIn,
  ]);
}

// Copy /resources into public/ so fetch('resources/foo') works for the async path, and
// bundle them into resources.zip so the runner can populate VFS for Filesystem::open

rmSync(RESOURCES_OUT, { recursive: true, force: true });
cpSync(join(REPO, 'resources'), RESOURCES_OUT, { recursive: true });

function walkFiles(root, prefix = '') {
  const out = [];
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const sub = prefix ? `${prefix}/${entry.name}` : entry.name;
    const abs = join(root, entry.name);
    if (entry.isDirectory()) {
      out.push(...walkFiles(abs, sub));
    } else if (entry.isFile()) {
      out.push({ path: sub, data: readFileSync(abs) });
    }
  }
  return out;
}
writeFileSync(join(PUBLIC, 'resources.zip'), buildStoredZip(walkFiles(join(REPO, 'resources'))));

// Write a manifest the gallery can read

writeFileSync(join(PUBLIC, 'manifest.json'), JSON.stringify(wanted, null, 2) + '\n');

console.log(`\n✓ built ${wanted.length} example(s) into ${PUBLIC}`);
console.log(`  next: \`npm run serve\` (or \`npm run dev\` to rebuild + serve)`);
