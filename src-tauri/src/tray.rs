//! The menu bar presence.
//!
//! This is what AgentDock got right and we got wrong: the app must live in the menu bar,
//! so closing the window does not close the product. The tray shows a dot per agent and
//! lists every session, with the counts always visible in the menu itself.

use crate::model::{Session, Status};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Emitter, Manager};

/// A 22×24 menu-bar icon: three dots, amber · blue · green, on a transparent plate.
///
/// Generated rather than shipped as a file, so the colours stay tied to the status model.
fn tray_image() -> tauri::image::Image<'static> {
    let (w, h) = (22usize, 24usize);
    let mut rgba = vec![0u8; w * h * 4];

    // dot radius 4, centred vertically at 12
    let dots: [(usize, [u8; 3]); 3] = [
        (5, [0xff, 0x9f, 0x0a]), // needs you
        (11, [0x0a, 0x84, 0xff]), // working
        (17, [0x30, 0xd1, 0x58]), // done
    ];
    for (cx, color) in dots {
        for y in 0..h {
            for x in 0..w {
                let dx = (x as i32 - cx as i32).abs();
                let dy = (y as i32 - 12).abs();
                if dx <= 3 && dy <= 3 {
                    // anti-aliased edge
                    let d = (dx.max(dy)) as f32;
                    let a = (1.0 - (d - 2.5).clamp(0.0, 1.0)) * 255.0;
                    let i = (y * w + x) * 4;
                    rgba[i] = color[0];
                    rgba[i + 1] = color[1];
                    rgba[i + 2] = color[2];
                    rgba[i + 3] = a as u8;
                }
            }
        }
    }
    tauri::image::Image::new_owned(rgba, w as u32, h as u32)
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
}

/// Create the tray. Call once in `setup`.
pub fn build(app: &AppHandle, sessions: &[Session]) -> tauri::Result<()> {
    let m = menu(app, sessions)?;

    let tray = TrayIconBuilder::with_id("main")
        .icon(tray_image())
        .menu(&m)
        .show_menu_on_left_click(true)
        .tooltip("AgentWatch")
        .on_tray_icon_event(|_tray, _event| {
            // Left click opens the menu on macOS by default.
        })
        .build(app)?;

    app.manage(TrayState { tray });
    Ok(())
}

/// Refresh the menu with a new snapshot.
pub fn update(app: &AppHandle, sessions: &[Session]) {
    let Some(state) = app.try_state::<TrayState>() else { return };
    if let Ok(m) = menu(app, sessions) {
        let _ = state.tray.set_menu(Some(m));
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
