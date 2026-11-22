//! MCP (Model Context Protocol) JSON-RPC 2.0 server and `~/.claude.json` registration — SPEC-012.
//!
//! Covers Blueprint §4.9, §10.F9, §16.45, and §19.2.
//! Exposes 8 tools over `~/Library/Application Support/AgentWatch/mcp.sock`:
//! `list_agents`, `get_agent`, `who_edits`, `recent_turns`, `send_message`, `start_agent`,
//! `request_lock`, and `release_lock`.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::thread;

use serde_json::{json, Value};

use crate::store::{now_ms, SharedStore};

pub fn mcp_socket_path(data_dir: &Path) -> PathBuf {
    data_dir.join("mcp.sock")
}

pub fn spawn(data_dir: &Path, store: SharedStore) -> std::io::Result<PathBuf> {
    fs::create_dir_all(data_dir)?;
    let path = mcp_socket_path(data_dir);
    if path.exists() {
        let _ = fs::remove_file(&path);
    }
    let listener = UnixListener::bind(&path)?;
    let _ = fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o600));

    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let st = store.clone();
            thread::spawn(move || {
                let mut reader = BufReader::new(&stream);
                let mut writer = &stream;
                let mut line = String::new();
                while reader.read_line(&mut line).unwrap_or(0) > 0 {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        if let Some(resp) = handle_jsonrpc(trimmed, &st) {
                            let mut out = resp.to_string();
                            out.push('\n');
                            let _ = writer.write_all(out.as_bytes());
                            let _ = writer.flush();
                        }
                    }
                    line.clear();
                }
            });
        }
    });

    Ok(path)
}

/// Process one JSON-RPC 2.0 request and return the response (`None` for notifications).
pub fn handle_jsonrpc(raw: &str, store: &SharedStore) -> Option<Value> {
    let req: Value = serde_json::from_str(raw).ok()?;
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or_else(|| json!({}));

    if method.starts_with("notifications/") {
        return None;
    }

    let id = id.unwrap_or(Value::Null);

    let result = match method {
        "initialize" => json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "agentwatch", "version": "0.2.0" },
            "instructions": "Watch coding agents on this Mac. Call who_edits before editing shared files; use request_lock to claim a file; send_message types into or queues a message for another agent. Nothing leaves this Mac."
        }),
        "tools/list" => json!({
            "tools": [
                {
                    "name": "list_agents",
                    "description": "Every coding agent AgentWatch sees on this Mac: id, name, kind, status (working, needsYou, done, idle, dead), folder, repository, branch, place, model, turn start, and queued count.",
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "get_agent",
                    "description": "One agent in full, by its id from list_agents: includes files changed in the last 20 minutes, queued messages, and any collision with another agent.",
                    "inputSchema": {
                        "type": "object",
                        "properties": { "id": { "type": "string" } },
                        "required": ["id"]
                    }
                },
                {
                    "name": "who_edits",
                    "description": "The agents that changed a file or folder in the last 20 minutes, active collisions, and advisory file locks. Call before editing shared files.",
                    "inputSchema": {
                        "type": "object",
                        "properties": { "path": { "type": "string" } },
                        "required": ["path"]
                    }
                },
                {
                    "name": "recent_turns",
                    "description": "Recent finished turns across your agents with duration and last words.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "limit": { "type": "integer" },
                            "folder": { "type": "string" },
                            "agent": { "type": "string" }
                        }
                    }
                },
                {
                    "name": "send_message",
                    "description": "Send or queue a message for another agent.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "agent_id": { "type": "string" },
                            "text": { "type": "string" },
                            "when_done": { "type": "boolean" }
                        },
                        "required": ["agent_id", "text"]
                    }
                },
                {
                    "name": "start_agent",
                    "description": "Start a new coding agent (claude-code, opencode, codex, shell) in a folder, optionally in an isolated git worktree.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "kind": { "type": "string" },
                            "folder": { "type": "string" },
                            "prompt": { "type": "string" },
                            "worktree": { "type": "boolean" }
                        },
                        "required": ["kind", "folder", "prompt"]
                    }
                },
                {
                    "name": "request_lock",
                    "description": "Acquire an advisory file lock before editing a shared file.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "agent_id": { "type": "string" },
                            "path": { "type": "string" },
                            "ttl_seconds": { "type": "integer" }
                        },
                        "required": ["agent_id", "path"]
                    }
                },
                {
                    "name": "release_lock",
                    "description": "Release an advisory file lock.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "agent_id": { "type": "string" },
                            "path": { "type": "string" }
                        },
                        "required": ["agent_id", "path"]
                    }
                }
            ]
        }),
        "tools/call" => {
            let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
            call_tool(name, &args, store)
        }
        _ => json!({}),
    };

    Some(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    }))
}

fn call_tool(name: &str, args: &Value, store: &SharedStore) -> Value {
    let Ok(mut st) = store.lock() else {
        return tool_err("store lock unavailable");
    };

    match name {
        "list_agents" => {
            let snap = st.snapshot();
            let agents: Vec<Value> = snap
                .iter()
                .map(|s| {
                    json!({
                        "id": s.fact.id,
                        "agent": s.fact.provider,
                        "kind": s.fact.provider,
                        "name": s.fact.title,
                        "status": s.status.as_str(),
                        "folder": s.fact.cwd,
                        "place": s.fact.place,
                        "model": s.fact.model,
                        "turn_started_at": s.fact.turn_started_ms,
                        "last_activity_at": s.fact.last_activity_ms,
                        "queued": s.queued_messages.len()
                    })
                })
                .collect();
            tool_ok(json!({ "count": agents.len(), "agents": agents }))
        }
        "get_agent" => {
            let id = args.get("id").and_then(|s| s.as_str()).unwrap_or("");
            let snap = st.snapshot();
            let Some(s) = snap.iter().find(|x| x.fact.id == id) else {
                return tool_err(&format!("No agent with id {id}. Call list_agents."));
            };
            tool_ok(json!({
                "id": s.fact.id,
                "name": s.fact.title,
                "kind": s.fact.provider,
                "status": s.status.as_str(),
                "folder": s.fact.cwd,
                "repository": s.fact.repo,
                "branch": s.fact.branch,
                "place": s.fact.place,
                "model": s.fact.model,
                "edited_files": s.fact.edited_files,
                "queued_messages": s.queued_messages,
                "collision": s.collisions
            }))
        }
        "who_edits" => {
            let path = args.get("path").and_then(|s| s.as_str()).unwrap_or("");
            let rep = st.who_edits(path);
            tool_ok(serde_json::to_value(rep).unwrap_or_default())
        }
        "recent_turns" => {
            let limit = args
                .get("limit")
                .and_then(|n| n.as_u64())
                .unwrap_or(10)
                .min(50) as usize;
            let turns: Vec<_> = st.history_turns().into_iter().take(limit).collect();
            tool_ok(json!({ "turns": turns }))
        }
        "send_message" => {
            let agent_id = args.get("agent_id").and_then(|s| s.as_str()).unwrap_or("");
            let text = args.get("text").and_then(|s| s.as_str()).unwrap_or("");
            let when_done = args.get("when_done").and_then(|b| b.as_bool()).unwrap_or(false);
            match st.send_or_queue_message(agent_id, text, when_done) {
                Ok(msg) => tool_ok(json!({ "status": "ok", "detail": msg })),
                Err(e) => tool_err(&e),
            }
        }
        "start_agent" => {
            let kind = args.get("kind").and_then(|s| s.as_str()).unwrap_or("claude-code");
            let folder = args.get("folder").and_then(|s| s.as_str()).unwrap_or("");
            let prompt = args.get("prompt").and_then(|s| s.as_str()).unwrap_or("");
            let wt = args.get("worktree").and_then(|b| b.as_bool()).unwrap_or(false);
            match st.pty.spawn(kind, folder, prompt, wt, now_ms()) {
                Ok(v) => tool_ok(json!({ "status": "started", "id": v.id, "kind": v.kind, "folder": v.folder })),
                Err(e) => tool_err(&e),
            }
        }
        "request_lock" => {
            let agent_id = args.get("agent_id").and_then(|s| s.as_str()).unwrap_or("");
            let path = args.get("path").and_then(|s| s.as_str()).unwrap_or("");
            let ttl = args.get("ttl_seconds").and_then(|n| n.as_i64()).unwrap_or(300);
            let lock = st.request_lock(agent_id, path, ttl);
            tool_ok(serde_json::to_value(lock).unwrap_or_default())
        }
        "release_lock" => {
            let agent_id = args.get("agent_id").and_then(|s| s.as_str()).unwrap_or("");
            let path = args.get("path").and_then(|s| s.as_str()).unwrap_or("");
            let ok = st.release_lock(agent_id, path);
            tool_ok(json!({ "released": ok }))
        }
        _ => tool_err(&format!("Unknown tool {name}")),
    }
}

fn tool_ok(payload: Value) -> Value {
    json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&payload).unwrap_or_default() }],
        "isError": false
    })
}

fn tool_err(msg: &str) -> Value {
    json!({
        "content": [{ "type": "text", "text": msg }],
        "isError": true
    })
}

/// Check or toggle `mcpServers.agentwatch` in `~/.claude.json` (Blueprint §2 #17, §19.1).
pub fn claude_mcp_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".claude.json")
}

pub fn is_mcp_registered(path: &Path) -> bool {
    let Ok(txt) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(v) = serde_json::from_str::<Value>(&txt) else {
        return false;
    };
    v.get("mcpServers")
        .and_then(|m| m.get("agentwatch"))
        .is_some()
}

pub fn set_mcp_registered(path: &Path, enable: bool, mcp_bin: &str) -> Result<bool, String> {
    let mut root: Value = if path.exists() {
        let txt = fs::read_to_string(path).map_err(|e| e.to_string())?;
        if txt.trim().is_empty() {
            json!({})
        } else {
            serde_json::from_str(&txt).map_err(|e| e.to_string())?
        }
    } else {
        json!({})
    };

    if root.get("mcpServers").is_none() {
        root["mcpServers"] = json!({});
    }
    let servers = root
        .get_mut("mcpServers")
        .and_then(|m| m.as_object_mut())
        .ok_or("mcpServers is not an object")?;

    if enable {
        servers.insert("agentwatch".into(), json!({ "command": mcp_bin }));
    } else {
        servers.remove("agentwatch");
    }

    let pretty = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    fs::write(path, pretty + "\n").map_err(|e| e.to_string())?;
    Ok(enable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;
    use std::sync::{Arc, Mutex};

    #[test]
    fn mcp_tools_list_exposes_all_8_tools() {
        let st = Arc::new(Mutex::new(Store::with_window(1)));
        let resp = handle_jsonrpc(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#, &st).unwrap();
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 8);
    }

    #[test]
    fn mcp_lock_acquire_and_release_roundtrips() {
        let st = Arc::new(Mutex::new(Store::with_window(1)));
        let r1 = handle_jsonrpc(
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"request_lock","arguments":{"agent_id":"a1","path":"/repo/auth.rs"}}}"#,
            &st,
        )
        .unwrap();
        assert_eq!(r1["result"]["isError"], false);

        let r2 = handle_jsonrpc(
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"who_edits","arguments":{"path":"/repo"}}}"#,
            &st,
        )
        .unwrap();
        let txt = r2["result"]["content"][0]["text"].as_str().unwrap();
        assert!(txt.contains("/repo/auth.rs"));
    }
}
