#!/usr/bin/env node
/**
 * Rewrites the locale files with every level sorted the way `check-i18n.mjs` demands, so a new
 * block of keys can be added where it reads best and put in order afterwards.
 */
import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const localesDir = join(dirname(fileURLToPath(import.meta.url)), '..', 'src', 'locales')

function sortTree(node) {
  if (node === null || typeof node !== 'object' || Array.isArray(node)) return node
  const out = {}
  for (const key of Object.keys(node).sort((a, b) => a.localeCompare(b, 'en'))) {
    out[key] = sortTree(node[key])
  }
  return out
}

for (const locale of ['es', 'en']) {
  const path = join(localesDir, `${locale}.json`)
  const tree = JSON.parse(readFileSync(path, 'utf8'))
  writeFileSync(path, `${JSON.stringify(sortTree(tree), null, 2)}\n`, 'utf8')
  console.log(`sorted ${locale}.json`)
}
