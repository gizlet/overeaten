//! Drawing each screen: the 3D layer in the canvas's viewport, then the 2D
//! interface on the 640×480 canvas.

use crate::app::{App, Screen, Summary};
use crate::cards;
use crate::game::{Outcome, Round};
use crate::gfx::{self, ease_back, ease_out, fade, mix, rgb, rrect, Align, View};
use crate::m3::{v3, M4};
use crate::mesh::Builder;
use crate::render::{begin3d, end3d, Camera, Lighting, Look};
use crate::rules::*;
use crate::tasks::{Goal, State};
use crate::text as tx;
use crate::ui::{self, pal::*};
use raylib_sys::{self as ffi, Color, Mesh};
use std::cell::OnceCell;
use std::f32::consts::PI;

thread_local! {
    static STAGE: OnceCell<Mesh> = const { OnceCell::new() };
}

/// The framebuffer rectangle the canvas covers. `View` is in drawing units:
/// framebuffer pixels on macOS, window points elsewhere, so other high-DPI
/// windows are scaled up to the framebuffer here.
fn canvas_rect(view: View, shake: (f32, f32)) -> (i32, i32, i32, i32, i32, i32) {
    let (fw, fh, k) = unsafe {
        let (rw, rh) = (ffi::GetRenderWidth(), ffi::GetRenderHeight());
        let k = if cfg!(target_os = "macos") { 1.0 } else { rw as f32 / ffi::GetScreenWidth().max(1) as f32 };
        (rw, rh, k)
    };
    let x = ((view.ox + shake.0 * view.zoom) * k).round() as i32;
    let y = ((view.oy + shake.1 * view.zoom) * k).round() as i32;
    (x, y, (gfx::W * view.zoom * k).round() as i32, (gfx::H * view.zoom * k).round() as i32, fw, fh)
}

/// The colour behind everything this frame.
pub fn clear_color(app: &App) -> Color {
    match app.screen {
        Screen::Title | Screen::Levels | Screen::Dining | Screen::Result => app.scene.as_ref().map_or(INK, |s| s.clear),
        _ => rgb(0x14141C),
    }
}

/// The 2D backdrop drawn under the 3D layer.
pub fn backdrop(app: &App) {
    match app.screen {
        Screen::Heroes => {
            gfx::grad_v(0.0, 0.0, gfx::W, gfx::H, rgb(0x2B3A55), rgb(0x16152A));
            for k in 0..3 {
                let x = 130.0 + k as f32 * 190.0 + (app.t * 0.3 + k as f32).sin() * 20.0;
                gfx::glow(x, 120.0, 220.0, fade(rgb(0x6C8EBF), 0.18));
            }
        }
        Screen::Album => ui::dotted_backdrop(rgb(0x2F4858), rgb(0x1B2733), app.t),
        _ => {}
    }
}

/// The 3D layer.
pub fn draw3d(app: &App, view: View) {
    if app.icon {
        return;
    }
    let (x, y, w, h, fw, fh) = canvas_rect(view, app.fx_offset());
    match app.screen {
        Screen::Title | Screen::Levels | Screen::Dining | Screen::Result => {
            let (Some(sc), Some(r)) = (&app.scene, &app.round) else { return };
            begin3d(&sc.cam, x, y, w, h, fh);
            sc.draw(&app.gl, r, &app.heroes[r.hero], app.chef.as_ref(), app.t);
            end3d(fw, fh);
        }
        Screen::Heroes => {
            let cam = hero_cam();
            begin3d(&cam, x, y, w, h, fh);
            let mut light = Lighting::day();
            light.sun_dir = v3(-0.3, 0.8, 0.6).norm();
            let sel = app.hero_pos;
            light.add_light(v3(hero_px(sel), 3.0, 1.0), 4.0, rgb(0xFFE7B0));
            app.gl.light(&light, cam.eye);
            let stage = STAGE.with(|s| {
                *s.get_or_init(|| {
                    let mut b = Builder::default();
                    b.cylinder(v3(0.0, -0.2, 0.0), 1.0, 0.2, 40, rgb(0x8B5A3C), rgb(0xE2B98E));
                    b.upload(false)
                })
            });
            let a = crate::assets::get();
            for i in 0..HEROES.len() {
                let px = hero_px(i as f32);
                let chosen = i == app.hero_sel;
                let lift = if chosen { 0.08 } else { 0.0 };
                let disc = M4::scale(0.5, 1.0, 0.5).then(&M4::translate(v3(px, lift, 0.0)));
                let wood = Some(a.texture(a.tex("counter")));
                app.gl.draw(stage, wood, &disc, &Look { emissive: if chosen { 0.25 } else { 0.0 }, ..Look::default() });
                let open = app.save.hero_unlocked(i);
                let yaw = if chosen { (app.t * 0.6).sin() * 0.3 } else { (px * -0.25) + (app.t * 0.4 + i as f32).sin() * 0.1 };
                let m = app.heroes[i].place(v3(px, lift, 0.0), yaw, HEROES[i].height);
                let look = if open {
                    Look { rim: if chosen { 1.0 } else { 0.4 }, ..Look::default() }
                } else {
                    Look { tint: (rgb(0x0E0D18), 0.9), rim: 0.8, ..Look::default() }
                };
                app.heroes[i].draw(&app.gl, &m, &look, if chosen { Some(2.0) } else { None });
            }
            end3d(fw, fh);
        }
        Screen::Album => {}
    }
}

pub fn draw(app: &App) {
    if app.icon {
        icon(app);
        return;
    }
    match app.screen {
        Screen::Title => title(app),
        Screen::Heroes => heroes(app),
        Screen::Levels => levels(app),
        Screen::Dining => dining(app),
        Screen::Result => result(app),
        Screen::Album => album(app),
    }
    app.fx.draw_banner();
    if app.stats_on {
        stats(app);
    }
    if app.fade > 0.0 {
        gfx::rect(0.0, 0.0, gfx::W, gfx::H, fade(INK, app.fade * app.fade));
    }
}

impl App {
    pub fn fx_offset(&self) -> (f32, f32) {
        self.fx.peek_offset()
    }
}

// ---- Title ---------------------------------------------------------------

fn logo(app: &App, cx: f32, y: f32, size: f32) {
    let t = app.t;
    let name = app.tr(tx::TITLE);
    let chars: Vec<char> = name.chars().collect();
    let cjk = app.lang == Lang::Zh;
    if cjk {
        let step = size * 1.02;
        let x0 = cx - step * (chars.len() as f32 - 1.0) / 2.0;
        for (i, ch) in chars.iter().enumerate() {
            let k = ((t * 3.2 - i as f32 * 0.5).sin()).max(0.0);
            let x = x0 + i as f32 * step;
            let yy = y - k * 10.0;
            let s = ch.to_string();
            let big = i >= 2;
            let sz = if big { size } else { size * 0.78 };
            let dy = if big { 0.0 } else { size * 0.2 };
            gfx::text(&s, x + 4.0, yy + dy + 6.0, sz, fade(rgb(0x7A2416), 0.6), Align::Center);
            gfx::text_outlined(&s, x, yy + dy, sz, if big { CREAM } else { MUSTARD }, INK, 4.0, Align::Center);
        }
    } else {
        gfx::text(name, cx + 4.0, y + 6.0, size * 0.9, fade(rgb(0x7A2416), 0.6), Align::Center);
        gfx::text_outlined(name, cx, y - ((t * 3.0).sin().max(0.0)) * 6.0, size * 0.9, CREAM, INK, 4.0, Align::Center);
    }
}

fn title(app: &App) {
    gfx::grad_v(0.0, 0.0, gfx::W, 220.0, fade(INK, 0.75), fade(INK, 0.0));
    logo(app, 320.0, 30.0, 78.0);
    let sub = if app.lang == Lang::Zh { "OVEREATEN" } else { "自助大胃王" };
    gfx::text_outlined(sub, 320.0, 128.0, 24.0, MUSTARD, INK, 2.5, Align::Center);
    gfx::text_outlined(app.tr(tx::TAGLINE), 320.0, 160.0, 18.0, CREAM, INK, 2.0, Align::Center);
    let owned = CARDS.iter().filter(|c| app.save.has(c.id)).count();
    rrect(16.0, 16.0, 120.0, 30.0, 15.0, fade(INK, 0.7));
    gfx::text(&format!("{} {owned}/{}", app.tr(tx::COLLECTED), CARDS.len()), 76.0, 20.0, 17.0, CREAM, Align::Center);
    rrect(504.0, 16.0, 120.0, 30.0, 15.0, fade(INK, 0.7));
    ui::star(522.0, 31.0, 9.0, 1.0, GOLD, fade(CREAM, 0.3));
    gfx::text(&format!("{}", app.save.total_stars()), 572.0, 20.0, 17.0, CREAM, Align::Center);
    rrect(16.0, 408.0, 608.0, 24.0, 12.0, fade(INK, 0.55));
    gfx::text(app.tr(tx::CREDITS), 320.0, 411.0, 15.0, fade(CREAM, 0.85), Align::Center);
    let [a, _, x] = app.labels;
    ui::prompts(&[(a, app.tr(tx::START)), (x, app.tr(tx::ALBUM)), ("Y", app.tr(tx::STATS)), ("Select", app.tr(tx::LANGUAGE))], true);
}

fn icon(app: &App) {
    ui::dotted_backdrop(rgb(0xEA6A3F), rgb(0xC4402A), 0.0);
    gfx::glow(320.0, 240.0, 230.0, fade(rgb(0xFFD08A), 0.55));
    if let Some(p) = app.portraits.get(Art::Dish(GOLDEN)) {
        cards::blit(p, 120.0, 60.0, 400.0, 400.0 * cards::PH as f32 / cards::PW as f32, true, WHITE);
    }
    gfx::text_outlined("吃", 320.0, 330.0, 110.0, CREAM, INK, 6.0, Align::Center);
}

// ---- Heroes ---------------------------------------------------------------

fn heroes(app: &App) {
    gfx::text_outlined(app.tr(tx::PICK_HERO), 320.0, 18.0, 30.0, CREAM, INK, 3.0, Align::Center);
    let i = app.hero_sel;
    let h = &HEROES[i];
    let open = app.save.hero_unlocked(i);
    for k in 0..HEROES.len() {
        let x = hero_screen_x(k);
        let sel = k == i;
        let c = if sel { MUSTARD } else { fade(CREAM, 0.6) };
        if app.save.hero_unlocked(k) {
            gfx::text_outlined(HEROES[k].name.get(app.lang), x, 294.0, if sel { 20.0 } else { 16.0 }, c, INK, 2.0, Align::Center);
        } else {
            ui::lock(x, 306.0, fade(CREAM, 0.8));
        }
    }
    ui::panel(24.0, 322.0, 592.0, 112.0);
    let accent = rgb(h.accent);
    rrect(36.0, 334.0, 6.0, 88.0, 3.0, accent);
    let name = if open { h.name.get(app.lang) } else { app.tr(tx::LOCKED) };
    gfx::text(name, 52.0, 330.0, 24.0, CREAM, Align::Left);
    let desc = if open {
        h.perk.get(app.lang).to_string()
    } else {
        match h.unlock {
            Unlock::Stars(n) => format!("{}（{}/{n}）", tx::fill(app.tr(tx::NEED_STARS), n), app.save.total_stars()),
            Unlock::Hospital => app.tr(tx::NEED_HOSPITAL).to_string(),
            Unlock::Start => String::new(),
        }
    };
    gfx::text_wrapped(&desc, 52.0, 362.0, 300.0, 15.0, 19.0, fade(CREAM, 0.9), Align::Left);
    for (k, label) in tx::RATING.iter().enumerate() {
        let y = 338.0 + k as f32 * 22.0;
        gfx::text(label.get(app.lang), 372.0, y, 15.0, fade(CREAM, 0.85), Align::Left);
        for p in 0..5 {
            let on = open && (p as u8) < h.ratings[k];
            rrect(450.0 + p as f32 * 30.0, y + 3.0, 26.0, 11.0, 5.0, if on { accent } else { fade(CREAM, 0.15) });
        }
    }
    let [a, b, x] = app.labels;
    ui::prompts(&[(a, app.tr(tx::CHOOSE)), (b, app.tr(tx::BACK)), (x, app.tr(tx::ALBUM))], true);
}

const HERO_STEP: f32 = 1.2;

fn hero_cam() -> Camera {
    Camera { eye: v3(0.0, 2.2, 9.6), target: v3(0.0, 0.62, 0.0), fovy: 0.6 }
}

/// Where hero `i` stands on the stage, in metres from the middle.
fn hero_px(i: f32) -> f32 {
    (i - (HEROES.len() as f32 - 1.0) / 2.0) * HERO_STEP
}

fn hero_screen_x(i: usize) -> f32 {
    let c = hero_cam().view_proj(4.0 / 3.0).project(v3(hero_px(i as f32), 0.0, 0.0));
    (c[0] / c[3] + 1.0) * 320.0
}

// ---- Rooms ----------------------------------------------------------------

fn kind_chip(g: &Goal, x: f32, y: f32, lang: Lang) -> f32 {
    let k = g.kind();
    let label = k.name().get(lang);
    let w = gfx::measure(label, 13.0) + 12.0;
    rrect(x, y, w, 18.0, 9.0, rgb(k.color()));
    gfx::text(label, x + w / 2.0, y + 1.0, 13.0, INK, Align::Center);
    w
}

fn levels(app: &App) {
    let lang = app.lang;
    gfx::grad_h(300.0, 0.0, 340.0, gfx::H, fade(INK, 0.0), fade(INK, 0.55));
    // Restaurants along the top.
    let mut x = 16.0;
    for r in &RESTAURANTS {
        let label = if r.open { r.name.get(lang).to_string() } else { format!("{} · {}", r.name.get(lang), app.tr(tx::SOON)) };
        let w = gfx::measure(&label, 16.0) + 24.0;
        rrect(x, 14.0, w, 28.0, 14.0, if r.open { rgb(r.accent) } else { fade(INK, 0.6) });
        gfx::text(&label, x + w / 2.0, 18.0, 16.0, if r.open { CREAM } else { fade(CREAM, 0.5) }, Align::Center);
        x += w + 8.0;
    }
    // The hero eating today.
    let h = &HEROES[app.hero_sel];
    if let Some(p) = app.portraits.get(Art::Hero(app.hero_sel)) {
        rrect(16.0, 50.0, 132.0, 42.0, 21.0, fade(INK, 0.65));
        gfx::circle(37.0, 71.0, 18.0, rgb(h.accent));
        cards::blit(p, 14.0, 46.0, 48.0, 44.0, true, WHITE);
        gfx::text(h.name.get(lang), 62.0, 61.0, 16.0, CREAM, Align::Left);
    }
    // Room dots.
    for i in 0..LEVELS.len() {
        let x = 330.0 + i as f32 * 36.0;
        let sel = i == app.level_sel;
        let open = app.save.level_open(i);
        gfx::circle(x + 14.0, 30.0, if sel { 14.0 } else { 11.0 }, if sel { MUSTARD } else if open { fade(CREAM, 0.85) } else { fade(INK, 0.7) });
        if open {
            gfx::text(&format!("{}", i + 1), x + 14.0, 21.0, 16.0, INK, Align::Center);
        } else {
            ui::lock(x + 14.0, 30.0, fade(CREAM, 0.6));
        }
        let st = app.save.stars_of(i);
        for s in 0..3 {
            ui::star(x + 6.0 + s as f32 * 8.0, 50.0, 3.6, if (s as u32) < st { 1.0 } else { 0.0 }, GOLD, fade(INK, 0.6));
        }
    }
    // Tickets slide in from the side.
    for i in 0..LEVELS.len() {
        let off = i as f32 - app.level_pos;
        if off.abs() > 1.2 {
            continue;
        }
        ticket(app, i, 330.0 + off * 320.0, 64.0);
    }
    let [a, b, x] = app.labels;
    let enter = if app.save.level_open(app.level_sel) { app.tr(tx::ENTER) } else { app.tr(tx::ROOM_LOCKED) };
    ui::prompts(&[(a, enter), (b, app.tr(tx::HERO)), (x, app.tr(tx::ALBUM))], true);
}

fn ticket(app: &App, i: usize, x: f32, y: f32) {
    let lang = app.lang;
    let d = &LEVELS[i];
    let (w, h) = (296.0, 366.0);
    rrect(x + 3.0, y + 5.0, w, h, 10.0, fade(INK, 0.4));
    rrect(x, y, w, h, 10.0, PAPER);
    // Perforations and the stub.
    for k in 0..18 {
        gfx::circle(x + 10.0 + k as f32 * 16.0, y + 58.0, 2.5, fade(INK, 0.25));
    }
    gfx::circle(x, y + 58.0, 8.0, fade(INK, 0.6));
    gfx::circle(x + w, y + 58.0, 8.0, fade(INK, 0.6));
    rrect(x, y, w, 50.0, 10.0, rgb(RESTAURANTS[0].accent));
    gfx::text(&tx::fill(app.tr(tx::ROOM), i + 1), x + 14.0, y + 4.0, 14.0, fade(CREAM, 0.9), Align::Left);
    gfx::text(d.name.get(lang), x + 14.0, y + 19.0, 24.0, CREAM, Align::Left);
    gfx::text(&format!("{} {}", app.tr(tx::ADMISSION), d.admission), x + w - 14.0, y + 8.0, 15.0, CREAM, Align::Right);
    gfx::text(&format!("{} {:.0}s", app.tr(tx::CLOSING), d.duration), x + w - 14.0, y + 28.0, 15.0, fade(CREAM, 0.85), Align::Right);
    let mut yy = y + 68.0;
    yy += gfx::text_wrapped(d.gimmick.get(lang), x + 14.0, yy, w - 28.0, 15.0, 19.0, fade(INK, 0.8), Align::Left) + 6.0;
    gfx::text(app.tr(tx::MAIN_TASK), x + 14.0, yy, 14.0, TOMATO, Align::Left);
    kind_chip(&d.main, x + 16.0 + gfx::measure(app.tr(tx::MAIN_TASK), 14.0) + 6.0, yy, lang);
    yy += 22.0;
    yy += gfx::text_wrapped(&d.main.text(lang), x + 14.0, yy, w - 28.0, 18.0, 22.0, INK, Align::Left) + 8.0;
    gfx::text(&format!("{} ({})", app.tr(tx::SIDE_GOALS), tx::fill(app.tr(tx::EACH_GOAL), RULES.bonus)), x + 14.0, yy, 14.0, TOMATO, Align::Left);
    yy += 20.0;
    let goals: Vec<Goal> = if i == app.level_sel { app.ticket_goals.clone() } else { Vec::new() };
    for g in &goals {
        let cw = kind_chip(g, x + 14.0, yy, lang);
        gfx::text_wrapped(&g.text(lang), x + 20.0 + cw, yy, w - 40.0 - cw, 15.0, 18.0, INK, Align::Left);
        yy += 24.0;
    }
    // Today's menu: the six dishes, their points and how filling they are.
    let my = y + h - 122.0;
    gfx::text(app.tr(tx::MENU), x + 14.0, my - 2.0, 14.0, TOMATO, Align::Left);
    for (k, dish) in DISHES[..EVERYDAY].iter().enumerate() {
        let cx = x + 14.0 + k as f32 * 45.0;
        if let Some(p) = app.portraits.get(Art::Dish(k)) {
            cards::blit(p, cx - 2.0, my + 14.0, 44.0, 40.0, true, WHITE);
        }
        gfx::text(&format!("{}", dish.value), cx + 20.0, my + 50.0, 13.0, INK, Align::Center);
        gfx::text(&format!("{}%", dish.fullness), cx + 20.0, my + 63.0 - 2.0, 11.0, fade(INK, 0.55), Align::Center);
    }
    // Stars and the best score.
    let by = y + h - 60.0;
    rrect(x + 10.0, by, w - 20.0, 50.0, 8.0, fade(INK, 0.08));
    let st = app.save.stars_of(i);
    for s in 0..3 {
        ui::star(x + 32.0 + s as f32 * 28.0, by + 25.0, 11.0, if (s as u32) < st { 1.0 } else { 0.0 }, GOLD, fade(INK, 0.25));
    }
    let lines = tx::fill(&tx::fill(app.tr(tx::STAR_LINES), d.stars[0]), d.stars[1]);
    gfx::text(&lines, x + w - 14.0, by + 6.0, 13.0, fade(INK, 0.7), Align::Right);
    let best = app.save.best_of(i).map_or("—".to_string(), |b| b.to_string());
    gfx::text(&format!("{} {best}", app.tr(tx::BEST)), x + w - 14.0, by + 24.0, 18.0, INK, Align::Right);
    if !app.save.level_open(i) {
        rrect(x, y, w, h, 10.0, fade(INK, 0.55));
        ui::lock(x + w / 2.0, y + h / 2.0 - 10.0, CREAM);
        gfx::text(app.tr(tx::ROOM_LOCKED), x + w / 2.0, y + h / 2.0 + 16.0, 18.0, CREAM, Align::Center);
    }
}

// ---- The meal --------------------------------------------------------------

fn dining(app: &App) {
    let Some(r) = &app.round else { return };
    hud(app, r);
    if let Some(sel) = app.paused {
        pause(app, sel);
    }
}

fn hud(app: &App, r: &Round) {
    let lang = app.lang;
    let d = r.def();
    // The danger: red edges and a countdown over the visitor.
    if let Some(t) = r.danger {
        let pulse = 0.5 + 0.5 * (app.t * 2.0 * PI).sin();
        let a = 0.25 + 0.3 * pulse;
        gfx::grad_v(0.0, 0.0, gfx::W, 60.0, fade(RED, a), fade(RED, 0.0));
        gfx::grad_v(0.0, gfx::H - 60.0, gfx::W, 60.0, fade(RED, 0.0), fade(RED, a));
        gfx::grad_h(0.0, 0.0, 60.0, gfx::H, fade(RED, a), fade(RED, 0.0));
        gfx::grad_h(gfx::W - 60.0, 0.0, 60.0, gfx::H, fade(RED, 0.0), fade(RED, a));
        let left = (r.hero_def().hospital_s - t).max(0.0);
        let floor = r.layout.floor_at(r.player.pos);
        let (x, y) = app.on_canvas(r.player.pos, floor + 200.0);
        gfx::circle(x, y, 22.0, fade(INK, 0.8));
        gfx::ring(x, y, 17.0, 21.0, -90.0, -90.0 + 360.0 * left / r.hero_def().hospital_s, RED);
        gfx::text(&format!("{:.0}", left.ceil()), x, y - 13.0, 24.0, CREAM, Align::Center);
    }
    // The bubble over the plate in reach.
    if let Some(i) = r.reachable() {
        let p = &r.plates[i];
        let b = &r.layout.belts[p.belt];
        let (x, y) = app.on_canvas(b.point(p.s).0, b.top() + 45.0);
        let seen = r.visible(p);
        let dish = &DISHES[p.dish];
        let chained = r.last_dish().is_some_and(|l| l != p.dish) && r.combo_left() > 0.0;
        let next = if chained { r.combo + 1 } else { 1 };
        let label = if seen {
            format!("{}  +{}", dish.name.get(lang), r.points_of(p.dish, next))
        } else {
            app.tr(tx::UNKNOWN).to_string()
        };
        let bw = gfx::measure(&label, 16.0) + 22.0;
        let (bx, by) = ((x - bw / 2.0).clamp(8.0, 632.0 - bw), (y - 36.0).max(70.0));
        rrect(bx, by, bw, 26.0, 13.0, fade(INK, 0.88));
        gfx::tri((x - 6.0, by + 25.0), (x + 6.0, by + 25.0), (x, by + 32.0), fade(INK, 0.88));
        let col = if seen { rgb(dish.hue.plate()) } else { fade(CREAM, 0.6) };
        gfx::circle(bx + 11.0, by + 13.0, 4.0, col);
        gfx::text(&label, bx + bw / 2.0 + 4.0, by + 4.0, 16.0, CREAM, Align::Center);
    }
    // The mage's foresight.
    if r.hero_def().foresight {
        for (bi, b) in r.layout.belts.iter().enumerate() {
            let (x, y) = app.on_canvas(b.point(b.serve_s()).0, b.top() + 140.0);
            let label = format!("{}：{}", app.tr(tx::NEXT), DISHES[r.next_dish[bi]].name.get(lang));
            let w = gfx::measure(&label, 13.0) + 12.0;
            rrect(x - w / 2.0, y - 10.0, w, 18.0, 9.0, fade(rgb(0x5B3F8C), 0.85));
            gfx::text(&label, x, y - 8.0, 13.0, CREAM, Align::Center);
        }
    }

    // Top left: the room, the task and the side goals.
    let pw = 290.0;
    let ph = 76.0 + r.goals.len() as f32 * 20.0;
    ui::panel(12.0, 12.0, pw, ph);
    gfx::text(&format!("{} · {}", tx::fill(app.tr(tx::ROOM), r.level + 1), d.name.get(lang)), 24.0, 17.0, 14.0, fade(CREAM, 0.7), Align::Left);
    let main_ok = match r.main_state {
        State::Done => Some(true),
        State::Failed => Some(false),
        State::Open => None,
    };
    ui::mark(32.0, 46.0, 8.0, main_ok);
    gfx::text_wrapped(&d.main.text(lang), 46.0, 36.0, pw - 44.0, 16.0, 18.0, CREAM, Align::Left);
    let (now, target) = d.main.progress(&r.tally);
    let fill = if r.main_state == State::Done { MINT } else { MUSTARD };
    ui::bar(24.0, 62.0, pw - 80.0, 9.0, now / target.max(1e-3), fill);
    let prog = match d.main {
        Goal::Precision { .. } => format!("{:.0}%", r.fullness),
        Goal::Risk { secs, score } => format!("{:.1}/{secs:.0}s {}/{score}", r.tally.red_secs, r.tally.score),
        _ => format!("{:.0}/{:.0}", now, target),
    };
    gfx::text(&prog, pw + 4.0, 57.0, 14.0, fade(CREAM, 0.9), Align::Right);
    for (k, g) in r.goals.iter().enumerate() {
        let y = 80.0 + k as f32 * 20.0;
        let ok = match r.goal_state[k] {
            State::Done => Some(true),
            State::Failed => Some(false),
            State::Open => None,
        };
        ui::mark(30.0, y + 8.0, 6.0, ok);
        let c = if ok == Some(false) { fade(CREAM, 0.4) } else { fade(CREAM, 0.88) };
        gfx::text(&g.text(lang), 42.0, y, 14.0, c, Align::Left);
    }

    // Top right: the clock.
    let left = r.time_left();
    let late = left < 15.0;
    ui::panel(520.0, 12.0, 108.0, 52.0);
    let k = left / d.duration;
    gfx::circle(548.0, 38.0, 16.0, fade(INK, 0.6));
    gfx::ring(548.0, 38.0, 11.0, 16.0, -90.0, -90.0 + 360.0 * k, if late { RED } else { MUSTARD });
    let secs = left.ceil() as i32;
    gfx::text(&format!("{}:{:02}", secs / 60, secs % 60), 600.0, 24.0, 24.0, if late { rgb(0xFF8A80) } else { CREAM }, Align::Center);
    if let Some(g) = r.next_golden() {
        if matches!(d.main, Goal::Golden { .. }) && g - r.t < 30.0 {
            rrect(528.0, 70.0, 100.0, 20.0, 10.0, fade(rgb(0xC9971C), 0.9));
            gfx::text(&tx::fill(app.tr(tx::GOLDEN_IN), format!("{:.0}", (g - r.t).max(0.0))), 578.0, 72.0, 14.0, INK, Align::Center);
        }
    }

    // Bottom left: the score and the combo.
    ui::panel(12.0, 396.0, 200.0, 72.0);
    let bump = 1.0 + app.bump_score * 0.15;
    gfx::text(app.tr(tx::SCORE), 24.0, 402.0, 14.0, fade(CREAM, 0.7), Align::Left);
    gfx::transformed(24.0, 444.0, 0.0, bump, bump, || {
        gfx::text(&format!("{:.0}", app.shown_score), 0.0, -26.0, 34.0, MUSTARD, Align::Left);
    });
    if r.combo >= 2 && r.combo_left() > 0.0 {
        let m = r.multiplier();
        gfx::text(&format!("{} {} ×{m:.2}", app.tr(tx::COMBO), r.combo), 200.0, 406.0, 15.0, BLUSH, Align::Right);
        ui::bar(120.0, 430.0, 80.0, 7.0, r.combo_left() / RULES.combo_window, BLUSH);
    }
    gfx::text(&format!("{} {}", r.tally.plates, app.tr(tx::PLATES)), 200.0, 444.0, 14.0, fade(CREAM, 0.75), Align::Right);

    // Bottom right: the stomach.
    let (gx, gy, gw) = (380.0, 420.0, 230.0);
    ui::panel(368.0, 396.0, 260.0, 72.0);
    let scale = |p: f32| gx + gw * (p / 150.0).clamp(0.0, 1.0);
    gfx::text(app.tr(tx::FULLNESS), gx, 402.0, 14.0, fade(CREAM, 0.7), Align::Left);
    let shown = app.shown_fullness;
    gfx::text(&format!("{:.0}%", shown), gx + gw, 400.0, 18.0, if r.danger.is_some() { rgb(0xFF8A80) } else { CREAM }, Align::Right);
    rrect(gx, gy, gw, 18.0, 9.0, fade(INK, 0.7));
    rrect(scale(RULES.red_line), gy, gx + gw - scale(RULES.red_line), 18.0, 9.0, fade(RED, 0.45));
    let col = if shown > RULES.red_line {
        RED
    } else if shown > RULES.capacity {
        rgb(0xFFB45E)
    } else {
        mix(MINT, MUSTARD, shown / 100.0)
    };
    if shown > 0.5 {
        rrect(gx, gy, (scale(shown) - gx).max(18.0), 18.0, 9.0, col);
    }
    // The plate in reach, as a ghost.
    if let Some(i) = r.reachable() {
        let p = &r.plates[i];
        if r.visible(p) {
            let add = r.fill_of(p.dish);
            let over = r.fullness + add > RULES.red_line;
            let a = 0.4 + 0.2 * (app.t * 6.0).sin();
            gfx::rect(scale(shown), gy + 3.0, scale(shown + add) - scale(shown), 12.0, fade(if over { RED } else { CREAM }, a));
        }
    }
    gfx::rect(scale(100.0) - 1.0, gy - 3.0, 2.0, 24.0, CREAM);
    gfx::rect(scale(RULES.red_line) - 1.0, gy - 3.0, 2.0, 24.0, RED);
    for (p, label) in [(0.0, "0"), (100.0, "100"), (150.0, "150")] {
        gfx::text(label, scale(p), gy + 22.0, 12.0, fade(CREAM, 0.6), Align::Center);
    }
    if r.lean.abs() > 0.05 {
        let dir = if r.lean > 0.0 { "→" } else { "←" };
        gfx::text_outlined(&format!("{} {}", app.tr(tx::LEAN), dir), 320.0, 92.0, 20.0, rgb(0x9FD8FF), INK, 2.0, Align::Center);
    }
    app.fx.draw_world();
}

fn pause(app: &App, sel: usize) {
    gfx::rect(0.0, 0.0, gfx::W, gfx::H, fade(INK, 0.6));
    let items = app.pause_items();
    let h = 90.0 + items.len() as f32 * 40.0;
    ui::panel(170.0, 240.0 - h / 2.0, 300.0, h);
    gfx::text(app.tr(tx::PAUSED), 320.0, 240.0 - h / 2.0 + 14.0, 28.0, CREAM, Align::Center);
    for (i, it) in items.iter().enumerate() {
        let y = 240.0 - h / 2.0 + 60.0 + i as f32 * 40.0;
        if i == sel {
            rrect(196.0, y, 248.0, 34.0, 17.0, MUSTARD);
        }
        gfx::text(it.get(app.lang), 320.0, y + 6.0, 20.0, if i == sel { INK } else { CREAM }, Align::Center);
    }
    gfx::text(app.tr(tx::QUIT_HINT), 320.0, 240.0 + h / 2.0 + 10.0, 15.0, fade(CREAM, 0.8), Align::Center);
}

// ---- The bill ----------------------------------------------------------------

/// The size of the rack's cards and how many fit in a row, for `n` new cards.
pub fn rack_layout(n: usize) -> (f32, f32, usize) {
    if n <= 4 {
        (68.0, 96.0, 4)
    } else if n <= 10 {
        (54.0, 76.0, 5)
    } else {
        (44.0, 62.0, 6)
    }
}

/// Where card `i` of `n` new cards sits in the rack: its top left.
pub fn rack_slot(i: usize, n: usize) -> (f32, f32) {
    let (w, h, per_row) = rack_layout(n);
    let gap = 6.0;
    let in_row = n.min(per_row);
    let total = in_row as f32 * w + (in_row.max(1) - 1) as f32 * gap;
    let x0 = 478.0 - total / 2.0;
    let (row, col) = (i / per_row, i % per_row);
    (x0 + col as f32 * (w + gap), 244.0 + row as f32 * (h + 8.0))
}

fn result(app: &App) {
    let Some(s) = &app.summary else { return };
    let lang = app.lang;
    gfx::rect(0.0, 0.0, gfx::W, gfx::H, fade(INK, 0.45));
    // The receipt on the left.
    let (x, y, w, h) = (24.0, 20.0, 290.0, 410.0);
    let slide = ease_out(s.t * 2.5);
    let y = y - (1.0 - slide) * 300.0;
    rrect(x + 3.0, y + 5.0, w, h, 6.0, fade(INK, 0.4));
    rrect(x, y, w, h, 6.0, PAPER);
    for k in 0..19 {
        gfx::tri((x + k as f32 * 15.3, y + h), (x + k as f32 * 15.3 + 15.3, y + h), (x + k as f32 * 15.3 + 7.6, y + h + 7.0), PAPER);
    }
    let d = &LEVELS[s.level];
    gfx::text(app.tr(tx::BILL), x + w / 2.0, y + 10.0, 22.0, INK, Align::Center);
    gfx::text(&format!("{} · {} · {}", RESTAURANTS[0].name.get(lang), tx::fill(app.tr(tx::ROOM), s.level + 1), d.name.get(lang)), x + w / 2.0, y + 38.0, 14.0, fade(INK, 0.6), Align::Center);
    let mut yy = y + 64.0;
    let line = |yy: f32, a: &str, b: String, c: Color| {
        gfx::text(a, x + 18.0, yy, 16.0, fade(INK, 0.85), Align::Left);
        gfx::text(&b, x + w - 18.0, yy, 16.0, c, Align::Right);
    };
    line(yy, app.tr(tx::ADMISSION), format!("-{}", d.admission), fade(INK, 0.6));
    yy += 24.0;
    line(yy, app.tr(tx::FOOD), format!("{}  ({} {})", s.food, s.plates, app.tr(tx::PLATES)), INK);
    yy += 24.0;
    line(yy, app.tr(tx::COMBO_BONUS), format!("+{}", s.combo_bonus), BLUSH);
    yy += 24.0;
    line(yy, app.tr(tx::PEAK), format!("{:.0}%", s.peak), if s.peak > RULES.red_line { RED } else { fade(INK, 0.7) });
    yy += 24.0;
    gfx::text(app.tr(tx::GOALS), x + 18.0, yy, 16.0, fade(INK, 0.85), Align::Left);
    yy += 22.0;
    for (g, st) in &s.goals {
        ui::mark(x + 28.0, yy + 8.0, 7.0, Some(*st == State::Done));
        gfx::text(&g.text(lang), x + 42.0, yy, 14.0, fade(INK, 0.8), Align::Left);
        if *st == State::Done {
            gfx::text(&format!("+{}", RULES.bonus), x + w - 18.0, yy, 14.0, rgb(0x2F8F83), Align::Right);
        }
        yy += 20.0;
    }
    if s.goals.is_empty() || s.bonus == 0 {
        yy += 0.0;
    }
    yy += 6.0;
    for k in 0..26 {
        gfx::rect(x + 14.0 + k as f32 * 10.0, yy, 6.0, 2.0, fade(INK, 0.3));
    }
    yy += 10.0;
    gfx::text(app.tr(tx::TOTAL), x + 18.0, yy + 6.0, 20.0, INK, Align::Left);
    gfx::text(&format!("{}", s.score), x + w - 18.0, yy, 34.0, if s.outcome.won() { TOMATO } else { fade(INK, 0.5) }, Align::Right);
    if s.outcome == Outcome::Hospital {
        gfx::text(app.tr(tx::VOID), x + w / 2.0, yy + 42.0, 14.0, RED, Align::Center);
    } else if s.outcome.won() && s.best_before.is_none_or(|b| s.score > b) && s.t > 1.0 {
        gfx::text(app.tr(tx::NEW_BEST), x + w - 18.0, yy + 40.0, 16.0, rgb(0x2F8F83), Align::Right);
    }
    // The stamp.
    let stamp = match s.outcome {
        Outcome::Success => (tx::STAMP_WIN, rgb(0x2F8F83)),
        Outcome::Closed => (tx::STAMP_CLOSED, rgb(0x8A8A8A)),
        Outcome::Broke => (tx::STAMP_BROKE, TOMATO),
        Outcome::Hospital => (tx::STAMP_HOSPITAL, RED),
    };
    let k = ((s.t - 0.5) * 4.0).clamp(0.0, 1.0);
    if k > 0.0 {
        let sc = 1.0 + (1.0 - ease_back(k)) * 1.2;
        gfx::transformed(x + w - 70.0, y + h - 60.0, -14.0, sc, sc, || {
            let label = stamp.0.get(lang);
            let sw = gfx::measure(label, 22.0) + 24.0;
            gfx::rrect_lines(-sw / 2.0, -20.0, sw, 40.0, 8.0, 3.0, fade(stamp.1, k));
            gfx::text(label, 0.0, -13.0, 22.0, fade(stamp.1, k), Align::Center);
        });
    }

    // Stars on the right.
    rrect(340.0, 24.0, 276.0, 76.0, 14.0, fade(INK, 0.6));
    for i in 0..3 {
        let at = 0.6 + i as f32 * 0.3;
        let k = ((s.t - at) * 4.0).clamp(0.0, 1.0);
        let on = (i as u32) < s.stars;
        let r = 22.0 * if on { ease_back(k).max(0.3) } else { 1.0 };
        ui::star(410.0 + i as f32 * 68.0, 62.0, r, if on { k } else { 0.0 }, GOLD, fade(CREAM, 0.2));
    }
    for (i, &hi) in s.new_heroes.iter().enumerate() {
        let label = tx::fill(app.tr(tx::NEW_HERO), HEROES[hi].name.get(lang));
        rrect(340.0, 108.0 + i as f32 * 30.0, 276.0, 26.0, 13.0, rgb(HEROES[hi].accent));
        gfx::text(&label, 478.0, 112.0 + i as f32 * 30.0, 16.0, INK, Align::Center);
    }

    // New cards on the right, turning over one by one.
    let ny = 214.0;
    gfx::text(app.tr(tx::NEW_CARDS), 478.0, ny, 18.0, CREAM, Align::Center);
    if s.new_cards.is_empty() {
        gfx::text(app.tr(tx::NO_NEW_CARDS), 478.0, ny + 60.0, 16.0, fade(CREAM, 0.7), Align::Center);
    }
    for (i, &ci) in s.new_cards.iter().enumerate() {
        let start = Summary::reveal_at(i);
        if s.t < start {
            continue;
        }
        let (cx, cy) = rack_slot(i, s.new_cards.len());
        let (cw, chh, _) = rack_layout(s.new_cards.len());
        let drop = ease_out((s.t - start) * 4.0);
        let k = ((s.t - start - 0.25) / 0.4).clamp(0.0, 1.0);
        let card = &CARDS[ci];
        cards::flipping(&app.gl, cx, cy - (1.0 - drop) * 40.0, cw, chh, card, app.portraits.get(card.art), lang, app.t, k);
    }
    if app.save.failed {
        gfx::text(app.tr(tx::SAVE_FAILED), 478.0, 420.0, 14.0, RED, Align::Center);
    }
    app.fx.draw_world();
    let [a, b, x] = app.labels;
    ui::prompts(&[(a, app.tr(tx::AGAIN)), (b, app.tr(tx::ROOMS)), (x, app.tr(tx::ALBUM))], true);
}

// ---- The album ---------------------------------------------------------------

fn album(app: &App) {
    let lang = app.lang;
    gfx::text_outlined(app.tr(tx::ALBUM_TITLE), 24.0, 14.0, 26.0, CREAM, INK, 2.5, Align::Left);
    let owned = CARDS.iter().filter(|c| app.save.has(c.id)).count();
    gfx::text(&format!("{owned}/{}", CARDS.len()), 616.0, 20.0, 18.0, fade(CREAM, 0.8), Align::Right);
    // The books, as tabs.
    let mut x = 24.0;
    for (bi, book) in BOOKS.iter().enumerate() {
        let n = CARDS.iter().filter(|c| c.book == *book).count();
        let got = CARDS.iter().filter(|c| c.book == *book && app.save.has(c.id)).count();
        let label = format!("{} {got}/{n}", book.name().get(lang));
        let w = gfx::measure(&label, 16.0) + 26.0;
        let sel = bi == app.album_book;
        rrect(x, 52.0, w, 30.0, 10.0, if sel { MUSTARD } else { fade(INK, 0.6) });
        gfx::text(&label, x + w / 2.0, 57.0, 16.0, if sel { INK } else { CREAM }, Align::Center);
        if app.save.book_done(*book) {
            ui::star(x + w - 4.0, 54.0, 7.0, 1.0, GOLD, GOLD);
        }
        x += w + 8.0;
    }
    let book = BOOKS[app.album_book];
    let done = app.save.book_done(book);
    let reward = if done { tx::fill(app.tr(tx::BOOK_DONE), BOOK_BONUS) } else { tx::fill(app.tr(tx::BOOK_REWARD), BOOK_BONUS) };
    gfx::text(&reward, 616.0, 58.0, 14.0, if done { GOLD } else { fade(CREAM, 0.6) }, Align::Right);
    let ids = app.book_cards();
    let cols = app.book_cols();
    let (cw, chh, gap) = match cols {
        8 => (66.0, 92.0, 8.0),
        6 => (88.0, 124.0, 12.0),
        _ => (104.0, 146.0, 20.0),
    };
    let total = cols as f32 * cw + (cols - 1) as f32 * gap;
    let x0 = 320.0 - total / 2.0;
    for (k, &ci) in ids.iter().enumerate() {
        let (col, row) = (k % cols, k / cols);
        let (x, y) = (x0 + col as f32 * (cw + gap), 100.0 + row as f32 * (chh + 16.0));
        let card = &CARDS[ci];
        let sel = k == app.album_sel;
        if sel {
            let p = 0.5 + 0.5 * (app.t * 4.0).sin();
            rrect(x - 5.0, y - 5.0, cw + 10.0, chh + 10.0, 12.0, fade(MUSTARD, 0.6 + 0.4 * p));
        }
        if app.save.has(card.id) {
            cards::face(&app.gl, x, y, cw, chh, card, app.portraits.get(card.art), lang, app.t);
        } else {
            cards::back(x, y, cw, chh, true);
        }
    }
    // The selected card's name and condition along the bottom.
    if let Some(&ci) = ids.get(app.album_sel) {
        let card = &CARDS[ci];
        let owned = app.save.has(card.id);
        let hidden = card.rarity == Rarity::Hidden && !owned;
        let name = if hidden { app.tr(tx::HIDDEN_NAME) } else { card.name.get(lang) };
        let cond = if hidden { app.tr(tx::HIDDEN_COND) } else { card.condition.get(lang) };
        ui::panel(24.0, 376.0, 592.0, 56.0);
        gfx::text(name, 38.0, 382.0, 18.0, CREAM, Align::Left);
        let rar = card.rarity.name().get(lang);
        gfx::text(rar, 600.0, 384.0, 14.0, MUSTARD, Align::Right);
        let c = if owned { cond.to_string() } else { format!("{} · {}", app.tr(tx::NOT_YET), cond) };
        gfx::text(&c, 38.0, 406.0, 14.0, fade(CREAM, 0.85), Align::Left);
    }
    if let (Some(ci), Some(t)) = (app.open_card(), app.album_open) {
        open_card(app, ci, t);
    } else {
        let [a, b, _] = app.labels;
        ui::prompts(&[(a, app.tr(tx::VIEW)), ("L1/R1", app.tr(tx::BOOK)), (b, app.tr(tx::BACK))], true);
    }
}

fn open_card(app: &App, ci: usize, t: f32) {
    let lang = app.lang;
    let k = ease_back((t * 4.0).min(1.0));
    gfx::rect(0.0, 0.0, gfx::W, gfx::H, fade(rgb(0x14141C), 0.93 * k.min(1.0)));
    let card = &CARDS[ci];
    let owned = app.save.has(card.id);
    let (w, h) = (216.0, 300.0);
    let wobble = (t * 1.3).sin() * 3.0;
    gfx::transformed(200.0, 236.0, wobble * 0.5, k, k, || {
        if owned {
            cards::face(&app.gl, -w / 2.0, -h / 2.0, w, h, card, app.portraits.detail().or(app.portraits.get(card.art)), lang, app.t);
        } else {
            cards::back(-w / 2.0, -h / 2.0, w, h, true);
        }
    });
    let hidden = card.rarity == Rarity::Hidden && !owned;
    let name = if hidden { app.tr(tx::HIDDEN_NAME) } else { card.name.get(lang) };
    let cond = if hidden { app.tr(tx::HIDDEN_COND) } else { card.condition.get(lang) };
    let a = (t * 4.0).min(1.0);
    gfx::text(card.book.name().get(lang), 330.0, 120.0, 16.0, fade(MUSTARD, a), Align::Left);
    gfx::text(name, 330.0, 142.0, 28.0, fade(CREAM, a), Align::Left);
    let (edge, _) = (card.rarity, 0);
    let rar = edge.name().get(lang);
    rrect(330.0, 182.0, gfx::measure(rar, 15.0) + 18.0, 22.0, 11.0, fade(MUSTARD, a));
    gfx::text(rar, 339.0, 185.0, 15.0, INK, Align::Left);
    gfx::text_wrapped(cond, 330.0, 220.0, 280.0, 17.0, 22.0, fade(CREAM, 0.9 * a), Align::Left);
    if !owned {
        gfx::text(app.tr(tx::NOT_YET), 330.0, 290.0, 16.0, fade(rgb(0xFF8A80), a), Align::Left);
    }
    let [a_, b, _] = app.labels;
    ui::prompts(&[(a_, app.tr(tx::CLOSE)), (b, app.tr(tx::BACK))], true);
}

// ---- The stats panel ---------------------------------------------------------

fn stats(app: &App) {
    let s = &app.stats;
    let n = s.now;
    let (x, y, w) = (12.0, 132.0, 236.0);
    let r = app.round.as_ref().filter(|_| matches!(app.screen, Screen::Dining | Screen::Result));
    let h = if r.is_some() { 250.0 } else { 128.0 };
    rrect(x, y, w, h, 10.0, fade(rgb(0x14131F), 0.86));
    let ms = s.frame_ms();
    let fps = if ms > 0.0 { 1000.0 / ms } else { 0.0 };
    let line = |i: usize, label: &str, v: String| {
        let yy = y + 30.0 + i as f32 * 16.0;
        gfx::text(label, x + 10.0, yy, 13.0, fade(CREAM, 0.65), Align::Left);
        gfx::text(&v, x + w - 10.0, yy, 13.0, CREAM, Align::Right);
    };
    gfx::text(&format!("{fps:.0} FPS  {ms:.1} ms"), x + 10.0, y + 7.0, 16.0, if fps >= 45.0 { MINT } else { MUSTARD }, Align::Left);
    // The frame-time graph.
    let gx = x + 150.0;
    for (i, f) in s.frames.iter().rev().take(40).enumerate() {
        let hh = (f / 40.0 * 18.0).min(18.0);
        let c = if *f > 22.0 { RED } else { MINT };
        gfx::rect(gx + 76.0 - i as f32 * 2.0, y + 26.0 - hh, 1.5, hh, fade(c, 0.8));
    }
    let opt = |v: Option<f32>, unit: &str| v.map_or("—".to_string(), |v| format!("{v:.0}{unit}"));
    line(0, "CPU", format!("{} / {} · {} cores", opt(n.cpu, "%"), opt(n.cpu_self, "%"), n.cores));
    line(1, "MEM", format!("{} free / {}", opt(n.mem_free_mb, "M"), opt(n.mem_total_mb, "M")));
    line(2, "APP", opt(n.mem_self_mb, " MB"));
    let c = app.gl.last.get();
    let room = app.scene.as_ref().map_or(0, |s| s.tris);
    line(3, "GPU", format!("{} draws · {}k tris ({}k room)", c.draws, c.tris / 1000, room / 1000));
    let gl = match unsafe { ffi::rlGetVersion() } {
        v if v == ffi::rlGlVersion::RL_OPENGL_ES_30 as i32 => "GLES 3",
        v if v == ffi::rlGlVersion::RL_OPENGL_ES_20 as i32 => "GLES 2",
        v if v == ffi::rlGlVersion::RL_OPENGL_33 as i32 => "GL 3.3",
        _ => "GL",
    };
    line(4, "TEMP/CLK", format!("{} · {} · {gl}", opt(n.temp_c, "°C"), opt(n.mhz, " MHz")));
    if let Some(r) = r {
        let hd = r.hero_def();
        let d = r.def();
        line(6, "ROOM", format!("{} · seed {}", d.id, app.save.meals));
        line(7, "HERO", format!("{} ×{:.2} stomach", hd.id, hd.stomach));
        line(8, "DIGEST/WALK", format!("{:.1}%/s · {:.0} cm/s", hd.digest, hd.walk));
        line(9, "FULL/RED", format!("{:.1}% · {:.1}s/{:.0}s", r.fullness, r.danger.unwrap_or(0.0), hd.hospital_s));
        line(10, "BELT", format!("{:.0} cm/s · {} plates", RULES.belt_speed, r.plates.len()));
        line(11, "SCORE", format!("{} + {} bonus · ×{:.2}", r.tally.score, r.bonus, r.multiplier()));
        line(12, "LEAN/DARK", format!("{:+.2} · {:.1}", r.lean, r.layout.dark));
    }
}
