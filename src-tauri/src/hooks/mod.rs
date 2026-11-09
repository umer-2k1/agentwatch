//! Claude Code hook integration.
//!
//! A transcript can never tell us "it is blocked on a permission prompt" — that lives only
//! in the terminal UI. Hooks are the only way to learn it (SPEC-001).

pub mod event;
pub mod server;
pub mod settings;
