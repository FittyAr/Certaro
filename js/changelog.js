const CHANGELOG_URL = "https://raw.githubusercontent.com/FittyAr/Certaro/main/CHANGELOG.md";

// Map technical categories to user-friendly labels
const GROUP_LABELS = {
  es: {
    "Added": "Nuevo", "Changed": "Mejorado", "Fixed": "Corregido", "Removed": "Eliminado",
    "Improved": "Mejorado", "Security": "Seguridad"
  },
  en: {
    "Added": "New", "Changed": "Improved", "Fixed": "Fixed", "Removed": "Removed",
    "Improved": "Improved", "Security": "Security"
  }
};

function friendlyGroup(name) {
  const lang = (typeof currentLang !== "undefined" ? currentLang : "es");
  const map = GROUP_LABELS[lang] || GROUP_LABELS.es;
  return map[name] || name;
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
    renderChangelog(entries, container);
    if (fallback) fallback.style.display = "none";
  } catch (e) {
    // Show fallback content instead of error
    if (fallback) fallback.style.display = "block";
    const errEl = document.getElementById("changelog-error");
    // Only show error if we have no entries at all and fallback also fails
    if (!fallback) { if (errEl) errEl.style.display = "block"; }
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
      current = { version: ver, date: h2[2].trim().replace(/^[-\s]+/, ""), groups: {}, raw: [] };
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
      // Filter out overly technical items for user-facing view
      const text = bullet[1].trim();
      if (text.length < 5) continue;
      current.groups[current._group].push(text);
    }
  }
  if (current) entries.push(current);
  // Remove entries with no groups (empty)
  return entries.filter(e => Object.keys(e.groups).length > 0);
}

function renderChangelog(entries, container) {
  if (!container) return;
  const showAll = entries.length > 3;
  let expanded = false;

  function render(list) {
    container.innerHTML = list.map((e, i) => {
      const isLatest = i === 0;
      const groupsHtml = Object.entries(e.groups).map(([g, items]) => {
        return `<div class="cl-group"><span class="cl-tag ${tagClass(g)}">${escapeHtml(friendlyGroup(g))}</span><ul>${items.map(it=>`<li>${escapeHtml(it)}</li>`).join("")}</ul></div>`;
      }).join("");
      return `<article class="cl-entry ${isLatest ? 'cl-latest' : ''}">
        <div class="cl-dot">${String(entries.length - i).padStart(2,"0")}</div>
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

    if (showAll) {
      const btn = document.getElementById("changelog-toggle");
      if (btn) {
        btn.style.display = "inline-flex";
        const key = expanded ? "changelog.showLess" : "changelog.showMore";
        btn.textContent = (typeof t !== "undefined" ? t(key) : (expanded ? "Ver menos" : "Ver más"));
        btn.onclick = () => {
          expanded = !expanded;
          render(expanded ? entries : entries.slice(0,3));
        };
      }
    }
  }
  render(entries.slice(0,3));

  // Patch locale switch to re-render with translated tags
  if (typeof window !== "undefined" && window.setLocale && !window._clPatched) {
    window._clPatched = true;
    const orig = window.setLocale;
    window.setLocale = function(lang) {
      orig(lang);
      const isExpanded = container.querySelectorAll(".cl-entry").length > 3;
      render(isExpanded ? entries : entries.slice(0,3));
      if (typeof applyTranslations === "function") applyTranslations();
    };
  }
}

function tagClass(g) {
  const l = g.toLowerCase();
  if (l.includes("added") || l.includes("nuevo")) return "added";
  if (l.includes("fixed") || l.includes("correg")) return "fixed";
  if (l.includes("changed") || l.includes("mejor")) return "changed";
  if (l.includes("removed") || l.includes("elimin")) return "removed";
  return "";
}
function escapeHtml(s){ return s.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;"); }

document.addEventListener("DOMContentLoaded", loadChangelog);
