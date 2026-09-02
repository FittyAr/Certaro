const CHANGELOG_URL = "https://raw.githubusercontent.com/FittyAr/Certaro/main/CHANGELOG.md";

// Map technical categories to user-friendly labels
const GROUP_LABELS = {
  es: {
    "Added": "Nuevo",
    "Changed": "Mejorado",
    "Fixed": "Corregido",
    "Removed": "Eliminado",
    "Improved": "Mejorado",
    "Security": "Seguridad",
    "Technical": "Técnico",
    "Known limitations": "Notas de versión"
  },
  en: {
    "Added": "New",
    "Changed": "Improved",
    "Fixed": "Fixed",
    "Removed": "Removed",
    "Improved": "Improved",
    "Security": "Security",
    "Technical": "Technical",
    "Known limitations": "Release notes"
  }
};

let cachedEntries = null;
let isExpanded = false;

function friendlyGroup(name) {
  const lang = (typeof currentLang !== "undefined" ? currentLang : "es");
  const map = GROUP_LABELS[lang] || GROUP_LABELS.es;
  return map[name] || name;
}

function formatInlineMarkdown(text) {
  if (!text) return "";
  // Clean up any placeholder token
  let clean = text.replace(/__ElectroObraApp_PLACEHOLDER__/g, "ElectroObra");

  // Escape basic HTML entities first
  let html = clean
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");

  // Format bold **text**
  html = html.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  // Format inline code `code`
  html = html.replace(/`([^`]+)`/g, "<code>$1</code>");
  // Format markdown links [text](url)
  html = html.replace(/\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g, '<a href="$2" target="_blank" rel="noopener">$1</a>');

  return html;
}

async function loadChangelog() {
  const container = document.getElementById("changelog-timeline");
  const loading = document.getElementById("changelog-loading");
  const fallback = document.getElementById("changelog-fallback");

  try {
    const res = await fetch(CHANGELOG_URL, { cache: "no-store" });
    if (!res.ok) throw new Error("fetch failed");
    const md = await res.text();
    const entries = parseChangelog(md);
    if (!entries.length) throw new Error("no entries");

    cachedEntries = entries;
    renderChangelog(entries, container);
    if (fallback) fallback.style.display = "none";
  } catch (e) {
    // Show fallback content with full localization
    renderFallback();
  } finally {
    if (loading) loading.style.display = "none";
  }
}

function parseChangelog(md) {
  const lines = md.split("\n");
  const entries = [];
  let current = null;

  for (const raw of lines) {
    const h2 = raw.match(/^##\s+\[?([^\]]+)\]?\s*-?\s*(.*)/);
    if (h2) {
      if (current) entries.push(current);
      const ver = h2[1].trim();
      // Skip "Unreleased" or empty
      if (/unreleased/i.test(ver)) { current = null; continue; }
      current = { version: ver, date: h2[2].trim().replace(/^[-\s]+/, ""), groups: {} };
      continue;
    }
    if (!current) continue;
    const h3 = raw.match(/^###\s+(.+)/);
    if (h3) {
      current._group = h3[1].trim();
      if (!current.groups[current._group]) current.groups[current._group] = [];
      continue;
    }
    const bullet = raw.match(/^\s*[-*]\s+(.+)/);
    if (bullet && current._group) {
      const text = bullet[1].trim();
      if (text.length < 3) continue;
      current.groups[current._group].push(text);
    }
  }
  if (current) entries.push(current);
  return entries.filter(e => Object.keys(e.groups).length > 0);
}

function renderChangelog(entries, container) {
  if (!container) return;
  const showToggle = entries.length > 2;

  const displayList = isExpanded ? entries : entries.slice(0, 2);

  container.innerHTML = displayList.map((e, i) => {
    const isLatest = i === 0;
    const groupsHtml = Object.entries(e.groups).map(([g, items]) => {
      const tagLabel = friendlyGroup(g);
      const cls = tagClass(g);
      return `<div class="cl-group">
        <span class="cl-tag ${cls}">${escapeHtml(tagLabel)}</span>
        <ul>${items.map(it => `<li>${formatInlineMarkdown(it)}</li>`).join("")}</ul>
      </div>`;
    }).join("");

    return `<article class="cl-entry ${isLatest ? 'cl-latest' : ''}">
      <div class="cl-dot">${String(entries.length - i).padStart(2, "0")}</div>
      <div class="cl-card">
        <div class="cl-card-head">
          <span class="cl-version">${escapeHtml(e.version)}</span>
          ${e.date ? `<span class="cl-date">${escapeHtml(e.date)}</span>` : ""}
          ${isLatest ? `<span class="cl-badge">${typeof t !== "undefined" ? t("changelog.latest") : "LO ÚLTIMO"}</span>` : ""}
        </div>
        ${groupsHtml}
      </div>
    </article>`;
  }).join("");

  const btn = document.getElementById("changelog-toggle");
  if (btn) {
    if (showToggle) {
      btn.style.display = "inline-flex";
      const key = isExpanded ? "changelog.showLess" : "changelog.showMore";
      btn.textContent = (typeof t !== "undefined" ? t(key) : (isExpanded ? "Ver menos" : "Ver más"));
      btn.onclick = () => {
        isExpanded = !isExpanded;
        renderChangelog(entries, container);
      };
    } else {
      btn.style.display = "none";
    }
  }
}

function renderFallback() {
  const fallback = document.getElementById("changelog-fallback");
  const list = document.getElementById("changelog-fallback-list");
  if (fallback) fallback.style.display = "block";
  if (list && typeof t !== "undefined") {
    const items = t("changelog.fallback.items").split("|");
    list.innerHTML = items.map(i => `<li>${escapeHtml(i.trim())}</li>`).join("");
  }
}

function tagClass(g) {
  const l = g.toLowerCase();
  if (l.includes("added") || l.includes("nuevo")) return "added";
  if (l.includes("fixed") || l.includes("correg")) return "fixed";
  if (l.includes("changed") || l.includes("mejor") || l.includes("improved")) return "changed";
  if (l.includes("removed") || l.includes("elimin")) return "removed";
  if (l.includes("technical") || l.includes("técnic")) return "changed";
  return "";
}

function escapeHtml(s) {
  return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

// Re-render when language changes
window.addEventListener("certaro:localechange", () => {
  const container = document.getElementById("changelog-timeline");
  if (cachedEntries && container) {
    renderChangelog(cachedEntries, container);
  } else {
    renderFallback();
  }
});

document.addEventListener("DOMContentLoaded", loadChangelog);

