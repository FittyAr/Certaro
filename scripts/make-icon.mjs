#!/usr/bin/env node
/**
 * Draws the source icon that `pnpm tauri icon` expands into every platform format.
 *
 * Written by hand with `zlib` instead of pulling in an image library: the icon is a few solid
 * shapes on a rounded square, and a build-time dependency to draw them is not worth it. Run this
 * again only if the mark changes.
 *
 * Usage: `node scripts/make-icon.mjs [size] [output]`
 */
import { deflateSync } from 'node:zlib'
import { writeFileSync, mkdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const size = Number(process.argv[2] ?? 1024)
const output = process.argv[3] ?? join(root, 'src-tauri', 'icons', 'source.png')

// Kept in sync with --primary / --accent of src/assets/tokens.css, dark theme.
const BACKGROUND = [15, 23, 42, 255] // hsl(222 47% 11%)
const MARK = [96, 165, 250, 255] // hsl(217 91% 60%)

const pixels = Buffer.alloc(size * size * 4)

const radius = size * 0.22
const inside = (x, y) => {
  // Rounded square: outside the corner quadrants, distance to the corner centre decides.
  const cx = Math.min(Math.max(x, radius), size - radius)
  const cy = Math.min(Math.max(y, radius), size - radius)
  const dx = x - cx
  const dy = y - cy
  return dx * dx + dy * dy <= radius * radius
}

/**
 * A lightning bolt as two triangles, in a 0..1 coordinate space so it scales with the canvas.
 * Point-in-polygon by ray casting.
 */
const BOLT = [
  [0.56, 0.14],
  [0.3, 0.55],
  [0.46, 0.55],
  [0.4, 0.86],
  [0.68, 0.44],
  [0.51, 0.44],
]

const inBolt = (nx, ny) => {
  let hit = false
  for (let i = 0, j = BOLT.length - 1; i < BOLT.length; j = i++) {
    const [xi, yi] = BOLT[i]
    const [xj, yj] = BOLT[j]
    if (yi > ny !== yj > ny && nx < ((xj - xi) * (ny - yi)) / (yj - yi) + xi) hit = !hit
  }
  return hit
}

for (let y = 0; y < size; y++) {
  for (let x = 0; x < size; x++) {
    const offset = (y * size + x) * 4
    if (!inside(x + 0.5, y + 0.5)) continue

    const colour = inBolt((x + 0.5) / size, (y + 0.5) / size) ? MARK : BACKGROUND
    pixels[offset] = colour[0]
    pixels[offset + 1] = colour[1]
    pixels[offset + 2] = colour[2]
    pixels[offset + 3] = colour[3]
  }
}

// PNG wants a filter byte in front of every scanline; 0 means "no filter".
const raw = Buffer.alloc(size * (size * 4 + 1))
for (let y = 0; y < size; y++) {
  raw[y * (size * 4 + 1)] = 0
  pixels.copy(raw, y * (size * 4 + 1) + 1, y * size * 4, (y + 1) * size * 4)
}

const CRC_TABLE = Array.from({ length: 256 }, (_, n) => {
  let c = n
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1
  return c >>> 0
})

const crc32 = (buf) => {
  let c = 0xffffffff
  for (const byte of buf) c = CRC_TABLE[(c ^ byte) & 0xff] ^ (c >>> 8)
  return (c ^ 0xffffffff) >>> 0
}

const chunk = (type, data) => {
  const length = Buffer.alloc(4)
  length.writeUInt32BE(data.length)
  const body = Buffer.concat([Buffer.from(type, 'ascii'), data])
  const crc = Buffer.alloc(4)
  crc.writeUInt32BE(crc32(body))
  return Buffer.concat([length, body, crc])
}

const ihdr = Buffer.alloc(13)
ihdr.writeUInt32BE(size, 0)
ihdr.writeUInt32BE(size, 4)
ihdr[8] = 8 // bit depth
ihdr[9] = 6 // colour type: RGBA
ihdr[10] = 0 // deflate
ihdr[11] = 0 // adaptive filtering
ihdr[12] = 0 // no interlace

const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk('IHDR', ihdr),
  chunk('IDAT', deflateSync(raw, { level: 9 })),
  chunk('IEND', Buffer.alloc(0)),
])

mkdirSync(dirname(output), { recursive: true })
writeFileSync(output, png)
console.log(`wrote ${output} (${size}x${size}, ${png.length} bytes)`)
