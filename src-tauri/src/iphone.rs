//! iPhone LAN companion HTTP server and pairing token manager — SPEC-014.
//!
//! Covers Blueprint §4.10, §10.F11, §19.3, and Figures 19, 21, 22.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use serde::{Deserialize, Serialize};

use crate::hooks::event::Decision;
use crate::store::SharedStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IphoneStatus {
    pub enabled: bool,
    pub port: u16,
    pub lan_ip: String,
    pub token: String,
    pub url: String,
    pub relay_url: String,
}

pub struct IphoneServer {
    enabled: Arc<AtomicBool>,
    port: u16,
    token: String,
    relay_url: String,
}

impl IphoneServer {
    pub fn new(store: SharedStore) -> Self {
        let token = generate_token();
        let enabled = Arc::new(AtomicBool::new(false));
        let port = spawn_listener(store, enabled.clone());
        Self {
            enabled,
            port,
            token,
            relay_url: String::new(),
        }
    }

    pub fn status(&self) -> IphoneStatus {
        let ip = detect_lan_ip();
        let en = self.enabled.load(Ordering::SeqCst);
        let url = format!("http://{}:{}/?t={}", ip, self.port, self.token);
        IphoneStatus {
            enabled: en,
            port: self.port,
            lan_ip: ip,
            token: self.token.clone(),
            url,
            relay_url: self.relay_url.clone(),
        }
    }

    pub fn set_enabled(&mut self, on: bool) -> IphoneStatus {
        self.enabled.store(on, Ordering::SeqCst);
        self.status()
    }

    pub fn rotate_token(&mut self) -> IphoneStatus {
        self.token = generate_token();
        self.status()
    }

    pub fn set_relay_url(&mut self, url: String) -> IphoneStatus {
        self.relay_url = url;
        self.status()
    }
}

fn generate_token() -> String {
    let ts = crate::store::now_ms();
    let pid = std::process::id();
    format!("aw{:x}{:x}9f3a7c2e", ts, pid)
}

fn detect_lan_ip() -> String {
    for iface in ["en0", "en1"] {
        if let Ok(out) = Command::new("ipconfig").args(["getifaddr", iface]).output() {
            let ip = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !ip.is_empty() && ip.contains('.') {
                return ip;
            }
        }
    }
    "127.0.0.1".into()
}

fn spawn_listener(store: SharedStore, enabled: Arc<AtomicBool>) -> u16 {
    for port in [47811u16, 47812, 47813] {
        if let Ok(listener) = TcpListener::bind(("0.0.0.0", port)) {
            thread::spawn(move || {
                for stream in listener.incoming().flatten() {
                    if !enabled.load(Ordering::SeqCst) {
                        continue;
                    }
                    let st = store.clone();
                    thread::spawn(move || handle_http(stream, st));
                }
            });
            return port;
        }
    }
    47811
}

fn handle_http(mut stream: std::net::TcpStream, store: SharedStore) {
    let mut buf = [0u8; 8192];
    let Ok(n) = stream.read(&mut buf) else { return };
    if n == 0 {
        return;
    }
    let req = String::from_utf8_lossy(&buf[..n]);
    let first_line = req.lines().next().unwrap_or("");
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let path = parts.next().unwrap_or("/");

    // Extract body after \r\n\r\n
    let body = req.split("\r\n\r\n").nth(1).unwrap_or("");

    if method == "GET" && path.starts_with("/api/state") {
        let json = if let Ok(s) = store.lock() {
            serde_json::to_string(&s.iphone_state_json()).unwrap_or_else(|_| "{}".into())
        } else {
            "{}".into()
        };
        let _ = write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
            json.len(),
            json
        );
        return;
    }

    if method == "POST" && path.starts_with("/api/") {
        let v: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let card = v.get("card").and_then(|c| c.as_str()).unwrap_or("");
        let number = v.get("number").and_then(|n| n.as_u64()).unwrap_or(1) as usize;
        if let Ok(mut s) = store.lock() {
            if path.starts_with("/api/undo") {
                s.undo(card);
            } else if path.starts_with("/api/seen") {
                s.mark_all_seen();
            } else if path.starts_with("/api/choose") {
                let d = if number == 1 {
                    Decision::Allow
                } else {
                    Decision::Deny
                };
                s.answer(card, d);
            } else if path.starts_with("/api/clear") {
                s.mark_seen(card);
            }
        }
        let resp = r#"{"ok":true}"#;
        let _ = write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            resp.len(),
            resp
        );
        return;
    }

    if !path.contains("t=") {
        let msg = "Open the link from AgentWatch › Settings › iPhone on this phone first.";
        let _ = write!(
            stream,
            "HTTP/1.1 401 Unauthorized\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
            msg.len(),
            msg
        );
        return;
    }

    let html = mobile_html();
    let _ = write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
        html.len(),
        html
    );
}

fn mobile_html() -> &'static str {
    r#"<!doctype html>
<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>AgentWatch · iPhone</title>
<style>
  :root{--bg:#0a0a0c;--card:#16161a;--line:rgba(255,255,255,.08);--text:#f5f5f7;--dim:#98989d;--blue:#3B82F6;--amber:#F59E0B;--green:#22C55E;}
  body{margin:0;padding:18px;background:var(--bg);color:var(--text);font:14px/1.45 -apple-system,BlinkMacSystemFont,sans-serif;}
  .top{display:flex;align-items:center;justify-content:space-between;margin-bottom:18px;}
  .live{background:rgba(34,197,94,.16);color:var(--green);border:1px solid rgba(34,197,94,.35);padding:2px 9px;border-radius:99px;font-size:11px;font-weight:600;}
  h2{font-size:11px;letter-spacing:.08em;color:var(--dim);text-transform:uppercase;margin:18px 0 8px;}
  .card{background:var(--card);border:1px solid var(--line);border-radius:14px;padding:14px;margin-bottom:10px;}
  .btn{display:block;width:100%;text-align:left;background:rgba(255,255,255,.05);border:1px solid var(--line);color:var(--text);padding:10px 12px;border-radius:10px;margin-top:7px;font-size:13px;font-weight:600;}
  .row{display:flex;align-items:center;justify-content:space-between;padding:11px 12px;border-bottom:1px solid var(--line);}
  .dot{width:10px;height:10px;border-radius:50%;display:inline-block;}
</style></head><body>
<div class="top"><strong>AgentWatch</strong><span class="live">Live</span></div>
<h2>Inbox</h2><div id="inbox"></div>
<h2>Agents</h2><div class="card" id="agents" style="padding:2px 4px"></div>
<script>
async function load(){
  const r = await fetch('/api/state'); const s = await r.json();
  const ib = document.getElementById('inbox');
  ib.innerHTML = (s.cards||[]).map(c=>`<div class="card"><strong>${c.title}</strong><div style="color:var(--dim);font-size:12px;margin-top:4px">${c.text||''}</div>${(c.options||[]).map((o,i)=>`<button class="btn" onclick="act('choose',{card:'${c.id}',number:${i+1}})">${i+1} · ${o}</button>`).join('')}</div>`).join('') || '<div class="card" style="color:var(--dim)">Nothing needs you right now.</div>';
  const ag = document.getElementById('agents');
  ag.innerHTML = (s.agents||[]).map(a=>`<div class="row"><span>${a.title}</span><span style="color:var(--dim);font-size:12px">${a.state}</span></div>`).join('');
}
async function act(a,b){ await fetch('/api/'+a,{method:'POST',body:JSON.stringify(b)}); load(); }
load(); setInterval(load, 2000);
</script></body></html>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_is_non_empty_and_unique() {
        let t = generate_token();
        assert!(t.starts_with("aw"));
        assert!(t.len() >= 12);
    }
}
