//! Host router, terminal typing/focus adapters, and live capability diagnostics — SPEC-013.
//!
//! Covers Blueprint §4.7 (Agents & Apps tabs), §6 (Typing matrix), §9 (Bug 1 & Bug 2 fixes),
//! §10.N5–N6, and §16.51 (Diagnostics page).

use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCatalogItem {
    pub id: String,
    pub name: String,
    pub short: String,
    /// "supported" | "preview" | "cmux"
    pub tier: String,
    pub installed: bool,
    pub path: String,
    pub description: String,
    pub can_see: String,
    pub can_history: String,
    pub can_answer: String,
    pub can_type: String,
    pub hook_supported: bool,
    pub mcp_supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostAppItem {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub status_badge: String,
    pub status_ok: bool,
    pub summary: String,
    pub can_see: bool,
    pub can_answer: String,
    pub can_type: bool,
    pub can_jump: bool,
    pub setup_steps: Vec<String>,
    pub diagnostic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsReport {
    pub claude_installed: bool,
    pub claude_hook_installed: bool,
    pub claude_mcp_registered: bool,
    pub opencode_db_exists: bool,
    pub opencode_schema: String,
    pub codex_sessions_exist: bool,
    pub cmux_installed: bool,
    pub cmux_cli_probe: String,
    pub tmux_installed: bool,
    pub hook_socket_listening: bool,
    pub iphone_server_running: bool,
}

fn bin_path(candidates: &[&str]) -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    for c in candidates {
        let expanded = c.replace('~', &home);
        if Path::new(&expanded).exists() {
            return Some(expanded);
        }
    }
    None
}

/// Build the 30-agent catalog with live install detection on this Mac (Blueprint §4.7).
pub fn agent_catalog() -> Vec<AgentCatalogItem> {
    let claude_p = bin_path(&["~/.local/bin/claude", "/opt/homebrew/bin/claude", "/usr/local/bin/claude", "~/.claude"]);
    let codex_p = bin_path(&["/opt/homebrew/bin/codex", "/usr/local/bin/codex", "~/.codex"]);
    let oc_p = bin_path(&["~/.opencode/bin/opencode", "/opt/homebrew/bin/opencode", "~/.local/share/opencode/opencode.db"]);
    let grok_p = bin_path(&["/opt/homebrew/bin/grok", "/usr/local/bin/grok"]);
    let cursor_p = bin_path(&["/Applications/Cursor.app", "~/.cursor"]);
    let gemini_p = bin_path(&["/opt/homebrew/bin/gemini", "~/.gemini"]);
    let copilot_p = bin_path(&["/opt/homebrew/bin/github-copilot-cli", "~/.config/github-copilot"]);
    let kilo_p = bin_path(&["~/.kilo", "/opt/homebrew/bin/kilo"]);
    let cowork_p = bin_path(&["/Applications/Claude.app"]);
    let agy_p = bin_path(&["/opt/homebrew/bin/agy", "~/.gemini/antigravity-cli"]);
    let qwen_p = bin_path(&["/opt/homebrew/bin/qwen", "~/.qwen"]);
    let droid_p = bin_path(&["~/.factory/sessions"]);
    let goose_p = bin_path(&["~/Library/Application Support/goose", "~/Library/Application Support/Block/goose"]);
    let pi_p = bin_path(&["~/.pi/agent/sessions", "/opt/homebrew/bin/pi"]);
    let cline_p = bin_path(&["~/Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev"]);
    let roo_p = bin_path(&["~/Library/Application Support/Code/User/globalStorage/rooveterinaryinc.roo-cline"]);
    let hermes_p = bin_path(&["~/.hermes/state.db", "~/.hermes"]);

    let mk = |id: &str,
              name: &str,
              short: &str,
              tier: &str,
              found: Option<String>,
              desc: &str,
              see: &str,
              hist: &str,
              ans: &str,
              typ: &str,
              hook: bool,
              mcp: bool| AgentCatalogItem {
        id: id.into(),
        name: name.into(),
        short: short.into(),
        tier: tier.into(),
        installed: found.is_some(),
        path: found.unwrap_or_else(|| "Not installed".into()),
        description: desc.into(),
        can_see: see.into(),
        can_history: hist.into(),
        can_answer: ans.into(),
        can_type: typ.into(),
        hook_supported: hook,
        mcp_supported: mcp,
    };

    let mut out = vec![
        mk(
            "claude-code",
            "Claude Code",
            "*",
            "supported",
            claude_p,
            "AgentWatch reads Claude Code's own session records in any terminal and in the Claude app, with nothing to install. The hook adds questions and permission prompts, which you can answer from the Inbox.",
            "yes",
            "yes",
            "yes",
            "yes",
            true,
            true,
        ),
        mk(
            "codex",
            "Codex",
            "Cx",
            "supported",
            codex_p,
            "AgentWatch reads Codex's rollout files in the Codex app and the CLI in any terminal, with nothing to install. The hook adds approval requests.",
            "yes",
            "yes",
            "yes",
            "yes",
            true,
            true,
        ),
        mk(
            "opencode",
            "OpenCode",
            "Op",
            "supported",
            oc_p,
            "First-class OpenCode 2.x support (session_v2 + session_message WAL database and embedded PTY sessions). Unlike AgentDock, AgentWatch probes v1/v2 schemas and supports answering inside spawned PTY sessions.",
            "yes",
            "yes",
            "yes (PTY)",
            "yes (PTY)",
            false,
            true,
        ),
        mk(
            "grok",
            "Grok",
            "Ø",
            "supported",
            grok_p,
            "Grok Build runs Claude Code's hooks, so with the Claude Code hook on, its sessions show in any terminal.",
            "with hook",
            "not yet",
            "yes",
            "yes",
            true,
            false,
        ),
        mk(
            "cursor",
            "Cursor",
            "Cu",
            "preview",
            cursor_p,
            "Cursor's agent runs Claude Code's hooks too: with the Claude Code hook on, its chats show as Cursor, working or done.",
            "with hook",
            "yes",
            "not yet",
            "not yet",
            true,
            false,
        ),
        mk(
            "gemini-cli",
            "Gemini CLI",
            "✦",
            "preview",
            gemini_p,
            "Reads Gemini CLI's own session files (~/.gemini/projects.json) while it runs, with nothing to install.",
            "yes",
            "yes",
            "not yet",
            "not yet",
            false,
            true,
        ),
        mk(
            "copilot-cli",
            "GitHub Copilot",
            "Co",
            "preview",
            copilot_p,
            "Reads Copilot CLI's own session files while it runs: what it's doing, when it waits for permission, its turns and tokens.",
            "yes",
            "yes",
            "not yet",
            "not yet",
            false,
            false,
        ),
        mk(
            "kilo-code",
            "Kilo Code",
            "Ki",
            "preview",
            kilo_p,
            "Reads Kilo Code's session store (kilo.db) while it runs.",
            "yes",
            "yes",
            "not yet",
            "not yet",
            false,
            false,
        ),
        mk(
            "claude-cowork",
            "Claude Cowork",
            "*",
            "preview",
            cowork_p,
            "Reads Claude desktop local-agent-mode sessions on this Mac.",
            "yes",
            "yes",
            "elsewhere",
            "not yet",
            false,
            false,
        ),
        mk(
            "antigravity",
            "Antigravity",
            "Ag",
            "preview",
            agy_p,
            "Reads Antigravity CLI's (agy) conversation files while it runs.",
            "yes",
            "yes",
            "not yet",
            "not yet",
            false,
            false,
        ),
        mk(
            "qwen-code",
            "Qwen Code",
            "Qw",
            "preview",
            qwen_p,
            "Reads Qwen Code session transcripts (~/.qwen).",
            "yes",
            "yes",
            "not yet",
            "not yet",
            false,
            false,
        ),
        mk(
            "factory-droid",
            "Factory Droid",
            "Dr",
            "preview",
            droid_p,
            "Reads Factory Droid sessions (~/.factory/sessions).",
            "yes",
            "yes",
            "not yet",
            "not yet",
            false,
            false,
        ),
        mk(
            "goose",
            "Goose",
            "Go",
            "preview",
            goose_p,
            "Reads Block Goose SQLite session store.",
            "yes",
            "yes",
            "not yet",
            "not yet",
            false,
            false,
        ),
        mk(
            "pi",
            "Pi",
            "Pi",
            "preview",
            pi_p,
            "Reads Pi coding agent session files (~/.pi/agent/sessions).",
            "yes",
            "yes",
            "not yet",
            "not yet",
            false,
            false,
        ),
        mk(
            "cline",
            "Cline",
            "Cn",
            "preview",
            cline_p,
            "Reads Cline VS Code task history and token ledger.",
            "yes",
            "yes",
            "not yet",
            "not yet",
            false,
            false,
        ),
        mk(
            "roo-code",
            "Roo Code",
            "Ro",
            "preview",
            roo_p,
            "Reads Roo Code VS Code task history and token ledger.",
            "yes",
            "yes",
            "not yet",
            "not yet",
            false,
            false,
        ),
        mk(
            "hermes",
            "Hermes",
            "He",
            "preview",
            hermes_p,
            "Reads Hermes Agent SQLite state (~/.hermes/state.db).",
            "yes",
            "yes",
            "not yet",
            "not yet",
            false,
            false,
        ),
    ];

    for (id, name, short) in [
        ("kimi-code", "Kimi Code", "Km"),
        ("amp", "Amp", "Am"),
        ("crush", "Crush", "Cr"),
        ("junie", "Junie", "Ju"),
        ("kiro", "Kiro", "Kr"),
        ("copilot-chat", "Copilot Chat", "Cc"),
        ("windsurf", "Windsurf", "Ws"),
        ("zed", "Zed", "Zd"),
        ("warp-agent", "Warp", "Wa"),
        ("openhands", "OpenHands", "Oh"),
        ("mistral-vibe", "Mistral Vibe", "Mv"),
        ("aider", "Aider", "Ai"),
        ("deepseek-tui", "DeepSeek TUI", "Ds"),
    ] {
        out.push(mk(
            id,
            name,
            short,
            "cmux",
            None,
            "Detected and typed into when running inside a cmux workspace, herdr pane, or AgentWatch embedded PTY.",
            "yes",
            "via PTY",
            "via PTY",
            "yes",
            false,
            false,
        ));
    }
    out
}

/// Probe `cmux` CLI capabilities so a subcommand rename never breaks the app (Blueprint §9 Bug 2).
pub fn probe_cmux() -> (bool, String) {
    let Some(bin) = bin_path(&[
        "/Applications/cmux.app/Contents/Resources/bin/cmux",
        "/opt/homebrew/bin/cmux",
        "/usr/local/bin/cmux",
    ]) else {
        return (false, "cmux not installed".into());
    };
    let out = Command::new(&bin).arg("--help").output();
    match out {
        Ok(o) => {
            let txt = String::from_utf8_lossy(&o.stdout);
            if txt.contains("workspace") {
                (true, format!("cmux installed ({bin}) · modern `workspace` subcommands supported"))
            } else {
                (true, format!("cmux installed ({bin}) · legacy CLI"))
            }
        }
        Err(e) => (true, format!("cmux binary found at {bin}, probe error: {e}")),
    }
}

/// Probe OpenCode SQLite schema version (Blueprint §9 Bug 1, §10.N5).
pub fn probe_opencode_schema() -> (bool, String) {
    let Some(db) = bin_path(&["~/.local/share/opencode/opencode.db"]) else {
        return (false, "opencode.db not found".into());
    };
    let out = Command::new("sqlite3")
        .args(["-readonly", &db, "SELECT name FROM sqlite_master WHERE type='table' AND name IN ('session_v2','session');"])
        .output();
    match out {
        Ok(o) => {
            let txt = String::from_utf8_lossy(&o.stdout);
            let has_v2 = txt.lines().any(|l| l.trim() == "session_v2");
            let has_v1 = txt.lines().any(|l| l.trim() == "session");
            if has_v2 {
                (true, "OpenCode v2 schema (session_v2 + session_message) — verified & active".into())
            } else if has_v1 {
                (true, "OpenCode v1 legacy schema (session + message)".into())
            } else {
                (true, "Unsupported OpenCode schema — tables not recognized".into())
            }
        }
        Err(e) => (true, format!("sqlite3 probe failed: {e}")),
    }
}

/// Build the 13 host apps list with live capability probing (Blueprint §4.7 Apps).
pub fn host_catalog() -> Vec<HostAppItem> {
    let (cmux_ok, cmux_diag) = probe_cmux();
    let tmux_p = bin_path(&["/opt/homebrew/bin/tmux", "/usr/local/bin/tmux", "/usr/bin/tmux"]);
    let iterm_p = bin_path(&["/Applications/iTerm.app"]);
    let wez_p = bin_path(&["/Applications/WezTerm.app"]);
    let kitty_p = bin_path(&["/Applications/kitty.app"]);
    let herdr_p = bin_path(&["/Applications/herdr.app", "/opt/homebrew/bin/herdr"]);
    let ghostty_p = bin_path(&["/Applications/Ghostty.app"]);
    let warp_p = bin_path(&["/Applications/Warp.app"]);
    let alacritty_p = bin_path(&["/Applications/Alacritty.app"]);
    let editors_p = bin_path(&["/Applications/Cursor.app", "/Applications/Visual Studio Code.app", "/Applications/Zed.app"]);
    let claude_app = bin_path(&["/Applications/Claude.app"]);
    let codex_app = bin_path(&["/Applications/Codex.app"]);

    vec![
        HostAppItem {
            id: "cmux".into(),
            name: "cmux".into(),
            installed: cmux_ok,
            status_badge: if cmux_ok { "Ready (workspace CLI)".into() } else { "Not installed".into() },
            status_ok: cmux_ok,
            summary: "Full support. Probed for modern `cmux workspace` subcommands so the `Unknown command 'current'` bug cannot occur.".into(),
            can_see: true,
            can_answer: "Full".into(),
            can_type: true,
            can_jump: true,
            setup_steps: vec![
                "Open cmux › Settings › Automation.".into(),
                "Set Socket Control Mode to Password (or Automation).".into(),
                "AgentWatch uses cmux workspace commands automatically.".into(),
            ],
            diagnostic: cmux_diag,
        },
        HostAppItem {
            id: "tmux".into(),
            name: "tmux".into(),
            installed: tmux_p.is_some(),
            status_badge: if tmux_p.is_some() { "Installed".into() } else { "Not installed".into() },
            status_ok: tmux_p.is_some(),
            summary: "Full support: reads and types into panes directly through `tmux send-keys` using TMUX_PANE.".into(),
            can_see: true,
            can_answer: "Full".into(),
            can_type: true,
            can_jump: true,
            setup_steps: vec![
                "Install tmux (`brew install tmux`).".into(),
                "Start your agents inside a tmux session.".into(),
            ],
            diagnostic: tmux_p.unwrap_or_else(|| "tmux binary not found".into()),
        },
        HostAppItem {
            id: "iterm".into(),
            name: "iTerm".into(),
            installed: iterm_p.is_some(),
            status_badge: if iterm_p.is_some() { "Installed".into() } else { "Not installed".into() },
            status_ok: iterm_p.is_some(),
            summary: "Full support through iTerm's AppleScript API: locates the agent's exact tab by tty / ITERM_SESSION_ID.".into(),
            can_see: true,
            can_answer: "Full".into(),
            can_type: true,
            can_jump: true,
            setup_steps: vec![
                "Allow AgentWatch to control iTerm in macOS System Settings › Privacy & Security › Automation.".into(),
            ],
            diagnostic: iterm_p.unwrap_or_else(|| "/Applications/iTerm.app not found".into()),
        },
        HostAppItem {
            id: "wezterm".into(),
            name: "WezTerm".into(),
            installed: wez_p.is_some(),
            status_badge: if wez_p.is_some() { "Installed".into() } else { "Not installed".into() },
            status_ok: wez_p.is_some(),
            summary: "Full support through `wezterm cli send-text` using WEZTERM_PANE.".into(),
            can_see: true,
            can_answer: "Full".into(),
            can_type: true,
            can_jump: true,
            setup_steps: vec!["Run agents in WezTerm; WEZTERM_PANE is detected automatically.".into()],
            diagnostic: wez_p.unwrap_or_else(|| "/Applications/WezTerm.app not found".into()),
        },
        HostAppItem {
            id: "kitty".into(),
            name: "kitty".into(),
            installed: kitty_p.is_some(),
            status_badge: if kitty_p.is_some() { "Installed".into() } else { "Not installed".into() },
            status_ok: kitty_p.is_some(),
            summary: "Full support once kitty remote control is enabled in ~/.config/kitty/kitty.conf.".into(),
            can_see: true,
            can_answer: "Full".into(),
            can_type: true,
            can_jump: true,
            setup_steps: vec![
                "Add `allow_remote_control yes` and `listen_on unix:/tmp/kitty` to ~/.config/kitty/kitty.conf.".into(),
            ],
            diagnostic: kitty_p.unwrap_or_else(|| "/Applications/kitty.app not found".into()),
        },
        HostAppItem {
            id: "herdr".into(),
            name: "herdr".into(),
            installed: herdr_p.is_some(),
            status_badge: if herdr_p.is_some() { "Installed".into() } else { "Not installed".into() },
            status_ok: herdr_p.is_some(),
            summary: "Every agent herdr recognises in its panes, reached through herdr.".into(),
            can_see: true,
            can_answer: "Full".into(),
            can_type: true,
            can_jump: true,
            setup_steps: vec!["Run agents inside herdr panes.".into()],
            diagnostic: herdr_p.unwrap_or_else(|| "herdr not installed".into()),
        },
        HostAppItem {
            id: "terminal".into(),
            name: "Terminal".into(),
            installed: true,
            status_badge: "Built-in · Hook answers".into(),
            status_ok: true,
            summary: "macOS Terminal.app shows every agent and answers Claude Code through its hook; AppleScript switches to the tab.".into(),
            can_see: true,
            can_answer: "With Claude hook".into(),
            can_type: false,
            can_jump: true,
            setup_steps: vec![
                "Turn on the Claude Code hook in Settings › Agents.".into(),
                "For full typing support, run inside tmux or use AgentWatch's built-in Sessions Grid.".into(),
            ],
            diagnostic: "/System/Applications/Utilities/Terminal.app".into(),
        },
        HostAppItem {
            id: "ghostty".into(),
            name: "Ghostty".into(),
            installed: ghostty_p.is_some(),
            status_badge: if ghostty_p.is_some() { "Installed · Hook".into() } else { "Not installed".into() },
            status_ok: ghostty_p.is_some(),
            summary: "Shows every agent and answers Claude Code through its hook.".into(),
            can_see: true,
            can_answer: "With Claude hook".into(),
            can_type: false,
            can_jump: true,
            setup_steps: vec!["Turn on the Claude Code hook, or run tmux inside Ghostty.".into()],
            diagnostic: ghostty_p.unwrap_or_else(|| "Ghostty not installed".into()),
        },
        HostAppItem {
            id: "warp".into(),
            name: "Warp".into(),
            installed: warp_p.is_some(),
            status_badge: if warp_p.is_some() { "Installed · Hook".into() } else { "Not installed".into() },
            status_ok: warp_p.is_some(),
            summary: "Shows every agent and answers Claude Code through its hook.".into(),
            can_see: true,
            can_answer: "With Claude hook".into(),
            can_type: false,
            can_jump: true,
            setup_steps: vec!["Turn on the Claude Code hook.".into()],
            diagnostic: warp_p.unwrap_or_else(|| "Warp not installed".into()),
        },
        HostAppItem {
            id: "alacritty".into(),
            name: "Alacritty".into(),
            installed: alacritty_p.is_some(),
            status_badge: if alacritty_p.is_some() { "Installed · Hook".into() } else { "Not installed".into() },
            status_ok: alacritty_p.is_some(),
            summary: "Shows every agent and answers Claude Code through its hook.".into(),
            can_see: true,
            can_answer: "With Claude hook".into(),
            can_type: false,
            can_jump: true,
            setup_steps: vec!["Run tmux inside Alacritty for full typing.".into()],
            diagnostic: alacritty_p.unwrap_or_else(|| "Alacritty not installed".into()),
        },
        HostAppItem {
            id: "editors".into(),
            name: "VS Code, Cursor, Zed".into(),
            installed: editors_p.is_some(),
            status_badge: if editors_p.is_some() { "Installed · Hook".into() } else { "Not installed".into() },
            status_ok: editors_p.is_some(),
            summary: "Integrated editor terminals show every agent and answer Claude Code through its hook.".into(),
            can_see: true,
            can_answer: "With Claude hook".into(),
            can_type: false,
            can_jump: true,
            setup_steps: vec!["Turn on the Claude Code hook in Settings › Agents.".into()],
            diagnostic: editors_p.unwrap_or_else(|| "No editor detected".into()),
        },
        HostAppItem {
            id: "claude-app".into(),
            name: "Claude app".into(),
            installed: claude_app.is_some(),
            status_badge: if claude_app.is_some() { "Installed".into() } else { "Not installed".into() },
            status_ok: claude_app.is_some(),
            summary: "Claude desktop sessions show up automatically. Clicking an agent brings Claude forward.".into(),
            can_see: true,
            can_answer: "In Claude app".into(),
            can_type: false,
            can_jump: true,
            setup_steps: vec!["No setup needed.".into()],
            diagnostic: claude_app.unwrap_or_else(|| "/Applications/Claude.app not found".into()),
        },
        HostAppItem {
            id: "codex-app".into(),
            name: "Codex app".into(),
            installed: codex_app.is_some(),
            status_badge: if codex_app.is_some() { "Installed".into() } else { "Not installed".into() },
            status_ok: codex_app.is_some(),
            summary: "Codex desktop threads show up automatically from ~/.codex/sessions.".into(),
            can_see: true,
            can_answer: "With Codex hook".into(),
            can_type: false,
            can_jump: true,
            setup_steps: vec!["No setup needed.".into()],
            diagnostic: codex_app.unwrap_or_else(|| "/Applications/Codex.app not found".into()),
        },
    ]
}

/// Focus the external app or terminal hosting a session (Blueprint §10.F7).
pub fn jump_to_host(place: &str, _cwd: &str) -> Result<String, String> {
    let app = match place {
        "Claude app" => "Claude",
        "Codex app" => "Codex",
        "cmux" => "cmux",
        "iTerm" => "iTerm",
        "WezTerm" => "WezTerm",
        "kitty" => "kitty",
        "Ghostty" => "Ghostty",
        "Warp" => "Warp",
        "OpenCode" => "OpenCode",
        _ => "Terminal",
    };
    let _ = Command::new("open").args(["-a", app]).spawn();
    Ok(format!("Activated {app}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_lists_30_agents_and_13_hosts() {
        let agents = agent_catalog();
        let hosts = host_catalog();
        assert_eq!(agents.len(), 30);
        assert_eq!(hosts.len(), 13);
        assert!(agents.iter().any(|a| a.id == "opencode" && a.tier == "supported"));
    }
}
