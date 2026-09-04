#!/usr/bin/env node
/**
 * Herramienta de auditoría para detectar archivos monolíticos o excesivamente grandes en Certaro.
 * 
 * Uso:
 *   node scripts/find-monolithic-files.mjs [opciones]
 * 
 * Opciones:
 *   --threshold <num>   Límite de líneas para considerar un archivo monolítico (default: 300)
 *   --top <num>         Mostrar solo los N archivos más grandes (default: todos los que superen threshold)
 *   --dir <dirs>        Directorios a escanear separados por coma (default: src,crates,src-tauri)
 *   --ext <exts>        Extensiones a auditar separadas por coma (default: .vue,.ts,.js,.rs)
 *   --ignore-tests      Ignorar archivos en carpetas o con nombres de test (tests/, *.spec.ts, *.test.ts)
 *   --output <file>     Guardar el reporte en formato Markdown en la ruta especificada
 *   --markdown          Imprimir el reporte directamente en formato Markdown por consola
 *   --json              Imprimir el resultado en formato JSON
 *   --help, -h          Muestra este mensaje de ayuda
 */

import { readFileSync, writeFileSync, existsSync, readdirSync, statSync } from 'node:fs';
import { resolve, relative, join, extname } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = resolve(fileURLToPath(import.meta.url), '../..');

// Configuración por defecto
const DEFAULT_CONFIG = {
  threshold: 300,
  top: 0,
  dirs: ['src', 'crates', 'src-tauri'],
  exts: ['.vue', '.ts', '.js', '.rs'],
  ignoreTests: false,
  outputFile: null,
  markdownOnly: false,
  jsonOnly: false,
};

function parseArgs() {
  const args = process.argv.slice(2);
  const config = { ...DEFAULT_CONFIG };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--help' || arg === '-h') {
      printHelp();
      process.exit(0);
    } else if (arg === '--threshold') {
      config.threshold = parseInt(args[++i], 10) || DEFAULT_CONFIG.threshold;
    } else if (arg === '--top') {
      config.top = parseInt(args[++i], 10) || 0;
    } else if (arg === '--dir') {
      config.dirs = args[++i].split(',').map((d) => d.trim());
    } else if (arg === '--ext') {
      config.exts = args[++i].split(',').map((e) => (e.startsWith('.') ? e.trim() : `.${e.trim()}`));
    } else if (arg === '--ignore-tests') {
      config.ignoreTests = true;
    } else if (arg === '--output') {
      config.outputFile = args[++i];
    } else if (arg === '--markdown') {
      config.markdownOnly = true;
    } else if (arg === '--json') {
      config.jsonOnly = true;
    }
  }

  return config;
}

function printHelp() {
  console.log(`
Auditor de Archivos Monolíticos - Certaro
=========================================
Identifica archivos que superan el umbral de líneas de código y sugiere estrategias de segmentación.

Uso:
  node scripts/find-monolithic-files.mjs [opciones]
  pnpm audit:monoliths [opciones]

Opciones:
  --threshold <num>   Mínimo de líneas para clasificar como monolito (por defecto: 300)
  --top <num>         Limitar la lista a los N archivos más extensos (ej. --top 20)
  --dir <dirs>        Directorios a analizar (por defecto: src,crates,src-tauri)
  --ext <exts>        Extensiones a incluir (por defecto: .vue,.ts,.js,.rs)
  --ignore-tests      Excluir archivos de suites de pruebas (tests, specs)
  --output <ruta>     Generar reporte en Markdown listo para el plan de segmentación
  --markdown          Mostrar reporte en formato Markdown en terminal
  --json              Salida en formato JSON estructurado
  -h, --help          Mostrar esta ayuda
`);
}

function isTestFile(filePath) {
  const norm = filePath.replace(/\\/g, '/');
  return (
    norm.includes('/tests/') ||
    norm.includes('/__tests__/') ||
    norm.endsWith('.test.ts') ||
    norm.endsWith('.spec.ts') ||
    norm.endsWith('.test.js') ||
    norm.endsWith('.spec.js')
  );
}

function analyzeVueFile(content) {
  const templateMatch = content.match(/<template[^>]*>([\s\S]*?)<\/template>/i);
  const scriptMatch = content.match(/<script[^>]*>([\s\S]*?)<\/script>/i);
  const styleMatch = content.match(/<style[^>]*>([\s\S]*?)<\/style>/i);

  const templateLines = templateMatch ? templateMatch[1].split('\n').length : 0;
  const scriptLines = scriptMatch ? scriptMatch[1].split('\n').length : 0;
  const styleLines = styleMatch ? styleMatch[1].split('\n').length : 0;

  return {
    isVue: true,
    templateLines,
    scriptLines,
    styleLines,
  };
}

function analyzeRustFile(content) {
  const fnCount = (content.match(/\b(pub\s+)?(async\s+)?fn\s+[a-zA-Z0-9_]+/g) || []).length;
  const structCount = (content.match(/\b(pub\s+)?struct\s+[a-zA-Z0-9_]+/g) || []).length;
  const enumCount = (content.match(/\b(pub\s+)?enum\s+[a-zA-Z0-9_]+/g) || []).length;
  const implCount = (content.match(/\bimpl(\s*<[^>]+>)?\s+[a-zA-Z0-9_]+/g) || []).length;

  return {
    isRust: true,
    fnCount,
    structCount,
    enumCount,
    implCount,
  };
}

function analyzeTsJsFile(content) {
  const fnCount = (content.match(/\b(export\s+)?(async\s+)?function\s+[a-zA-Z0-9_]+/g) || []).length;
  const arrowFnCount = (content.match(/\b(export\s+)?const\s+[a-zA-Z0-9_]+\s*=\s*(async\s*)?\([^)]*\)\s*=>/g) || []).length;
  const classCount = (content.match(/\b(export\s+)?class\s+[a-zA-Z0-9_]+/g) || []).length;
  const interfaceCount = (content.match(/\b(export\s+)?interface\s+[a-zA-Z0-9_]+/g) || []).length;
  const typeCount = (content.match(/\b(export\s+)?type\s+[a-zA-Z0-9_]+\s*=/g) || []).length;

  return {
    isTsJs: true,
    fnCount: fnCount + arrowFnCount,
    classCount,
    interfaceCount,
    typeCount,
  };
}

function analyzeFile(fullPath) {
  const relPath = relative(rootDir, fullPath).replace(/\\/g, '/');
  const stat = statSync(fullPath);
  const content = readFileSync(fullPath, 'utf8');
  const lines = content.split('\n');
  const totalLines = lines.length;

  let blankLines = 0;
  let commentLines = 0;
  let inBlockComment = false;

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) {
      blankLines++;
      continue;
    }

    if (inBlockComment) {
      commentLines++;
      if (trimmed.includes('*/')) inBlockComment = false;
      continue;
    }

    if (trimmed.startsWith('/*')) {
      commentLines++;
      if (!trimmed.includes('*/')) inBlockComment = true;
      continue;
    }

    if (trimmed.startsWith('//') || trimmed.startsWith('///') || trimmed.startsWith('<!--') || trimmed.startsWith('*')) {
      commentLines++;
      continue;
    }
  }

  const codeLines = totalLines - blankLines - commentLines;
  const sizeKb = (stat.size / 1024).toFixed(1);
  const ext = extname(fullPath);

  let details = {};
  let suggestedSegmentation = '';

  if (ext === '.vue') {
    details = analyzeVueFile(content);
    if (details.scriptLines > 350) {
      suggestedSegmentation = 'Extraer logica y estado a composables dedicados (`use*.ts`)';
    } else if (details.templateLines > 300) {
      suggestedSegmentation = 'Dividir vista en subcomponentes modulares (tablas, modales, panels)';
    } else if (details.styleLines > 150) {
      suggestedSegmentation = 'Migrar estilos inline/custom a utilidades Tailwind o CSS modular';
    } else {
      suggestedSegmentation = 'Extraer subcomponentes secundarios y modularizar template';
    }
  } else if (ext === '.rs') {
    details = analyzeRustFile(content);
    if (relPath.includes('use_cases')) {
      suggestedSegmentation = 'Descomponer caso de uso en submódulos (orquestación, cálculo, validaciones)';
    } else if (relPath.includes('repositories') || relPath.includes('ports')) {
      suggestedSegmentation = 'Separar traits/puertos por agregados o entidades de dominio';
    } else if (relPath.includes('persistence') || relPath.includes('seed')) {
      suggestedSegmentation = 'Modularizar datos y queries por entidad de base de datos';
    } else if (relPath.includes('transfer')) {
      suggestedSegmentation = 'Dividir etapas de transferencia/ETL en pipelines específicos';
    } else {
      suggestedSegmentation = 'Modularizar structs e impl en archivos de submódulo (`folder/mod.rs`)';
    }
  } else if (ext === '.ts' || ext === '.js') {
    details = analyzeTsJsFile(content);
    if (relPath.includes('api/client')) {
      suggestedSegmentation = 'Dividir en clientes de API modulares por dominio (órdenes, facturas, personal)';
    } else if (relPath.includes('registry') || relPath.includes('help')) {
      suggestedSegmentation = 'Separar diccionarios o contenidos estáticos en archivos de datos por sección';
    } else if (details.fnCount > 15) {
      suggestedSegmentation = 'Agrupar funciones en módulos utilitarios cohesivos';
    } else {
      suggestedSegmentation = 'Separar responsabilidades por dominio o capa';
    }
  }

  // Nivel de severidad
  let severity = 'MEDIO';
  let severityColor = '\x1b[36m'; // Cyan
  if (totalLines >= 800) {
    severity = 'CRÍTICO';
    severityColor = '\x1b[31m'; // Red
  } else if (totalLines >= 500) {
    severity = 'ALTO';
    severityColor = '\x1b[33m'; // Yellow
  }

  let layer = 'Otro';
  if (relPath.startsWith('src/')) layer = 'Frontend (Vue/TS)';
  else if (relPath.startsWith('crates/') || relPath.startsWith('src-tauri/')) layer = 'Backend (Rust)';

  return {
    path: relPath,
    fullPath,
    totalLines,
    codeLines,
    commentLines,
    blankLines,
    sizeKb,
    ext,
    layer,
    severity,
    severityColor,
    isTest: isTestFile(relPath),
    details,
    suggestedSegmentation,
  };
}

function scanDirectory(dirPath, config, fileList = []) {
  if (!existsSync(dirPath)) return fileList;

  const entries = readdirSync(dirPath, { withFileTypes: true });
  for (const entry of entries) {
    const full = join(dirPath, entry.name);
    if (entry.isDirectory()) {
      if (['node_modules', 'target', 'dist', '.git', '.husky'].includes(entry.name)) {
        continue;
      }
      scanDirectory(full, config, fileList);
    } else if (entry.isFile()) {
      const ext = extname(entry.name);
      if (config.exts.includes(ext)) {
        if (config.ignoreTests && isTestFile(full)) {
          continue;
        }
        fileList.push(full);
      }
    }
  }
  return fileList;
}

function generateMarkdownReport(results, config) {
  const dateStr = new Date().toISOString().split('T')[0];
  const criticalCount = results.filter((r) => r.severity === 'CRÍTICO').length;
  const highCount = results.filter((r) => r.severity === 'ALTO').length;
  const mediumCount = results.filter((r) => r.severity === 'MEDIO').length;

  let md = `# Reporte de Archivos Monolíticos y Plan de Segmentación\n\n`;
  md += `> **Fecha de generación:** ${dateStr}  \n`;
  md += `> **Criterio:** Archivos con >= ${config.threshold} líneas  \n`;
  md += `> **Total detectados:** ${results.length} (🔴 ${criticalCount} Críticos, 🟡 ${highCount} Altos, 🔵 ${mediumCount} Medios)\n\n`;

  md += `## 1. Resumen Ejecutivo\n\n`;
  md += `| Métrica | Valor |\n`;
  md += `| :--- | :--- |\n`;
  md += `| Archivos monolíticos | **${results.length}** |\n`;
  md += `| 🔴 Críticos (>= 800 líneas) | **${criticalCount}** |\n`;
  md += `| 🟡 Altos (500 - 799 líneas) | **${highCount}** |\n`;
  md += `| 🔵 Moderados (${config.threshold} - 499 líneas) | **${mediumCount}** |\n\n`;

  md += `## 2. Inventario de Archivos Monolíticos\n\n`;
  md += `| # | Nivel | Archivo | Capa | Líneas Totales | Líneas Código | Tamaño (KB) | Sugerencia de Segmentación |\n`;
  md += `|---|---|---|---|---|---|---|---|\n`;

  results.forEach((r, idx) => {
    const badge = r.severity === 'CRÍTICO' ? '🔴 Crítico' : r.severity === 'ALTO' ? '🟡 Alto' : '🔵 Medio';
    md += `| ${idx + 1} | ${badge} | \`${r.path}\` | ${r.layer} | ${r.totalLines} | ${r.codeLines} | ${r.sizeKb} KB | ${r.suggestedSegmentation} |\n`;
  });

  md += `\n## 3. Desglose Detallado por Componente\n\n`;

  // Frontend breakdown
  const frontendMonoliths = results.filter((r) => r.layer.startsWith('Frontend'));
  if (frontendMonoliths.length > 0) {
    md += `### 3.1. Frontend (Vue / TypeScript)\n\n`;
    frontendMonoliths.forEach((r) => {
      md += `#### \`${r.path}\` (${r.totalLines} líneas - ${r.severity})\n`;
      if (r.ext === '.vue' && r.details) {
        md += `- **Composición:** \`<template>\`: ${r.details.templateLines} líneas | \`<script setup>\`: ${r.details.scriptLines} líneas | \`<style>\`: ${r.details.styleLines} líneas\n`;
      } else if (r.ext === '.ts' && r.details) {
        md += `- **Estructuras:** ${r.details.fnCount} funciones/métodos, ${r.details.interfaceCount} interfaces, ${r.details.typeCount} tipos\n`;
      }
      md += `- **Estrategia recomendada:** ${r.suggestedSegmentation}\n\n`;
    });
  }

  // Backend breakdown
  const backendMonoliths = results.filter((r) => r.layer.startsWith('Backend'));
  if (backendMonoliths.length > 0) {
    md += `### 3.2. Backend (Rust)\n\n`;
    backendMonoliths.forEach((r) => {
      md += `#### \`${r.path}\` (${r.totalLines} líneas - ${r.severity})\n`;
      if (r.details && r.details.isRust) {
        md += `- **Estructuras:** ${r.details.fnCount} funciones, ${r.details.implCount} bloques impl, ${r.details.structCount} structs, ${r.details.enumCount} enums\n`;
      }
      md += `- **Estrategia recomendada:** ${r.suggestedSegmentation}\n\n`;
    });
  }

  md += `## 4. Plan de Segmentación Recomendado\n\n`;
  md += `A partir de los archivos identificados, se recomienda priorizar en las siguientes fases:\n\n`;
  md += `1. **Fase 1: Desacoplamiento de Clientes API Frontend**\n`;
  md += `   - Descomponer \`src/api/client.ts\` (1700+ líneas) en submódulos por dominio (\`src/api/domains/ordenes.ts\`, \`personal.ts\`, \`facturas.ts\`, etc.).\n`;
  md += `2. **Fase 2: Segmentación de Vistas Complejas en Vue**\n`;
  md += `   - Extraer la lógica de \`CalendarioView.vue\` y \`KanbanView.vue\` a composables (\`useCalendario.ts\`, \`useKanban.ts\`).\n`;
  md += `   - Modularizar sub-componentes visuales (tablas, formularios modales, cards).\n`;
  md += `3. **Fase 3: Refactorización de Capas de Negocio y Repositorios en Rust**\n`;
  md += `   - Descomponer \`repositories.rs\` dividiendo los traits de puertos en archivos específicos por agregado.\n`;
  md += `   - Segmentar casos de uso extensos (\`liquidaciones.rs\`, \`kanban.rs\`, \`calendario.rs\`) en submódulos jerárquicos.\n`;

  return md;
}

function printConsoleTable(results, config) {
  const reset = '\x1b[0m';
  const bold = '\x1b[1m';
  const dim = '\x1b[2m';
  const green = '\x1b[32m';

  console.log(`\n${bold}========================================================================================${reset}`);
  console.log(`${bold}           AUDITORÍA DE ARCHIVOS MONOLÍTICOS - CERTARO${reset}`);
  console.log(`${bold}========================================================================================${reset}`);
  console.log(`${dim}Umbral mínimo: ${config.threshold} líneas | Directorios: ${config.dirs.join(', ')}${reset}\n`);

  if (results.length === 0) {
    console.log(`${green}✔ No se encontraron archivos que superen el umbral de ${config.threshold} líneas.${reset}\n`);
    return;
  }

  const critical = results.filter((r) => r.severity === 'CRÍTICO').length;
  const high = results.filter((r) => r.severity === 'ALTO').length;
  const medium = results.filter((r) => r.severity === 'MEDIO').length;

  console.log(`Total encontrados: ${bold}${results.length}${reset} archivos (` +
    `\x1b[31m${critical} Críticos\x1b[0m, ` +
    `\x1b[33m${high} Altos\x1b[0m, ` +
    `\x1b[36m${medium} Medios\x1b[0m)\n`);

  console.log(`${bold}${'#'.padEnd(4)} ${'Nivel'.padEnd(10)} ${'Líneas'.padStart(7)}  ${'Código'.padStart(7)}  ${'Tamaño'.padStart(9)}  ${'Archivo'}${reset}`);
  console.log(`${dim}${'-'.repeat(88)}${reset}`);

  results.forEach((r, idx) => {
    const num = `${idx + 1}.`.padEnd(4);
    const sev = `${r.severityColor}${r.severity.padEnd(10)}${reset}`;
    const tot = `${bold}${r.totalLines.toString().padStart(7)}${reset}`;
    const cod = `${dim}${r.codeLines.toString().padStart(7)}${reset}`;
    const sz = `${r.sizeKb} KB`.padStart(9);
    console.log(`${num} ${sev} ${tot}  ${cod}  ${sz}  ${r.path}`);
  });

  console.log(`${dim}${'-'.repeat(88)}${reset}`);
  console.log(`\n${bold}💡 Sugerencias inmediatas de segmentación:${reset}`);
  results.slice(0, 8).forEach((r) => {
    console.log(`  • ${bold}${r.path}${reset} (${r.totalLines} lns): ${dim}${r.suggestedSegmentation}${reset}`);
  });

  if (!config.outputFile) {
    console.log(`\n${dim}Tip: Ejecuta con --output docs/plan-segmentacion.md para exportar el reporte en Markdown.${reset}\n`);
  }
}

function main() {
  const config = parseArgs();

  const allFiles = [];
  for (const dir of config.dirs) {
    const fullDirPath = resolve(rootDir, dir);
    scanDirectory(fullDirPath, config, allFiles);
  }

  const analyzed = allFiles
    .map((f) => analyzeFile(f))
    .filter((r) => r.totalLines >= config.threshold)
    .sort((a, b) => b.totalLines - a.totalLines);

  const results = config.top > 0 ? analyzed.slice(0, config.top) : analyzed;

  if (config.jsonOnly) {
    console.log(JSON.stringify(results, null, 2));
    return;
  }

  if (config.markdownOnly) {
    console.log(generateMarkdownReport(results, config));
    return;
  }

  printConsoleTable(results, config);

  if (config.outputFile) {
    const outPath = resolve(rootDir, config.outputFile);
    const md = generateMarkdownReport(results, config);
    writeFileSync(outPath, md, 'utf8');
    console.log(`\x1b[32m✔ Reporte guardado con éxito en:\x1b[0m ${outPath}\n`);
  }
}

main();
