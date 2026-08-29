#!/usr/bin/env node
/**
 * Verifies the locale files against the rules in docs/14-configuracion-e-i18n.md §4.
 *
 * The legacy application shipped 78 keys that existed only in Spanish and 31 dead translations
 * that existed only in English, so the English build showed raw keys on screen. This check is what
 * stops that from happening again.
 *
 * Checks, in order:
 *   1. `es` and `en` have exactly the same set of keys.
 *   2. A key's named parameters (`{name}`) match across locales.
 *   3. No value is empty.
 *   4. Keys are sorted alphabetically within each level, so diffs stay readable.
 */
import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const localesDir = join(root, 'src', 'locales')

const CANONICAL = 'es'
const LOCALES = ['es', 'en']

function load(locale) {
  return JSON.parse(readFileSync(join(localesDir, `${locale}.json`), 'utf8'))
}

/** Flattens to `{ "A.B.C": "text" }`. */
function flatten(node, prefix = '', out = {}) {
  for (const [key, value] of Object.entries(node)) {
    const path = prefix ? `${prefix}.${key}` : key
    if (value !== null && typeof value === 'object' && !Array.isArray(value)) {
      flatten(value, path, out)
    } else {
      out[path] = value
    }
  }
  return out
}

function params(text) {
  return [...String(text).matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort()
}

/** Reports every object whose keys are not in ascending order. */
function unsortedPaths(node, prefix = '', out = []) {
  const keys = Object.keys(node)
  const sorted = [...keys].sort((a, b) => a.localeCompare(b, 'en'))
  if (keys.join('\u0000') !== sorted.join('\u0000')) {
    out.push(prefix || '(root)')
  }
  for (const [key, value] of Object.entries(node)) {
    if (value !== null && typeof value === 'object' && !Array.isArray(value)) {
      unsortedPaths(value, prefix ? `${prefix}.${key}` : key, out)
    }
  }
  return out
}

const problems = []
const trees = Object.fromEntries(LOCALES.map((l) => [l, load(l)]))
const flat = Object.fromEntries(LOCALES.map((l) => [l, flatten(trees[l])]))
const canonicalKeys = Object.keys(flat[CANONICAL])

for (const locale of LOCALES) {
  const keys = new Set(Object.keys(flat[locale]))

  if (locale !== CANONICAL) {
    for (const key of canonicalKeys) {
      if (!keys.has(key)) problems.push(`${locale}: missing key "${key}"`)
    }
    for (const key of keys) {
      if (!(key in flat[CANONICAL])) problems.push(`${locale}: key "${key}" is not in ${CANONICAL}`)
    }
  }

  for (const [key, value] of Object.entries(flat[locale])) {
    if (typeof value !== 'string' || value.trim() === '') {
      problems.push(`${locale}: "${key}" is empty`)
    }
    if (locale !== CANONICAL && key in flat[CANONICAL]) {
      const expected = params(flat[CANONICAL][key]).join(',')
      const actual = params(value).join(',')
      if (expected !== actual) {
        problems.push(`${locale}: "${key}" has parameters [${actual}], ${CANONICAL} has [${expected}]`)
      }
    }
  }

  for (const path of unsortedPaths(trees[locale])) {
    problems.push(`${locale}: keys under "${path}" are not sorted alphabetically`)
  }
}

if (problems.length > 0) {
  console.error('i18n check failed:\n')
  for (const problem of problems) console.error(`  - ${problem}`)
  console.error(`\n${problems.length} problem(s).`)
  process.exit(1)
}

console.log(`i18n check passed: ${canonicalKeys.length} keys, ${LOCALES.length} locales in sync`)
