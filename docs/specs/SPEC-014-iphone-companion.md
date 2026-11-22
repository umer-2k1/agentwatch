# SPEC-014 — iPhone LAN Companion Server & Mobile Web UI

Status: **specified** (Blueprint §4.10, §10.F11, §19.3, Figures 19, 21, 22)

## 1. Local HTTP Server (`:47811`)

When enabled in `Settings › iPhone`:
- Binds `0.0.0.0:47811` (or next available port, reported in Settings).
- Generates a random 24-byte URL-safe token stored in `config.json`.
- `New Link` rotates the token immediately, invalidating previous links.
- Requests without a valid `?t=<token>` or `Authorization: Bearer <token>` receive `401`:
  `"Open the link from AgentWatch › Settings › iPhone on this phone first."`

## 2. REST Contract (§19.3)

- `GET /?t=<token>` → serves the mobile dark-glass responsive web application (Inbox + Agents,
  live 500 ms timer, 2 s poll, 2 s undo banner).
- `GET /api/state?t=<token>` → returns `{ agents, cards, history, unseen }` matching §19.3.
- `POST /api/{undo|seen|choose|step|reply|clear}?t=<token>` with `{ card, number?, text? }` →
  executes the action on `SharedStore` with the same 2-second undo semantics as the desktop Inbox.
