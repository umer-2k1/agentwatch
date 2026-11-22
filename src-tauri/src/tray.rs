//! The menu-bar presence — Blueprint §4.3 (Figure 38).
//!
//! Renders the native NSMenu with quick navigation to every surface (`Ask`, `Inbox`,
//! `Activity`, `Sessions Grid`, `Settings`, `Welcome Tour`, `Diagnostics`), live agent rows,
//! and the needs-you count badge in the menu bar.

use crate::model::{Session, Status};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Emitter, Manager};

fn tray_image() -> tauri::image::Image<'static> {
    tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))
        .expect("bundled tray.png must decode")
}

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

fn menu(app: &AppHandle, sessions: &[Session]) -> tauri::Result<Menu<tauri::Wry>> {
    let (need, working, total) = counts(sessions);
    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> = Vec::new();

    items.push(Box::new(MenuItem::with_id(
        app,
        "nav:ask",
        "Ask an Agent…  ⌥⌥",
        true,
        None::<&str>,
    )?));
    items.push(Box::new(MenuItem::with_id(
        app,
        "nav:inbox",
        if need > 0 {
            format!("Open Inbox ({need})  ⌃⌥I")
        } else {
            "Open Inbox  ⌃⌥I".into()
        },
        true,
        None::<&str>,
    )?));
    items.push(Box::new(MenuItem::with_id(
        app,
        "nav:activity",
        "Activity…",
        true,
        None::<&str>,
    )?));
    items.push(Box::new(MenuItem::with_id(
        app,
        "nav:sessions",
        "Sessions Grid (Mission Control)…",
        true,
        None::<&str>,
    )?));
    items.push(Box::new(PredefinedMenuItem::separator(app)?));

    items.push(Box::new(MenuItem::with_id(
        app,
        "summary",
        if total == 0 {
            "No agents running".into()
        } else {
            format!("{need} needs you · {working} working · {total} active")
        },
        false,
        None::<&str>,
    )?));

    let mark = |s: &Session| match s.status {
        Status::NeedsYou => "🟠",
        Status::Working => "🔵",
        Status::Done => "🟢",
        Status::Idle => "⚪",
        Status::Dead => "🔴",
        Status::Unknown => "◌",
    };

    for s in sessions.iter().take(10) {
        let label = format!(
            "{}  {} — {} ({})",
            mark(s),
            if s.fact.title.is_empty() {
                &s.fact.repo
            } else {
                &s.fact.title
            },
            s.status.as_str(),
            s.fact.provider
        );
        items.push(Box::new(MenuItem::with_id(
            app,
            format!("agent:{}", s.fact.id),
            label,
            true,
            None::<&str>,
        )?));
    }

    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    items.push(Box::new(MenuItem::with_id(
        app,
        "nav:settings",
        "Settings…",
        true,
        Some("CmdOrCtrl+,"),
    )?));
    items.push(Box::new(MenuItem::with_id(
        app,
        "refresh",
        "Refresh",
        true,
        Some("CmdOrCtrl+R"),
    )?));
    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    items.push(Box::new(MenuItem::with_id(
        app,
        "nav:tour",
        "Welcome Tour…",
        true,
        None::<&str>,
    )?));
    items.push(Box::new(MenuItem::with_id(
        app,
        "nav:diagnostics",
        "Diagnostics & MCP…",
        true,
        None::<&str>,
    )?));
    items.push(Box::new(MenuItem::with_id(
        app,
        "nav:roadmap",
        "Request a Feature / Roadmap…",
        true,
        None::<&str>,
    )?));
    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    items.push(Box::new(MenuItem::with_id(
        app,
        "version",
        "AgentWatch 0.2.0 · Local & Private",
        false,
        None::<&str>,
    )?));
    items.push(Box::new(MenuItem::with_id(
        app,
        "quit",
        "Quit AgentWatch",
        true,
        Some("CmdOrCtrl+Q"),
    )?));

    let refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
        items.iter().map(|b| b.as_ref()).collect();
    Menu::with_items(app, &refs)
}

pub struct TrayState {
    pub tray: TrayIcon<tauri::Wry>,
    pub signature: std::sync::Mutex<String>,
}

pub fn build(app: &AppHandle, sessions: &[Session]) -> tauri::Result<()> {
    let m = menu(app, sessions)?;
    let tray = TrayIconBuilder::with_id("main")
        .menu(&m)
        .show_menu_on_left_click(true)
        .tooltip("AgentWatch")
        .title(title_for(sessions))
        .build(app)?;

    let _ = tray.set_icon(Some(tray_image()));

    app.manage(TrayState {
        tray,
        signature: std::sync::Mutex::new(signature(sessions)),
    });
    Ok(())
}

fn signature(sessions: &[Session]) -> String {
    let (n, w, t) = counts(sessions);
    let mut s = format!("{n}|{w}|{t}");
    for x in sessions.iter().take(10) {
        s.push('|');
        s.push_str(&x.fact.id);
        s.push(':');
        s.push_str(x.status.as_str());
    }
    s
}

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

pub fn on_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id().as_ref();
    match id {
        "quit" => app.exit(0),
        "refresh" => {
            show_main(app);
            let _ = app.emit("navigate", "now");
        }
        "summary" | "version" => {}
        other => {
            if let Some(view) = other.strip_prefix("nav:") {
                show_main(app);
                let _ = app.emit("navigate", view);
            } else if let Some(agent_id) = other.strip_prefix("agent:") {
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
