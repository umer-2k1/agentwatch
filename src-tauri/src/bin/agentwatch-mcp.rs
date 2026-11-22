//! `agentwatch-mcp` — stdio MCP server shim that forwards JSON-RPC 2.0 lines to `mcp.sock`.
//!
//! Blueprint §4.9, §19.2: when AgentWatch is not running, returns a clear JSON-RPC error
//! `"AgentWatch isn't running. Open it from /Applications."` so the calling agent never hangs.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

fn main() {
    let sock = socket_path();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines().flatten() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match UnixStream::connect(&sock) {
            Ok(mut stream) => {
                let mut msg = trimmed.to_string();
                msg.push('\n');
                if stream.write_all(msg.as_bytes()).is_ok() {
                    let mut reply = String::new();
                    if BufReader::new(&stream).read_line(&mut reply).unwrap_or(0) > 0 {
                        let _ = stdout.write_all(reply.as_bytes());
                        let _ = stdout.flush();
                    }
                }
            }
            Err(_) => {
                let err = r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"AgentWatch isn't running. Open it from /Applications."}}"#;
                let _ = writeln!(stdout, "{err}");
                let _ = stdout.flush();
            }
        }
    }
}

fn socket_path() -> PathBuf {
    if let Ok(p) = std::env::var("AGENTWATCH_MCP_SOCKET") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("Library/Application Support/AgentWatch/mcp.sock")
}
