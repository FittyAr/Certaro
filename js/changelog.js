const CHANGELOG_URL = "https://raw.githubusercontent.com/FittyAr/Certaro/main/CHANGELOG.md";
const GITHUB_CHANGELOG = "https://github.com/FittyAr/Certaro/blob/main/CHANGELOG.md";

async function loadChangelog() {
  const container = document.getElementById("changelog-timeline");
  const loading = document.getElementById("changelog-loading");
  try {
    const res = await fetch(CHANGELOG_URL);
    if (!res.ok) throw new Error("fetch failed");
    const md = await res.text();
    const entries = parseChangelog(md);
    if (!entries.length) throw new Error("no entries");
    renderChangelog(entries, container);
  } catch (e) {
    const errEl = document.getElementById("changelog-error");
    if (errEl) errEl.style.display = "block";
  } finally {
    if (loading) loading.style.display = "none";
  }
}

function parseChangelog(md) {
  // Expect: ## [0.1.0] - 2026-09-01  or ## 0.1.0
  const lines = md.split("\n");
  const entries = [];
  let current = null;
  for (const raw of lines) {
    const h2 = raw.match(/^##\s+\[?([^\]]+)\]?\s*-?\s*(.*)/);
    if (h2) {
      if (current) entries.push(current);
      current = { version: h2[1].trim(), date: h2[2].trim().replace(/^[-\s]+/, ""), groups: {}, raw: [] };
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
      current.groups[current._group].push(bullet[1].trim());
    }
  }
  if (current) entries.push(current);
  return entries;
}

function renderChangelog(entries, container) {
  if (!container) return;
  const showAll = entries.length > 3;
  let expanded = false;

  function render(list) {
    container.innerHTML = list.map((e, i) => {
      const isLatest = i === 0;
      const groupsHtml = Object.entries(e.groups).map(([g, items]) => {
        const tag = groupTag(g);
        return `<div class="cl-group"><span class="cl-tag ${tagClass(g)}">${escapeHtml(g)}</span><ul>${items.map(it=>`<li>${escapeHtml(it)}</li>`).join("")}</ul></div>`;
      }).join("");
      return `<article class="cl-entry ${isLatest ? 'cl-latest' : ''}">
        <div class="cl-dot">${String(entries.length - i).padStart(2,"0")}</div>
        <div class="cl-card">
          <div class="cl-card-head">
            <span class="cl-version">${escapeHtml(e.version)}</span>
            ${e.date ? `<span class="cl-date">${escapeHtml(e.date)}</span>` : ""}
            ${isLatest ? `<span class="cl-badge">${t("changelog.latest")}</span>` : ""}
          </div>
          ${groupsHtml || `<p class="cl-empty dim">${escapeHtml(e.raw.join(" "))}</p>`}
        </div>
      </article>`;
    }).join("");

    if (showAll) {
      const btn = document.getElementById("changelog-toggle");
      if (btn) {
        btn.style.display = "inline-flex";
        btn.textContent = expanded ? t("changelog.showLess") : t("changelog.showMore");
        btn.onclick = () => {
          expanded = !expanded;
          render(expanded ? entries : entries.slice(0,3));
          // re-apply translations for button? not needed
        };
      }
    }
  }
  render(entries.slice(0,3));
  // handle locale switch for latest badge
  const origSetLocale = window.setLocale;
  if (origSetLocale && !window._clPatched) {
    window._clPatched = true;
    window.setLocale = function(lang) {
      origSetLocale(lang);
      // re-render to update LATEST badge language
      const isExpanded = container.querySelectorAll(".cl-entry").length > 3;
      render(isExpanded ? entries : entries.slice(0,3));
      applyTranslations();
    };
  }
}

function groupTag(g) { return g.toLowerCase(); }
function tagClass(g) {
  const l = g.toLowerCase();
  if (l.includes("added") || l.includes("agregado")) return "added";
  if (l.includes("fixed") || l.includes("correg")) return "fixed";
  if (l.includes("changed") || l.includes("camb")) return "changed";
  if (l.includes("removed") || l.includes("elimin")) return "removed";
  return "";
}
function escapeHtml(s){ return s.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;"); }

document.addEventListener("DOMContentLoaded", loadChangelog);
