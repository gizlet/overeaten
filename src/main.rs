//! Overeaten · 吃吃吃: a reverse cooking game for Gizlet OS. You pay at the
//! door, then eat your way through the restaurant's task before closing
//! without eating yourself into hospital.
//!
//! Environment:
//! - `GIZLET_SAVE_DIR`: where progress is kept (`./save` otherwise).
//! - `GIZLET_LANG`: `en` for English; Simplified Chinese otherwise.
//! - `GIZLET_SWAP_AB=1`: swap the A and B buttons.
//! - `OVEREATEN_WINDOW=WxH`: the desktop window size (default 1280x960).
//! - `OVEREATEN_AUTOPILOT=1`: the visitor eats by itself.
//! - `OVEREATEN_CAPTURE=FILE.png` with `OVEREATEN_SCENE` (`title`, `street`,
//!   `dining`, `danger`, `result`, `album`, `icon`) and `OVEREATEN_AT` (seconds into
//!   the meal): render one frame to a PNG and exit.

mod app;
mod art;
mod assets;
#[cfg(all(target_os = "linux", feature = "gizlet-os"))]
mod evdev;
mod fx;
mod game;
mod gfx;
mod input;
mod rules;
mod save;
mod screens;
mod text;

use app::{App, Screen};
use raylib_sys::{self as ffi, ConfigFlags as F};
use std::ffi::CString;

/// Without a window system: raylib's DRM platform on the whole panel.
const KMS: bool = cfg!(feature = "gizlet-os");

fn env_size(name: &str) -> Option<(i32, i32)> {
    let v = std::env::var(name).ok()?;
    let (w, h) = v.split_once('x')?;
    Some((w.trim().parse().ok()?, h.trim().parse().ok()?))
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--version") {
        let version = option_env!("GIZLET_APP_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
        println!("overeaten {version}");
        return;
    }
    let capture = std::env::var("OVEREATEN_CAPTURE").ok();
    let (w, h) = if KMS { (0, 0) } else { env_size("OVEREATEN_WINDOW").unwrap_or((1280, 960)) };
    let mut flags = F::FLAG_VSYNC_HINT as u32;
    if !KMS {
        flags |= F::FLAG_MSAA_4X_HINT as u32 | F::FLAG_WINDOW_RESIZABLE as u32 | F::FLAG_WINDOW_HIGHDPI as u32;
    }
    if capture.is_some() {
        flags |= F::FLAG_WINDOW_HIDDEN as u32;
    }
    unsafe {
        ffi::SetTraceLogLevel(if KMS { ffi::TraceLogLevel::LOG_INFO } else { ffi::TraceLogLevel::LOG_ERROR } as i32);
        ffi::SetConfigFlags(flags);
        ffi::InitWindow(w, h, c"Overeaten · 吃吃吃".as_ptr());
        if !ffi::IsWindowReady() {
            eprintln!("overeaten: the display could not be opened");
            std::process::exit(1);
        }
        ffi::SetExitKey(0);
        ffi::SetTargetFPS(60);
        if !KMS {
            ffi::SetWindowMinSize(320, 240);
        }
    }
    // Sound is a bonus: a device without audio still plays.
    let audio = capture.is_none()
        && std::env::var("OVEREATEN_MUTE").map_or(true, |v| v != "1")
        && unsafe {
            ffi::InitAudioDevice();
            ffi::IsAudioDeviceReady()
        };
    if let Err(e) = assets::load(audio) {
        eprintln!("overeaten: {e}");
        unsafe { ffi::CloseWindow() };
        std::process::exit(1);
    }

    let mut controls = input::Controls::new();
    let mut app = App::new(save::Save::load(), &controls);
    app.autopilot = std::env::var("OVEREATEN_AUTOPILOT").is_ok_and(|v| v == "1");
    let mut frames_left = capture.as_ref().map(|_| setup_capture(&mut app, &controls));

    let start = unsafe { ffi::GetTime() };
    while !unsafe { ffi::WindowShouldClose() } {
        let dt = if capture.is_some() { 1.0 / 60.0 } else { unsafe { ffi::GetFrameTime() }.min(0.1) };
        let view = gfx::begin_frame();
        let now = unsafe { ffi::GetTime() } - start;
        controls.poll(dt, now);
        // Leave the meal running on its own while the window is in the background.
        let focused = KMS || capture.is_some() || unsafe { ffi::IsWindowFocused() };
        if !focused && app.screen == Screen::Dining && app.paused.is_none() {
            app.paused = Some(0);
        }
        if capture.is_none() || frames_left.is_some_and(|n| n > 1) {
            app.update(dt, &controls);
        }
        let shake = app.fx.offset();
        unsafe {
            ffi::BeginDrawing();
            ffi::ClearBackground(gfx::rgb(0x14141C));
            ffi::BeginMode2D(gfx::camera(view, shake));
        }
        screens::draw(&app);
        unsafe { ffi::EndMode2D() };
        gfx::letterbox(view, gfx::rgb(0x14141C));
        if let (Some(path), Some(n)) = (&capture, &mut frames_left) {
            *n -= 1;
            if *n == 0 {
                unsafe {
                    ffi::rlDrawRenderBatchActive();
                    let image = ffi::LoadImageFromScreen();
                    if let Ok(c) = CString::new(path.as_str()) {
                        ffi::ExportImage(image, c.as_ptr());
                    }
                    ffi::UnloadImage(image);
                    ffi::EndDrawing();
                }
                break;
            }
        }
        unsafe { ffi::EndDrawing() };
    }
    unsafe { ffi::CloseWindow() };
}

/// Puts the app in the scene a capture asks for. Returns the frames to draw first.
fn setup_capture(app: &mut App, c: &input::Controls) -> u32 {
    let scene = std::env::var("OVEREATEN_SCENE").unwrap_or_else(|_| "dining".into());
    let at: f32 = std::env::var("OVEREATEN_AT").ok().and_then(|v| v.parse().ok()).unwrap_or(20.0);
    app.fx.time = 0.0;
    match scene.as_str() {
        "title" => app.go(Screen::Title),
        "street" => app.go(Screen::Street),
        "album" => app.go(Screen::Album),
        "icon" => app.icon = true,
        "dining" | "danger" | "result" => {
            app.start_meal();
            app.fast_forward(if scene == "result" { f32::MAX } else { at }, c);
            if scene == "danger" {
                if let Some(r) = &mut app.round {
                    r.fullness = rules::RULES.red_line + 9.0;
                    r.danger = Some(2.2);
                    app.shown_fullness = r.fullness;
                }
            }
            if scene == "result" {
                // Let the meal's outro play and the bill turn the cards over.
                for _ in 0..(60 * 8) {
                    app.update(1.0 / 60.0, c);
                }
                return 3;
            }
        }
        other => eprintln!("overeaten: unknown scene {other}"),
    }
    // A few frames for fades and text atlases to settle.
    30
}
