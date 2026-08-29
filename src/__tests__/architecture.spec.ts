import { readFileSync } from 'node:fs'
import { globSync } from 'node:fs'
import { join, relative, sep } from 'node:path'
import { describe, expect, it } from 'vitest'

import en from '@/locales/en.json'
import es from '@/locales/es.json'

/**
 * The verifiable rules of `docs/16-frontend.md` §8. These are the tests that keep the interface
 * from degrading: the legacy system had none of them and ended up with loose colours and Spanish
 * sentences inside the code.
 */

const SRC = join(process.cwd(), 'src')

function files(pattern: string): string[] {
  return globSync(pattern, { cwd: SRC })
    .map((p) => join(SRC, p))
    .filter((p) => !p.includes(`${sep}__tests__${sep}`))
}

function read(path: string): string {
  return readFileSync(path, 'utf8')
}

function shortName(path: string): string {
  return relative(SRC, path).replaceAll(sep, '/')
}

describe('reglas de arquitectura del frontend', () => {
  it('sin colores literales fuera de tokens.css', () => {
    // A literal colour looks right in one theme and wrong in the other, and there are two.
    const palette =
      /\b(?:bg|text|border|ring|fill|stroke)-(?:slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-\d{2,3}\b/
    const literal = /#[0-9a-fA-F]{3,8}\b|\brgba?\(/

    for (const file of files('**/*.vue')) {
      const content = read(file)
      expect(content, `${shortName(file)}: clase de color de Tailwind`).not.toMatch(palette)
      expect(content, `${shortName(file)}: color literal`).not.toMatch(literal)
    }
  })

  it('ningun archivo importa Button de primevue', () => {
    for (const file of [...files('**/*.vue'), ...files('**/*.ts')]) {
      expect(read(file), shortName(file)).not.toMatch(/from ['"]primevue\/button['"]/)
    }
  })

  it('invoke solo aparece en api/client.ts', () => {
    for (const file of [...files('**/*.vue'), ...files('**/*.ts')]) {
      if (shortName(file) === 'api/client.ts') continue
      expect(read(file), shortName(file)).not.toMatch(/@tauri-apps\/api\/core/)
    }
  })

  it('ninguna vista importa directamente de api/', () => {
    for (const file of files('views/**/*.vue')) {
      expect(read(file), shortName(file)).not.toMatch(/from ['"]@\/api\//)
    }
  })

  it('api/ no importa stores/', () => {
    for (const file of files('api/**/*.ts')) {
      expect(read(file), shortName(file)).not.toMatch(/from ['"]@\/stores\//)
    }
  })

  it('components/ui no importa stores ni api', () => {
    for (const file of files('components/ui/**/*.{vue,ts}')) {
      expect(read(file), shortName(file)).not.toMatch(/from ['"]@\/(stores|api)\//)
    }
  })

  it('sin formatos de fecha escritos a mano', () => {
    // The format is configuration, not translation, and lives in `Locale.FormatoFecha`.
    for (const file of [...files('components/**/*.vue'), ...files('views/**/*.vue')]) {
      if (shortName(file).endsWith('domain/DateInput.vue')) continue
      expect(read(file), shortName(file)).not.toMatch(/toLocaleDateString|['"]dd\/MM|['"]MM\/dd/)
    }
  })

  it('las dos traducciones tienen exactamente las mismas claves', () => {
    expect(flatten(es)).toEqual(flatten(en))
  })

  it('toda clave usada en un template existe en los dos idiomas', () => {
    const known = new Set(flatten(es))

    for (const file of [...files('components/**/*.vue'), ...files('views/**/*.vue')]) {
      const content = read(file)
      for (const match of content.matchAll(/\$?t\(\s*['"]([A-Z][\w.]*)['"]/g)) {
        expect(known, `${shortName(file)}: ${match[1]}`).toContain(match[1])
      }
    }
  })
})

/** Dotted keys of a nested translation object, sorted. */
function flatten(node: object, prefix = ''): string[] {
  return Object.entries(node)
    .flatMap(([key, value]) => {
      const path = prefix ? `${prefix}.${key}` : key
      return typeof value === 'object' && value !== null ? flatten(value, path) : [path]
    })
    .sort()
}
