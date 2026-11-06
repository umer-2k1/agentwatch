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

function render(sessions) {
  latest = sessions;

  // Counts in the header.
  const by = {};
  for (const s of sessions) by[s.status] = (by[s.status] || 0) + 1;
  el.counts.innerHTML = ORDER.filter((k) => by[k])
    .map((k) => `<span><b>${by[k]}</b> ${pretty(k)}</span>`)
    .join("");

  // Dot strip.
  el.strip.innerHTML = "";
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
        <div>Start a Claude Code session in any terminal and it appears here.</div>
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
      <span class="pill ${s.status}">${pretty(s.status)}${s.confidence === "unknown" ? " ?" : ""}</span>`;
    el.list.appendChild(row);
  }
}

function pretty(s) {
  return { needsYou: "needs you", working: "working", done: "done", idle: "idle", dead: "dead", unknown: "unknown" }[s] || s;
}

function escapeHtml(v) {
  return String(v ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
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

async function boot() {
  render(await invoke("get_sessions"));
  renderProviders();

  // Push updates.
  await listen("sessions", (event) => render(event.payload));

  // Safety net if an event is missed.
  setInterval(async () => {
    try {
      render(await invoke("get_sessions"));
    } catch (_) {}
  }, 2000);

  setInterval(renderProviders, 5000);
}

boot();
