#!/usr/bin/env node
/**
 * Propagates the version from the `VERSION` file to every manifest that carries one.
 *
 * `VERSION` is the single source of truth (docs/18-devops.md §1). Four files repeating a version
 * number is four chances to ship a release where the installer, the window title and the about box
 * disagree, so CI runs this with `--check` and fails when they drift.
 *
 * Usage: `node scripts/sync-version.mjs [--check]`
 */
import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const checkOnly = process.argv.includes('--check')

const version = readFileSync(join(root, 'VERSION'), 'utf8').trim()
if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error(`VERSION is not semver: "${version}"`)
  process.exit(1)
}

/** Each target reports its current version and knows how to rewrite it in place. */
const targets = [
  {
    file: 'package.json',
    read: (t) => JSON.parse(t).version,
    write: (t) => t.replace(/("version":\s*")[^"]*(")/, `$1${version}$2`),
  },
  {
    file: 'Cargo.toml',
    // Only the `[workspace.package]` version; member crates inherit it with `version.workspace`.
    read: (t) => t.match(/^\s*version\s*=\s*"([^"]+)"/m)?.[1],
    write: (t) => t.replace(/^(\s*version\s*=\s*")[^"]*(")/m, `$1${version}$2`),
  },
  {
    file: 'src-tauri/tauri.conf.json',
    read: (t) => JSON.parse(t).version,
    write: (t) => t.replace(/("version":\s*")[^"]*(")/, `$1${version}$2`),
  },
]

let drift = false

for (const target of targets) {
  const path = join(root, target.file)
  const text = readFileSync(path, 'utf8')
  const current = target.read(text)

  if (current === version) continue

  if (checkOnly) {
    console.error(`${target.file}: has "${current}", VERSION says "${version}"`)
    drift = true
    continue
  }

  writeFileSync(path, target.write(text))
  console.log(`${target.file}: ${current} -> ${version}`)
}

if (drift) {
  console.error('\nRun `node scripts/sync-version.mjs` to fix.')
  process.exit(1)
}

if (checkOnly) console.log(`all manifests agree on ${version}`)
