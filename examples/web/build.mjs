#!/usr/bin/env node
// Build ggez examples for wasm32-unknown-unknown and serve them using Vite from
// examples/web/public/
//
// Usage:
//   node build.mjs                      # build every example
//   node build.mjs 04_snake             # build a single example
//   node build.mjs 04_snake bunnymark   # build several
//   node build.mjs --release            # build in release mode (much faster runtime)
//   node build.mjs --no-bindgen-install # skip auto-installing wasm-bindgen-cli

import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync, mkdirSync, cpSync, existsSync, readdirSync, rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO = resolve(HERE, '..', '..');
const PUBLIC = join(HERE, 'public');
const EXAMPLES_OUT = join(PUBLIC, 'examples');
const RESOURCES_OUT = join(PUBLIC, 'resources');

const argv = process.argv.slice(2);
const flags = new Set(argv.filter(a => a.startsWith('--')));
const targets = argv.filter(a => !a.startsWith('--'));
const release = flags.has('--release');
const skipInstall = flags.has('--no-bindgen-install');

// Discover examples from Cargo.toml

const cargoToml = readFileSync(join(REPO, 'Cargo.toml'), 'utf8');
// Examples that need a feature flag are declared as `[[example]]` blocks.
const featureBlocks = [...cargoToml.matchAll(/\[\[example\]\]\s*\nname\s*=\s*"([^"]+)"\s*\nrequired-features\s*=\s*\[([^\]]+)\]/g)];
const requiredFeatures = new Map(featureBlocks.map(m => [m[1], m[2].split(',').map(s => s.trim().replace(/"/g, ''))]));
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

// Make sure wasm-bindgen-cli matches the locked wasm-bindgen version

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

function tryRun(cmd, args, opts = {}) {
  return spawnSync(cmd, args, { stdio: 'pipe', cwd: REPO, ...opts });
}

const bindgenProbe = tryRun('wasm-bindgen', ['--version']);
const installedBindgen = bindgenProbe.status === 0
  ? bindgenProbe.stdout.toString().trim().split(/\s+/).pop()
  : null;

if (installedBindgen !== requiredBindgen) {
  if (skipInstall) {
    console.error(`wasm-bindgen ${requiredBindgen} required (have ${installedBindgen ?? 'none'}); rerun without --no-bindgen-install`);
    process.exit(1);
  }
  console.log(`installing wasm-bindgen-cli ${requiredBindgen} (matching Cargo.lock)…`);
  run('cargo', ['install', '--locked', '--version', requiredBindgen, 'wasm-bindgen-cli']);
}

// Group examples by feature set so each cargo invocation builds many

const groups = new Map(); // featureKey -> { features: string[], names: string[] }
for (const name of wanted) {
  const feats = requiredFeatures.get(name) ?? [];
  const key = feats.slice().sort().join(',');
  if (!groups.has(key)) groups.set(key, { features: feats, names: [] });
  groups.get(key).names.push(name);
}

const profileDir = release ? 'release' : 'debug';
const profileArgs = release ? ['--release'] : [];

mkdirSync(EXAMPLES_OUT, { recursive: true });

for (const { features, names } of groups.values()) {
  const args = ['build', '--target', 'wasm32-unknown-unknown', ...profileArgs];
  if (features.length) args.push('--features', features.join(','));
  for (const n of names) args.push('--example', n);
  console.log(`\n→ cargo ${args.join(' ')}`);
  run('cargo', args);
}

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

// Copy /resources into public/ so fetch('resources/foo') works

rmSync(RESOURCES_OUT, { recursive: true, force: true });
cpSync(join(REPO, 'resources'), RESOURCES_OUT, { recursive: true });

// Write a manifest the gallery can read

function extractDescription(name) {
  const file = join(REPO, 'examples', `${name}.rs`);
  const lines = readFileSync(file, 'utf8').split('\n');
  const docs = [];
  for (const line of lines) {
    const m = line.match(/^\/\/!\s?(.*)$/);
    if (!m) {
      if (docs.length) break; // contiguous block only
      continue;
    }
    docs.push(m[1].trim());
  }
  return docs.join(' ').trim();
}

const manifest = {
  generatedAt: new Date().toISOString(),
  profile: profileDir,
  examples: wanted.map(name => ({
    name,
    features: requiredFeatures.get(name) ?? [],
    description: extractDescription(name),
  })),
};
writeFileSync(join(PUBLIC, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n');

console.log(`\n✓ built ${wanted.length} example(s) into ${PUBLIC}`);
console.log(`  next: \`npm run serve\` (or \`npm run dev\` to rebuild + serve)`);
