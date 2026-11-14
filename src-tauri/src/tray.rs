//! The menu bar presence.
//!
//! This is what AgentDock got right and we got wrong: the app must live in the menu bar,
//! so closing the window does not close the product. The tray shows a dot per agent and
//! lists every session, with the counts always visible in the menu itself.

use crate::model::{Session, Status};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Emitter, Manager};

/// A 34×22 menu-bar icon: three status dots, amber · blue · green.
///
/// Drawn at 2× and downsampled by macOS, so the edges stay clean at 16 pt. Colours are the
/// same tokens the board uses; a monochrome template would lose the whole point.
/// The menu-bar mark: a PNG of the app icon's 2×2 status motif at 44 px (22 pt @2×).
///
/// Shipped as a real PNG file rather than a hand-built RGBA buffer: the buffer was created
/// successfully but never rendered by macOS, and a decoded PNG is the reliable path.
fn tray_image() -> tauri::image::Image<'static> {
    tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))
        .expect("bundled tray.png must decode")
}

/// Menu-bar title. Never empty — an empty title plus an icon that macOS declines to draw
/// leaves an invisible status item, which is exactly the bug we hit.
fn title_for(sessions: &[Session]) -> String {
    let (need, _, total) = counts(sessions);
    if total == 0 {
        return "·".into();
    }
    if need > 0 {
        return need.to_string();
    }
    "·".into()
}

fn counts(sessions: &[Session]) -> (usize, usize, usize) {
    (
        sessions.iter().filter(|s| s.status == Status::NeedsYou).count(),
        sessions.iter().filter(|s| s.status == Status::Working).count(),
        sessions.len(),
    )
}

/// Build the menu from the current snapshot.
fn menu(app: &AppHandle, sessions: &[Session]) -> tauri::Result<Menu<tauri::Wry>> {
    let (need, working, total) = counts(sessions);

    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> = Vec::new();

    items.push(Box::new(
        MenuItem::with_id(
            app,
            "summary",
            if total == 0 {
                "No agents running".into()
            } else {
                format!("{} needs you · {} working · {} total", need, working, total)
            },
            false,
            None::<&str>,
        )?,
    ));
    items.push(Box::new(PredefinedMenuItem::separator(app)?));

    // One row per session, most urgent first. The dot is drawn in the label text.
    let mark = |s: &Session| match s.status {
        Status::NeedsYou => "●",
        Status::Working => "◐",
        Status::Done => "✓",
        Status::Idle => "·",
        Status::Dead => "✕",
        Status::Unknown => "?",
    };
    for s in sessions.iter().take(12) {
        let label = format!(
            "{}  {} — {}",
            mark(s),
            if s.fact.repo.is_empty() { &s.fact.title } else { &s.fact.repo },
            s.status.as_str()
        );
        items.push(Box::new(
            MenuItem::with_id(app, format!("agent:{}", s.fact.id), label, true, None::<&str>)?,
        ));
    }
    if sessions.len() > 12 {
        items.push(Box::new(MenuItem::with_id(
            app,
            "more",
            format!("… {} more (open AgentWatch)", sessions.len() - 12),
            false,
            None::<&str>,
        )?));
    }

    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    items.push(Box::new(MenuItem::with_id(
        app, "open", "Open AgentWatch", true, None::<&str>,
    )?));
    items.push(Box::new(MenuItem::with_id(
        app, "quit", "Quit AgentWatch", true, None::<&str>,
    )?));

    let refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
        items.iter().map(|b| b.as_ref()).collect();
    Menu::with_items(app, &refs)
}

pub struct TrayState {
    pub tray: TrayIcon<tauri::Wry>,
    /// Signature of the menu currently installed.
    ///
    /// Rebuilding the menu while it is open **closes it instantly** — that was the
    /// "tray opens and disappears" bug. We only rebuild when the content really changed.
    pub signature: std::sync::Mutex<String>,
}

/// Create the tray. Call once in `setup`.
pub fn build(app: &AppHandle, sessions: &[Session]) -> tauri::Result<()> {
    let m = menu(app, sessions)?;

    let tray = TrayIconBuilder::with_id("main")
        .title(title_for(sessions))
        .menu(&m)
        .show_menu_on_left_click(true)
        .tooltip("AgentWatch")
        .on_tray_icon_event(|_tray, _event| {
            // Left click opens the menu on macOS by default.
        })
        .build(app)?;

    // Builder-supplied icons never rendered on this macOS build; setting it after
    // construction does.
    if let Err(e) = tray.set_icon(Some(tray_image())) {
        eprintln!("AgentWatch: set_icon failed: {e}");
    }

    app.manage(TrayState {
        tray,
        signature: std::sync::Mutex::new(signature(sessions)),
    });
    Ok(())
}

/// A cheap fingerprint of everything the menu displays.
fn signature(sessions: &[Session]) -> String {
    let (n, w, t) = counts(sessions);
    let mut s = format!("{n}|{w}|{t}");
    for x in sessions.iter().take(12) {
        s.push('|');
        s.push_str(&x.fact.id);
        s.push(':');
        s.push_str(x.status.as_str());
        s.push(':');
        s.push_str(&x.fact.repo);
    }
    s
}

/// Refresh the menu — but only when something visibly changed.
pub fn update(app: &AppHandle, sessions: &[Session]) {
    let Some(state) = app.try_state::<TrayState>() else { return };
    let next = signature(sessions);
    let Ok(mut current) = state.signature.lock() else { return };
    if *current == next {
        return;
    }
    if let Ok(m) = menu(app, sessions) {
        let _ = state.tray.set_menu(Some(m));
        let _ = state.tray.set_title(Some(title_for(sessions)));
        *current = next;
    }
}

/// Menu click behaviour. Wire once; fires for both tray and window menus.
pub fn on_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id().as_ref();
    match id {
        "open" => show_main(app),
        "quit" => app.exit(0),
        "summary" | "more" => {}
        other => {
            if let Some(agent_id) = other.strip_prefix("agent:") {
                show_main(app);
                let _ = app.emit("focus-agent", agent_id);
            }
        }
    }
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}
