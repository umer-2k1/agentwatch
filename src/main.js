// AgentWatch frontend.
//
// Renders a snapshot. It never computes status — the resolver already did (D-004).
// Polls over IPC as a fallback, and also listens for the emitted `sessions` event so the
// board updates within 500 ms.

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const el = {
  strip: document.getElementById("strip"),
  list: document.getElementById("list"),
  counts: document.getElementById("counts"),
  providers: document.getElementById("providers"),
};

const ORDER = ["needsYou", "dead", "working", "done", "idle", "unknown"];

// Provider → presentation. Brand colour lives in styles.css as --brand-*; this map only
// decides which modifier class to apply, so a new agent needs no JS change beyond one line.
const PROVIDERS = {
  "claude-code": { short: "claude", label: "Claude Code" },
  codex: { short: "codex", label: "Codex" },
  cursor: { short: "cursor", label: "Cursor" },
  "gemini-cli": { short: "gemini", label: "Gemini CLI" },
  "copilot-cli": { short: "copilot", label: "Copilot" },
  opencode: { short: "opencode", label: "OpenCode" },
  grok: { short: "grok", label: "Grok" },
  "qwen-code": { short: "qwen", label: "Qwen" },
  "kilo-code": { short: "kilo", label: "Kilo" },
  cline: { short: "cline", label: "Cline" },
  roo: { short: "roo", label: "Roo" },
  goose: { short: "goose", label: "Goose" },
  droid: { short: "droid", label: "Droid" },
  hermes: { short: "hermes", label: "Hermes" },
  pi: { short: "pi", label: "Pi" },
};

function providerOf(s) {
  return PROVIDERS[s.provider] || { short: "", label: s.provider || "Agent" };
}

let latest = [];
let awaiting = new Set();

function label(s) {
  return s.title || s.repo || "Claude Code";
}

function subtitle(s) {
  return [s.repo, s.branch, s.model].filter(Boolean).join(" · ");
}

function quiet(s) {
  if (s.status === "working" && s.turnAgeS != null) return `${fmtAge(s.turnAgeS)} in turn`;
  if (s.quietForS < 60) return "just now";
  return `${fmtAge(s.quietForS)} ago`;
}

function fmtAge(sec) {
  if (sec < 60) return `${sec}s`;
  if (sec < 3600) return `${Math.floor(sec / 60)}m`;
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  return m ? `${h}h ${m}m` : `${h}h`;
}

function render(sessions, awaitingIds) {
  if (awaitingIds) awaiting = new Set(awaitingIds);
  latest = sessions;

  // Counts in the header.
  const by = {};
  for (const s of sessions) by[s.status] = (by[s.status] || 0) + 1;
  el.counts.innerHTML = ORDER.filter((k) => by[k])
    .map((k) => `<span><b>${by[k]}</b> ${pretty(k)}</span>`)
    .join("");

  // Dot strip. Collapsed when empty, so an idle app looks calm instead of broken.
  el.strip.innerHTML = "";
  el.strip.style.display = sessions.length ? "" : "none";
  for (const s of sessions) {
    const wrap = document.createElement("div");
    wrap.className = "dot-wrap";
    const prov = providerOf(s);
    wrap.title = `${label(s)} — ${pretty(s.status)} · ${prov.label}${s.confidence !== "certain" ? ` (${s.confidence})` : ""}\n${s.evidence?.detail || ""}`;
    wrap.innerHTML = `
      <span class="dot ${s.status} p-${prov.short}" aria-label="${pretty(s.status)}, ${prov.label}"></span>
      <span class="dot-label">${escapeHtml(label(s))}</span>`;
    el.strip.appendChild(wrap);
  }

  // List.
  el.list.innerHTML = "";
  if (!sessions.length) {
    el.list.innerHTML = `
      <div class="empty">
        <div class="big">No agents running</div>
        <div>Start Claude Code, Codex or OpenCode and it appears here.</div>
      </div>`;
    return;
  }

  for (const s of sessions) {
    const row = document.createElement("div");
    row.className = "row";
    row.title = s.evidence?.detail || "";
    const prov = providerOf(s);
    row.innerHTML = `
      <span class="mini dot ${s.status} p-${prov.short}" style="width:7px;height:7px;box-shadow:none"></span>
      <div>
        <div class="title">${escapeHtml(label(s))}</div>
        <div class="meta">
          <span class="chip p-${prov.short}">${escapeHtml(prov.label)}</span>
          <span>${escapeHtml(subtitle(s))}</span>
          <span>${quiet(s)}</span>
          ${s.pid ? `<span>pid ${s.pid}</span>` : ""}
        </div>
      </div>
      <div class="right">
        ${
          awaiting.has(s.id)
            ? `<span class="answer-keys"><button data-allow="${s.id}">1 Allow</button><button data-deny="${s.id}">2 Deny</button></span>`
            : `<span class="pill ${s.status}">${pretty(s.status)}${s.confidence === "unknown" ? " ?" : ""}</span>`
        }
      </div>`;
    el.list.appendChild(row);
  }
}

function pretty(s) {
  return { needsYou: "needs you", working: "working", done: "done", idle: "idle", dead: "dead", unknown: "unknown" }[s] || s;
}

function escapeHtml(v) {
  return String(v ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
}

/** Answer a blocked turn. The optimistic update is deliberate: the helper may already have
 *  timed out, in which case the command returns false and we simply re-render. */
async function decide(sessionId, decision) {
  try {
    await invoke("answer", { sessionId, decision });
  } catch (_) {}
  awaiting.delete(sessionId);
  refresh();
}

async function refresh() {
  try {
    const [sessions, pending] = await Promise.all([invoke("get_sessions"), invoke("awaiting")]);
    render(sessions, pending);
  } catch (_) {}
}

async function renderProviders() {
  try {
    const list = await invoke("get_providers");
    el.providers.innerHTML = list
      .map(
        (p) =>
          `<span class="badge ${p.verification}">${p.verification}</span>${escapeHtml(p.label)}${p.sessions ? ` · ${p.sessions}` : ""}`
      )
      .join("&nbsp;&nbsp;");
  } catch (e) {
    el.providers.textContent = "";
  }
}

/* ------------------------------------------------------------------ *
 * Appearance
 *
 * Everything is a CSS custom property, so a theme change is one attribute write and the
 * whole board follows. Preferences persist in localStorage; M0.3b moves them into the
 * app's config file once the Rust side owns settings.
 * ------------------------------------------------------------------ */

const ACCENTS = ["#0a84ff", "#ff9f0a", "#30d158", "#bf5af2", "#ff375f", "#64d2ff", "#e8e8ed"];

const defaults = { theme: "dark", accent: "#0a84ff", shape: "circle", density: "cozy", glass: 24 };
let prefs = { ...defaults };

function loadPrefs() {
  try {
    prefs = { ...defaults, ...JSON.parse(localStorage.getItem("agentwatch.prefs") || "{}") };
  } catch (_) {
    prefs = { ...defaults };
  }
}

function savePrefs() {
  try {
    localStorage.setItem("agentwatch.prefs", JSON.stringify(prefs));
  } catch (_) {}
}

function resolvedTheme() {
  if (prefs.theme !== "system") return prefs.theme;
  return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

function applyPrefs() {
  const root = document.documentElement;
  root.dataset.theme = resolvedTheme();
  root.dataset.shape = prefs.shape;
  root.dataset.density = prefs.density;
  root.style.setProperty("--accent", prefs.accent);
  root.style.setProperty("--blur-amount", `${prefs.glass}px`);

  // reflect state in the drawer
  document.querySelectorAll('.seg[data-key]').forEach((seg) => {
    const key = seg.dataset.key;
    seg.querySelectorAll("button").forEach((b) =>
      b.classList.toggle("on", b.dataset.val === prefs[key])
    );
  });
  document.querySelectorAll(".swatch").forEach((sw) =>
    sw.classList.toggle("on", sw.dataset.val === prefs.accent)
  );
  const glass = document.getElementById("glass");
  if (glass) glass.value = String(prefs.glass);
}

function buildDrawer() {
  const holder = document.querySelector('.swatches[data-key="accent"]');
  holder.innerHTML = ACCENTS.map(
    (c) =>
      `<button class="swatch" data-val="${c}" style="background:${c}" aria-label="Accent ${c}"></button>`
  ).join("");
}

function wireDrawer() {
  document.getElementById("gear").addEventListener("click", () => {
    document.getElementById("drawer").classList.toggle("open");
  });

  document.querySelectorAll(".seg[data-key]").forEach((seg) => {
    seg.addEventListener("click", (e) => {
      const btn = e.target.closest("button[data-val]");
      if (!btn) return;
      prefs[seg.dataset.key] = btn.dataset.val;
      savePrefs();
      applyPrefs();
    });
  });

  document.querySelector(".swatches").addEventListener("click", (e) => {
    const sw = e.target.closest(".swatch");
    if (!sw) return;
    prefs.accent = sw.dataset.val;
    savePrefs();
    applyPrefs();
  });

  document.getElementById("glass").addEventListener("input", (e) => {
    prefs.glass = Number(e.target.value);
    savePrefs();
    applyPrefs();
  });

  window.matchMedia("(prefers-color-scheme: light)").addEventListener("change", () => {
    if (prefs.theme === "system") applyPrefs();
  });
}

function wireKeys() {
  document.addEventListener("keydown", (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const tag = (e.target && e.target.tagName) || "";
    if (tag === "INPUT" || tag === "TEXTAREA") return;

    const first = [...awaiting][0];
    if (!first) return;
    if (e.key === "1") { e.preventDefault(); decide(first, "allow"); }
    if (e.key === "2") { e.preventDefault(); decide(first, "deny"); }
  });

  document.getElementById("list").addEventListener("click", (e) => {
    const allow = e.target.closest("button[data-allow]");
    const deny = e.target.closest("button[data-deny]");
    if (allow) decide(allow.dataset.allow, "allow");
    if (deny) decide(deny.dataset.deny, "deny");
  });
}

async function boot() {
  loadPrefs();
  buildDrawer();
  wireDrawer();
  applyPrefs();

  await refresh();
  renderProviders();
  wireKeys();

  // Push updates.
  await listen("sessions", (event) => render(event.payload));

  // Safety net if an event is missed.
  setInterval(refresh, 1000);

  setInterval(renderProviders, 5000);
}

boot();
