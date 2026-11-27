//! Overeaten · 自助大胃王: a 3D all-you-can-eat game for Gizlet OS. You pay
//! at the door, then eat your way through the room's task before closing
//! without eating yourself into hospital.
//!
//! Environment:
//! - `GIZLET_SAVE_DIR`: where progress is kept (`./save` otherwise).
//! - `GIZLET_LANG`: `en` for English; Simplified Chinese otherwise.
//! - `GIZLET_SWAP_AB=1`: swap the A and B buttons.
//! - `OVEREATEN_WINDOW=WxH`: the desktop window size (the largest 4:3 that fits by default).
//! - `OVEREATEN_AUTOPILOT=1`: the visitor eats by itself.
//! - `OVEREATEN_UNLOCK=1`: every room and hero open.
//! - `OVEREATEN_STATS=1`: start with the stats panel shown (Y toggles it).
//! - `OVEREATEN_CAPTURE=FILE.png` with `OVEREATEN_SCENE` (`title`, `heroes`,
//!   `levels`, `dining`, `danger`, `result`, `album`, `card`, `icon`),
//!   `OVEREATEN_LEVEL` (1–8), `OVEREATEN_HERO` (1–4) and `OVEREATEN_AT`
//!   (seconds into the meal): render one frame to a PNG and exit.

mod app;
mod assets;
mod cards;
#[cfg(all(target_os = "linux", feature = "gizlet-os"))]
mod evdev;
mod fx;
mod game;
mod gfx;
mod hero;
mod input;
mod level;
mod m3;
mod mesh;
mod render;
mod rules;
mod save;
mod scene;
mod screens;
mod stats;
mod tasks;
mod text;
mod ui;

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

fn env_num<T: std::str::FromStr>(name: &str) -> Option<T> {
    std::env::var(name).ok().and_then(|v| v.parse().ok())
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--version") {
        let version = option_env!("GIZLET_APP_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
        println!("overeaten {version}");
        return;
    }
    let capture = std::env::var("OVEREATEN_CAPTURE").ok();
    let asked = env_size("OVEREATEN_WINDOW");
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
        // Open small, then grow to fit the screen: macOS refuses a window
        // taller than its screen.
        let (w, h) = if KMS { (0, 0) } else { asked.unwrap_or((640, 480)) };
        ffi::InitWindow(w, h, c"Overeaten · 自助大胃王".as_ptr());
        if !ffi::IsWindowReady() || ffi::rlGetShaderIdDefault() == 0 {
            eprintln!("overeaten: the display could not be opened");
            std::process::exit(1);
        }
        if !KMS && asked.is_none() && capture.is_none() {
            let m = ffi::GetCurrentMonitor();
            let (mw, mh) = (ffi::GetMonitorWidth(m), ffi::GetMonitorHeight(m));
            let k = ((mw as f32 * 0.85) / 640.0).min((mh as f32 * 0.82) / 480.0).max(1.0);
            let (ww, wh) = ((640.0 * k) as i32, (480.0 * k) as i32);
            ffi::SetWindowSize(ww, wh);
            ffi::SetWindowPosition((mw - ww) / 2, ((mh - wh) / 2).max(30));
        }
        // raylib's one batch buffer is rewritten at every flush; on Panfrost
        // rewriting a buffer the frame still uses stalls the CPU. Many
        // buffers in rotation keep each flush on a fresh one (gizlet/little-prince).
        let batch = Box::leak(Box::new(ffi::rlLoadRenderBatch(48, 1024)));
        ffi::rlSetRenderBatchActive(batch);
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
    let mut app = match App::new(save::Save::load(), &controls) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("overeaten: {e}");
            unsafe { ffi::CloseWindow() };
            std::process::exit(1);
        }
    };
    app.autopilot = std::env::var("OVEREATEN_AUTOPILOT").is_ok_and(|v| v == "1");
    let mut frames_left = capture.as_ref().map(|_| setup_capture(&mut app, &controls));

    let start = unsafe { ffi::GetTime() };
    while !unsafe { ffi::WindowShouldClose() } {
        let dt = if capture.is_some() { 1.0 / 60.0 } else { unsafe { ffi::GetFrameTime() }.min(0.1) };
        let now = unsafe { ffi::GetTime() } - start;
        controls.poll(dt, now);
        // Leave the meal waiting while the window is in the background.
        let focused = KMS || capture.is_some() || unsafe { ffi::IsWindowFocused() };
        if !focused && app.screen == Screen::Dining && app.paused.is_none() {
            app.paused = Some(0);
        }
        if capture.is_none() || frames_left.is_some_and(|n| n > 1) {
            app.update(dt, &controls);
        }
        app.prepare();
        let view = gfx::begin_frame();
        let shake = app.fx_offset();
        unsafe {
            ffi::BeginDrawing();
            ffi::ClearBackground(screens::clear_color(&app));
            ffi::BeginMode2D(gfx::camera(view, (0.0, 0.0)));
        }
        screens::backdrop(&app);
        unsafe { ffi::EndMode2D() };
        screens::draw3d(&app, view);
        unsafe { ffi::BeginMode2D(gfx::camera(view, shake)) };
        screens::draw(&app);
        unsafe { ffi::EndMode2D() };
        gfx::letterbox(view, gfx::rgb(0x14141C));
        app.gl.end_frame();
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
    let at: f32 = env_num("OVEREATEN_AT").unwrap_or(20.0);
    if let Some(l) = env_num::<usize>("OVEREATEN_LEVEL") {
        app.level_sel = l.clamp(1, rules::LEVELS.len()) - 1;
        app.level_pos = app.level_sel as f32;
    }
    if let Some(h) = env_num::<usize>("OVEREATEN_HERO") {
        app.hero_sel = h.clamp(1, rules::HEROES.len()) - 1;
        app.hero_pos = app.hero_sel as f32;
    }
    app.fx.time = 0.0;
    match scene.as_str() {
        "title" => {
            app.go(Screen::Title);
            for _ in 0..(60.0 * at.min(8.0)) as usize {
                app.update(1.0 / 60.0, c);
            }
        }
        "heroes" => app.go(Screen::Heroes),
        "levels" => app.go(Screen::Levels),
        "album" | "card" => {
            app.go(Screen::Album);
            if let Some(b) = env_num::<usize>("OVEREATEN_BOOK") {
                app.album_book = b.min(rules::BOOKS.len() - 1);
            }
            if let Some(s) = env_num::<usize>("OVEREATEN_CARD") {
                app.album_sel = s;
            }
            if scene == "card" {
                app.album_open = Some(0.0);
            }
        }
        "icon" => app.icon = true,
        "dining" | "danger" | "result" => {
            app.go(Screen::Levels);
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
                for _ in 0..(60 * 9) {
                    app.update(1.0 / 60.0, c);
                }
                return 3;
            }
        }
        other => eprintln!("overeaten: unknown scene {other}"),
    }
    // A few frames for fades and text atlases to settle.
    40
}
