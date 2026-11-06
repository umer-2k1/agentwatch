//! Agent providers.
//!
//! One file per agent (D-003). A provider turns that agent's on-disk format into
//! `SessionFact`s. It must not know about status (that is the resolver), about Tauri, or
//! about any other provider.
//!
//! Why one file each: the competitor keeps every vendor's parser in one 5,000-line store,
//! which is why each new agent is a release and untested parsers ship. A trait boundary
//! gives us a seam to test each format against a recorded fixture.

pub mod claude;

use crate::model::SessionFact;

/// How much we trust an adapter (D-003).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verification {
    /// Tested against a live session on a real install.
    Verified,
    /// Written from documentation. May be wrong. The UI says so.
    /// Constructed as soon as the second adapter lands (M1.0).
    #[allow(dead_code)]
    Unverified,
}

impl Verification {
    pub fn as_str(&self) -> &'static str {
        match self {
            Verification::Verified => "verified",
            Verification::Unverified => "unverified",
        }
    }
}

/// A source of sessions on disk.
///
/// `poll` is called on a timer. Implementations keep their own per-file read state, so a
/// poll costs one `read` of the bytes appended since the last call, not a full re-parse
/// (SPEC-002).
pub trait Provider: Send {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn verification(&self) -> Verification;

    /// Scan for sessions and return the current facts for every one found.
    fn poll(&mut self, now_ms: i64) -> Vec<SessionFact>;
}
