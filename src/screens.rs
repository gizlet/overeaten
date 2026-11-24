//! Drawing each screen on the 640×480 canvas.

use crate::app::{App, Screen, Summary};
use crate::art::{self, pal::*, Pose};
use crate::game::{floor::*, Facing, Outcome, Round};
use crate::gfx::{self, ease_back, ease_out, fade, mix, rgb, rrect, Align};
use crate::rules::*;
use crate::text as tx;
use raylib_sys::Color;
use std::f32::consts::PI;

pub fn draw(app: &App) {
    if app.icon {
        icon(app.t);
        return;
    }
    match app.screen {
        Screen::Title => title(app),
        Screen::Street => street(app),
        Screen::Dining => dining(app),
        Screen::Result => result(app),
        Screen::Album => album(app),
    }
    if app.fade > 0.0 {
        gfx::rect(0.0, 0.0, gfx::W, gfx::H, fade(INK, app.fade * app.fade));
    }
}

/// Button prompts along the bottom of the safe area, right-aligned.
fn prompts(items: &[(&str, &str)], light: bool) {
    let size = 24.0;
    let (fill, ink, label) = if light { (CREAM, INK, CREAM) } else { (INK, CREAM, INK) };
    let widths: Vec<f32> = items
        .iter()
        .map(|(b, a)| gfx::measure(b, size * 0.72).max(size * 0.5) + size * 0.55 + 6.0 + gfx::measure(a, 18.0))
        .collect();
    let total: f32 = widths.iter().sum::<f32>() + 18.0 * (items.len() as f32 - 1.0);
    let mut x = 624.0 - total;
    for ((b, a), w) in items.iter().zip(widths) {
        let bw = art::button_badge(b, x, 444.0, size, fill, ink);
        gfx::text(a, x + bw + 6.0, 446.0, 18.0, label, Align::Left);
        x += w + 18.0;
    }
}

fn dotted_backdrop(top: Color, bottom: Color, t: f32) {
    gfx::grad_v(0.0, 0.0, gfx::W, gfx::H, top, bottom);
    let off = (t * 8.0).rem_euclid(32.0);
    for row in -1..17 {
        for col in -1..22 {
            let x = col as f32 * 32.0 + off + if row % 2 == 0 { 0.0 } else { 16.0 };
            let y = row as f32 * 32.0 + off;
            gfx::circle(x, y, 2.2, fade(CREAM, 0.12));
        }
    }
}

/// The app icon: the visitor mid-bite, in the 480×480 square in the middle of the canvas.
fn icon(t: f32) {
    dotted_backdrop(rgb(0xEA6A3F), rgb(0xC4402A), 0.0);
    gfx::glow(320.0, 230.0, 230.0, fade(rgb(0xFFD08A), 0.55));
    let pose = Pose { eat: Some(0.5), ..Pose::idle(Facing::Down, t) };
    art::visitor(320.0, 440.0, 1.5, &pose);
}

// ---- Title -------------------------------------------------------------

fn title(app: &App) {
    let t = app.t;
    dotted_backdrop(rgb(0xEA6A3F), rgb(0xC4402A), t);
    gfx::glow(320.0, 330.0, 260.0, fade(rgb(0xFFD08A), 0.45));

    // The logo, each character bouncing in turn.
    let chars: Vec<char> = "吃吃吃".chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        let k = ((t * 3.2 - i as f32 * 0.55).sin()).max(0.0);
        let x = 214.0 + i as f32 * 106.0;
        let y = 38.0 - k * 12.0;
        let s = ch.to_string();
        gfx::text(&s, x + 4.0, y + 6.0, 92.0, fade(rgb(0x7A2416), 0.6), Align::Center);
        gfx::text_outlined(&s, x, y, 92.0, CREAM, INK, 4.0, Align::Center);
    }
    gfx::text_outlined("OVEREATEN", 320.0, 146.0, 30.0, MUSTARD, INK, 2.5, Align::Center);
    gfx::text(app.tr(tx::TAGLINE), 320.0, 186.0, 18.0, CREAM, Align::Center);

    // 小馋 at a little table, plates circling.
    let (cx, cy) = (320.0, 350.0);
    let orbit = |i: usize| {
        let a = t * 0.9 + i as f32 * PI * 2.0 / 6.0;
        (cx + a.cos() * 150.0, cy - 6.0 + a.sin() * 42.0, a.sin())
    };
    for i in 0..6 {
        let (x, y, z) = orbit(i);
        if z < 0.0 {
            art::dish(x, y, i, 1.5);
        }
    }
    ellipse_table(cx, cy + 34.0);
    let k = (t * 0.6).fract();
    let pose =
        Pose { eat: if k > 0.45 { Some((k - 0.45) / 0.55) } else { None }, belly: 0.9, ..Pose::idle(Facing::Down, t) };
    art::visitor(cx, cy + 28.0, 0.78, &pose);
    for i in 0..6 {
        let (x, y, z) = orbit(i);
        if z >= 0.0 {
            art::dish(x, y, i, 1.5);
        }
    }

    let owned = CARDS.iter().filter(|c| app.save.has(c.id)).count();
    rrect(16.0, 16.0, 112.0, 32.0, 16.0, fade(INK, 0.7));
    gfx::text(&format!("{} {owned}/{}", app.tr(tx::COLLECTED), CARDS.len()), 72.0, 21.0, 18.0, CREAM, Align::Center);
    let [a, _, x] = app.labels;
    prompts(&[(a, app.tr(tx::START)), (x, app.tr(tx::ALBUM)), ("Select", app.tr(tx::LANGUAGE))], true);
}

fn ellipse_table(x: f32, y: f32) {
    gfx::ellipse(x, y + 14.0, 120.0, 22.0, fade(INK, 0.25));
    gfx::ellipse(x, y + 6.0, 116.0, 26.0, WALNUT_DARK);
    gfx::ellipse(x, y, 116.0, 26.0, WALNUT);
    gfx::ellipse(x, y - 1.0, 108.0, 22.0, rgb(0xA06A45));
}

// ---- The street --------------------------------------------------------

fn street(app: &App) {
    let t = app.t;
    gfx::grad_v(0.0, 0.0, gfx::W, 330.0, rgb(0x27314F), rgb(0x6B5B95));
    gfx::grad_v(0.0, 330.0, gfx::W, 150.0, rgb(0x4A4458), rgb(0x2C2838));
    for i in 0..9 {
        gfx::rect(0.0, 340.0 + i as f32 * 16.0, gfx::W, 1.0, fade(CREAM, 0.06));
    }
    // String lights.
    for i in 0..17 {
        let x = i as f32 * 40.0 + 8.0;
        let y = 64.0 + ((x - 320.0) / 320.0).powi(2) * -18.0 + 18.0;
        let on = 0.6 + 0.4 * (t * 2.0 + i as f32 * 0.9).sin();
        gfx::glow(x, y, 16.0, fade(rgb(0xFFD27F), 0.5 * on));
        gfx::circle(x, y, 3.0, mix(rgb(0xFFB45E), rgb(0xFFF1C1), on));
    }
    gfx::curve((0.0, 64.0), (320.0, 100.0), (640.0, 64.0), 1.5, fade(INK, 0.8));
    gfx::text_outlined(app.tr(tx::PICK), 320.0, 16.0, 30.0, CREAM, INK, 2.5, Align::Center);

    for (i, r) in RESTAURANTS.iter().enumerate() {
        let d = i as f32 - app.street_pos;
        let sel = 1.0 - d.abs().min(1.0);
        let s = 0.8 + 0.2 * sel;
        let cx = 320.0 + d * 210.0;
        let cy = 246.0 - sel * 6.0 + (t * 2.0 + i as f32).sin() * sel * 2.0;
        gfx::transformed(cx, cy, 0.0, s, s, || ticket(app, i, r, sel));
    }
    let [a, b, x] = app.labels;
    let r = &RESTAURANTS[app.street_sel];
    if r.open {
        let enter = format!("{} {}", app.tr(tx::ENTER), r.admission);
        prompts(&[(a, &enter), (b, app.tr(tx::BACK)), (x, app.tr(tx::ALBUM))], true);
    } else {
        prompts(&[(b, app.tr(tx::BACK)), (x, app.tr(tx::ALBUM))], true);
    }
}

/// A restaurant's ticket, centered on the origin.
fn ticket(app: &App, i: usize, r: &Restaurant, sel: f32) {
    let (w, h) = (190.0, 300.0);
    let (x, y) = (-w / 2.0, -h / 2.0);
    let accent = rgb(r.accent);
    rrect(x + 4.0, y + 8.0, w, h, 14.0, fade(INK, 0.4));
    rrect(x, y, w, h, 14.0, PAPER);
    // Awning.
    gfx::transformed(0.0, 0.0, 0.0, 1.0, 1.0, || {
        rrect(x, y, w, 40.0, 14.0, accent);
        for k in 0..6 {
            if k % 2 == 1 {
                gfx::rect(x + k as f32 * w / 6.0, y + 10.0, w / 6.0, 30.0, CREAM);
            }
        }
        for k in 0..6 {
            let c = if k % 2 == 0 { accent } else { CREAM };
            gfx::circle(x + (k as f32 + 0.5) * w / 6.0, y + 40.0, w / 12.0, c);
        }
    });
    art::storefront(i, 0.0, y + 98.0, app.t);
    gfx::text(app.tr(r.name), 0.0, y + 140.0, 26.0, INK, Align::Center);
    gfx::text(app.tr(r.way), 0.0, y + 174.0, 16.0, fade(INK, 0.75), Align::Center);
    let mut dx = x + 14.0;
    while dx < x + w - 14.0 {
        gfx::rect(dx, y + 202.0, 6.0, 2.0, fade(INK, 0.3));
        dx += 11.0;
    }
    gfx::text(app.tr(tx::TASK), x + 14.0, y + 212.0, 16.0, accent, Align::Left);
    gfx::text_wrapped(app.tr(r.task), x + 14.0, y + 232.0, w - 28.0, 17.0, 22.0, INK, Align::Left);
    art::coin(x + 26.0, y + h - 22.0, 9.0);
    gfx::text(&format!("{} {}", app.tr(tx::ADMISSION), r.admission), x + 42.0, y + h - 33.0, 18.0, INK, Align::Left);
    if !r.open {
        rrect(x, y, w, h, 14.0, fade(INK, 0.55));
        art::lock(0.0, y + 70.0, CREAM);
        rrect(-70.0, y + 96.0, 140.0, 34.0, 17.0, fade(INK, 0.85));
        gfx::text(app.tr(tx::SOON), 0.0, y + 101.0, 20.0, CREAM, Align::Center);
    }
    if sel > 0.5 {
        gfx::rrect_lines(x - 5.0, y - 5.0, w + 10.0, h + 10.0, 18.0, 3.0, fade(MUSTARD, sel));
    }
}

// ---- Dining ------------------------------------------------------------

enum Thing {
    Island,
    Plate(usize),
    Visitor,
    Booth,
    Yuzu,
    Condiments,
    Tea,
}

/// The plate's drawing scale on the stage.
const PLATE_SCALE: f32 = 0.31;
/// The visitor's art scale on the stage.
const VISITOR_SCALE: f32 = 0.5;

fn dining(app: &App) {
    let Some(r) = &app.round else { return };
    let t = app.t;
    let reach = r.reachable();
    art::on_stage(|| {
        art::room(t);
        let mut things: Vec<(f32, Thing)> = vec![
            (CY - OUTER, Thing::Island),
            (r.player.pos.y, Thing::Visitor),
            (BOOTH.1 + 40.0, Thing::Booth),
            (YUZU.1 + 40.0, Thing::Yuzu),
            (CY + 14.0, Thing::Condiments),
            (CY + 14.0, Thing::Tea),
        ];
        for (i, p) in r.plates.iter().enumerate() {
            things.push((p.pos().y + 0.5, Thing::Plate(i)));
        }
        things.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, th) in &things {
            match th {
                Thing::Island => art::island(r.belt_travel),
                Thing::Booth => art::booth(),
                Thing::Yuzu => art::yuzu(),
                Thing::Condiments => art::condiments(),
                Thing::Tea => art::tea(),
                Thing::Plate(i) => plate(app, r, *i, reach == Some(*i)),
                Thing::Visitor => visitor(app, r),
            }
        }
        morsel(r);
    });

    // The plate within reach: a bubble naming the dish and its worth.
    if let Some(i) = reach {
        let (px, py) = art::canvas(r.plates[i].pos(), COUNTER_H);
        let d = &DISHES[r.plates[i].dish];
        let by = py - 52.0 - (t * 5.0).sin().abs() * 3.0;
        let [a, ..] = app.labels;
        let name = app.tr(d.name);
        let worth = format!("+{}", d.coins);
        let w = gfx::measure(a, 13.0).max(10.0) + 11.0 + gfx::measure(name, 16.0) + gfx::measure(&worth, 16.0) + 26.0;
        let bx = (px - w / 2.0).clamp(8.0, gfx::W - 8.0 - w);
        rrect(bx + 2.0, by + 3.0, w, 28.0, 14.0, fade(INK, 0.35));
        rrect(bx, by, w, 28.0, 14.0, fade(HOUSE, 0.96));
        gfx::rrect_lines(bx, by, w, 28.0, 14.0, 1.5, fade(CREAM, 0.6));
        gfx::tri((px - 6.0, by + 27.0), (px + 6.0, by + 27.0), (px, by + 34.0), fade(HOUSE, 0.96));
        let aw = art::button_badge(a, bx + 5.0, by + 5.0, 18.0, MUSTARD, INK);
        let nw = gfx::text(name, bx + aw + 10.0, by + 4.0, 16.0, CREAM, Align::Left);
        gfx::text(&worth, bx + aw + 15.0 + nw, by + 4.0, 16.0, MUSTARD, Align::Left);
    }

    app.fx.draw_world();

    // Danger: a slow red pulse at the edges and the countdown overhead.
    if let Some(d) = r.danger {
        let pulse = 0.5 + 0.5 * (t * PI * 2.0).sin();
        let a = 0.25 + 0.2 * pulse;
        gfx::grad_v(0.0, 0.0, gfx::W, 70.0, fade(RED, a), fade(RED, 0.0));
        gfx::grad_v(0.0, 410.0, gfx::W, 70.0, fade(RED, 0.0), fade(RED, a));
        gfx::grad_h(0.0, 0.0, 70.0, gfx::H, fade(RED, a), fade(RED, 0.0));
        gfx::grad_h(570.0, 0.0, 70.0, gfx::H, fade(RED, 0.0), fade(RED, a));
        let left = (RULES.hospital_s - d).max(0.0);
        let (hx, hy) = art::canvas(r.player.pos, 0.0);
        let hy = hy - 92.0;
        gfx::circle(hx, hy, 20.0, fade(INK, 0.85));
        gfx::ring(hx, hy, 17.0, 20.0, -90.0, -90.0 + 360.0 * left / RULES.hospital_s, RED);
        let n = format!("{}", left.ceil() as i32);
        let pop = ease_back(1.0 - left.fract());
        gfx::text_outlined(&n, hx, hy - 13.0 * pop, 26.0 * pop, CREAM, INK, 2.0, Align::Center);
    }

    // Outros: confetti, dimmed lights or the ambulance.
    if let (Some(o), t) = (r.outcome, r.outro) {
        match o {
            Outcome::Closed => gfx::rect(0.0, 0.0, gfx::W, gfx::H, fade(INK, (t / 1.5).min(1.0) * 0.45)),
            Outcome::Hospital => {
                let (px, py) = art::canvas(r.player.pos, 0.0);
                let x = 720.0 - ease_out(t / 1.6) * (720.0 - px - 46.0);
                gfx::transformed(x, py - 14.0, 0.0, 1.6, 1.6, || ambulance(t));
            }
            Outcome::Done => {
                for i in 0..24 {
                    let k = (t * 0.6 + i as f32 * 0.37).fract();
                    let x = (i as f32 * 53.0 + (t + i as f32).sin() * 20.0).rem_euclid(gfx::W);
                    let c = [MUSTARD, TOMATO, MINT, SKY, CREAM][i % 5];
                    gfx::transformed(x, k * 500.0 - 20.0, t * 200.0 + i as f32 * 40.0, 1.0, 1.0, || {
                        gfx::rect(-3.0, -2.0, 6.0, 4.0, c)
                    });
                }
            }
        }
    }

    hud(app, r);
    app.fx.draw_banner();
    if let Some(sel) = app.paused {
        pause(app, r, sel);
    }
}

/// A plate on the belt (stage), popping on when served and fading back to the kitchen.
fn plate(app: &App, r: &Round, i: usize, near: bool) {
    let p = &r.plates[i];
    let (x, y) = art::stage(p.pos(), COUNTER_H);
    let age = (p.age / 0.48).clamp(0.0, 1.0);
    let pop = ease_back(age).max(0.05);
    let drop = (1.0 - ease_out(age)) * 70.0;
    let s = PLATE_SCALE * pop * (1.0 - p.leaving).max(0.0);
    if p.dish == GOLDEN {
        let g = 70.0 + 6.0 * (app.t * 4.0).sin();
        gfx::additive(|| gfx::glow(x, y - 10.0, g, fade(rgb(0xFFC85A), 0.47)));
    }
    if near {
        let k = 0.5 + 0.5 * (app.t * 6.0).sin();
        for j in 0..3 {
            let rr = 58.0 + j as f32 * 5.0 + k * 4.0;
            gfx::ring(x, y + 4.0, rr, rr + 3.0, 0.0, 360.0, fade(MUSTARD, 0.9 - j as f32 * 0.3));
        }
    }
    let tint = fade(gfx::WHITE, 1.0 - p.leaving);
    art::dish_stack(p.dish, x, y - drop, s, tint, true);
}

fn ambulance(t: f32) {
    let flash = (t * 5.0).sin() > 0.0;
    gfx::ellipse(0.0, 16.0, 36.0, 6.0, fade(INK, 0.3));
    gfx::glow(-6.0, -24.0, 18.0, fade(if flash { rgb(0x5AA9E6) } else { RED }, 0.9));
    rrect(-30.0, -18.0, 44.0, 32.0, 4.0, WHITE);
    rrect(-46.0, -6.0, 20.0, 20.0, 4.0, WHITE);
    rrect(-43.0, -3.0, 11.0, 8.0, 2.0, rgb(0x9CC9E8));
    gfx::rect(-46.0, 2.0, 60.0, 4.0, RED);
    gfx::rect(-11.0, -14.0, 6.0, 16.0, RED);
    gfx::rect(-16.0, -9.0, 16.0, 6.0, RED);
    rrect(-10.0, -24.0, 8.0, 6.0, 2.0, if flash { rgb(0x5AA9E6) } else { RED });
    for wx in [-30.0, 4.0] {
        gfx::circle(wx, 14.0, 6.0, INK);
        gfx::circle(wx, 14.0, 2.5, rgb(0xC9C2B2));
    }
}

fn visitor(app: &App, r: &Round) {
    let pl = &r.player;
    let eat = pl.eating.as_ref().map(|e| (e.t / RULES.eat_s).min(1.0));
    let from_left = pl.eating.as_ref().is_some_and(|e| e.from.x < pl.pos.x);
    let pose = Pose {
        facing: pl.facing,
        walking: pl.moving,
        walk: pl.walk,
        eat,
        from_left,
        belly: app.shown_fullness / RULES.capacity,
        red: r.danger.is_some() || r.outcome == Some(Outcome::Hospital),
        t: app.t,
    };
    let (x, y) = art::stage(pl.pos, 0.0);
    let hop = if r.outcome == Some(Outcome::Done) { (r.outro * 8.0).sin().abs() * 16.0 } else { 0.0 };
    art::visitor(x, y - hop, VISITOR_SCALE, &pose);
}

/// The bite on its way from the plate to the visitor's mouth (stage).
fn morsel(r: &Round) {
    let Some(e) = &r.player.eating else { return };
    let k = e.t / RULES.bite_s;
    if k >= 1.0 {
        return;
    }
    let (fx, fy) = art::stage(e.from, COUNTER_H + 10.0);
    let (vx, vy) = art::stage(r.player.pos, 0.0);
    let (mx, my) = art::mouth(VISITOR_SCALE);
    let (tx, ty) = (vx + mx, vy + my);
    let ease = ease_out(k);
    let (x, y) = (fx + (tx - fx) * ease, fy + (ty - fy) * ease - (k * PI).sin() * 50.0);
    art::held(e.dish, x, y, 64.0 * (1.0 - 0.3 * ease));
}

fn hud(app: &App, r: &Round) {
    let t = app.t;
    let light = rgb(0xF6EAD0);
    // Top right: time to closing.
    let left = r.time_left();
    let urgent = left < 15.0 && r.outcome.is_none();
    let pulse = if urgent { 1.0 + 0.06 * (t * 6.0).sin() } else { 1.0 };
    art::panel(514.0, 10.0, 114.0, 44.0);
    art::hud_icon(true, 538.0, 32.0, 30.0, gfx::WHITE);
    gfx::ring(
        538.0,
        32.0,
        16.0,
        18.0,
        -90.0,
        -90.0 + 360.0 * left / RULES.duration,
        if urgent { TOMATO } else { CELADON },
    );
    let secs = left.ceil() as i32;
    let tc = if urgent { rgb(0xFF8A6B) } else { light };
    gfx::transformed(590.0, 32.0, 0.0, pulse, pulse, || {
        gfx::text(&format!("{}:{:02}", secs / 60, secs % 60), 0.0, -15.0, 26.0, tc, Align::Center);
    });

    // Bottom left: the task, coins eaten against admission.
    let (x0, y0, w0) = (12.0, 414.0, 220.0);
    art::panel(x0, y0, w0, 56.0);
    gfx::text(app.tr(RESTAURANTS[0].task), x0 + 14.0, y0 + 7.0, 15.0, fade(light, 0.85), Align::Left);
    let bump = 1.0 + 0.3 * app.fx.bump_coins;
    art::coin(x0 + 22.0, y0 + 38.0, 9.0 * bump);
    let (bx, bw) = (x0 + 38.0, 96.0);
    rrect(bx, y0 + 32.0, bw, 12.0, 6.0, fade(INK, 0.55));
    let frac = (app.shown_coins / RULES.admission as f32).clamp(0.0, 1.0);
    if frac > 0.0 {
        rrect(bx, y0 + 32.0, (bw * frac).max(12.0), 12.0, 6.0, MUSTARD);
        rrect(bx + 3.0, y0 + 33.5, (bw * frac - 6.0).max(6.0), 3.5, 1.75, fade(WHITE, 0.4));
    }
    let label = format!("{}/{}", app.shown_coins.round() as i32, RULES.admission);
    gfx::text(&label, x0 + w0 - 12.0, y0 + 24.0, 24.0, light, Align::Right);
    if r.combo >= 2 && r.combo_left() > 0.0 {
        art::panel(x0, y0 - 34.0, 124.0, 28.0);
        rrect(x0 + 10.0, y0 - 11.0, 104.0 * r.combo_left() / RULES.combo_window, 2.5, 1.2, fade(rgb(0xFF8FA3), 0.9));
        gfx::text(
            &format!("{} ×{}", app.tr(tx::COMBO), r.combo),
            x0 + 62.0,
            y0 - 31.0,
            17.0,
            rgb(0xFFB3C1),
            Align::Center,
        );
    }

    // Bottom middle and right: plates and the stomach.
    let danger = r.danger.is_some();
    let (x1, y1, w1) = (240.0, 414.0, 388.0);
    art::panel(x1, y1, w1, 56.0);
    if danger {
        let a = 0.5 + 0.5 * (t * PI * 2.0).sin();
        gfx::rrect_lines(x1 - 2.0, y1 - 2.0, w1 + 4.0, 60.0, 14.0, 3.0, fade(RED, a));
    }
    let pb = 1.0 + 0.25 * app.fx.bump_plates;
    art::hud_icon(false, x1 + 28.0, y1 + 28.0, 34.0 * pb, gfx::WHITE);
    gfx::text(&format!("{}", r.plates_eaten), x1 + 50.0, y1 + 14.0, 26.0, light, Align::Left);
    let (bx, bw, by, bh) = (x1 + 92.0, 230.0, y1 + 30.0, 16.0);
    let per = bw / 150.0;
    rrect(bx, by, bw, bh, 8.0, fade(INK, 0.6));
    gfx::rect(bx + 100.0 * per, by + 2.0, 20.0 * per, bh - 4.0, fade(MUSTARD, 0.22));
    rrect(bx + 120.0 * per, by, 30.0 * per, bh, 8.0, fade(rgb(0x5A2A28), 0.9));
    let f = app.shown_fullness.clamp(0.0, 150.0);
    let fill = if f >= RULES.red_line {
        RED
    } else if f >= RULES.capacity {
        rgb(0xF28C28)
    } else if f >= 70.0 {
        MUSTARD
    } else {
        CELADON
    };
    if f > 0.5 {
        rrect(bx, by, (f * per).max(bh), bh, 8.0, fill);
        rrect(bx + 3.0, by + 2.0, (f * per - 6.0).max(8.0), 4.0, 2.0, fade(WHITE, 0.35));
    }
    // What the plate in reach would add.
    if let Some(i) = r.reachable() {
        let add = DISHES[r.plates[i].dish].fullness as f32;
        let from = r.fullness.min(150.0);
        let to = (r.fullness + add).min(150.0);
        let c = if r.fullness + add > RULES.red_line { RED } else { light };
        gfx::rect(bx + from * per, by + 3.0, (to - from) * per, bh - 6.0, fade(c, 0.45 + 0.25 * (t * 6.0).sin()));
    }
    gfx::rect(bx + 100.0 * per - 1.0, by - 4.0, 2.0, bh + 8.0, light);
    gfx::tri(
        (bx + 120.0 * per - 5.0, by - 7.0),
        (bx + 120.0 * per + 5.0, by - 7.0),
        (bx + 120.0 * per, by - 1.0),
        MUSTARD,
    );
    let label = format!("{} {}%", app.tr(tx::FULLNESS), f.round() as i32);
    gfx::text(&label, bx, y1 + 7.0, 15.0, if danger { rgb(0xFF9A9A) } else { fade(light, 0.85) }, Align::Left);
}

fn pause(app: &App, r: &Round, sel: usize) {
    gfx::rect(0.0, 0.0, gfx::W, gfx::H, fade(INK, 0.6));
    let (x, y, w, h) = (170.0, 110.0, 300.0, 260.0);
    rrect(x + 4.0, y + 8.0, w, h, 18.0, fade(INK, 0.5));
    rrect(x, y, w, h, 18.0, PAPER);
    gfx::text(app.tr(tx::PAUSED), 320.0, y + 18.0, 30.0, INK, Align::Center);
    let secs = r.time_left().ceil() as i32;
    let info =
        format!("{} {}:{:02}   ·   {} / {}", app.tr(tx::TIME_LEFT), secs / 60, secs % 60, r.coins, RULES.admission);
    gfx::text(&info, 320.0, y + 60.0, 17.0, fade(INK, 0.7), Align::Center);
    for (i, item) in [tx::CONTINUE, tx::LEAVE].iter().enumerate() {
        let iy = y + 98.0 + i as f32 * 52.0;
        let on = i == sel;
        rrect(x + 30.0, iy, w - 60.0, 42.0, 21.0, if on { TOMATO } else { fade(INK, 0.08) });
        gfx::text(app.tr(*item), 320.0, iy + 9.0, 22.0, if on { CREAM } else { INK }, Align::Center);
    }
    gfx::text(app.tr(tx::QUIT_HINT), 320.0, y + h - 34.0, 16.0, fade(INK, 0.6), Align::Center);
}

// ---- The bill ----------------------------------------------------------

/// Where card `i` of `n` new cards settles in the result rack (top-left).
pub fn rack_slot(i: usize, n: usize) -> (f32, f32) {
    let per_row = 3;
    let row = i / per_row;
    let in_row = if row + 1 < n.div_ceil(per_row) { per_row } else { n - row * per_row };
    let col = i % per_row;
    let w = in_row as f32 * 96.0 + (in_row as f32 - 1.0) * 12.0;
    (176.0 - w / 2.0 + col as f32 * 108.0, 88.0 + row as f32 * 160.0)
}

fn result(app: &App) {
    let Some(s) = &app.summary else { return };
    let t = s.t;
    let (top, bottom) = match s.outcome {
        Outcome::Done => (rgb(0xEA6A3F), rgb(0xC4402A)),
        Outcome::Closed => (rgb(0x7A5240), rgb(0x4E3326)),
        Outcome::Hospital => (rgb(0x46557A), rgb(0x2A3350)),
    };
    dotted_backdrop(top, bottom, app.t);

    // Left: the rack of new cards.
    gfx::text_outlined(app.tr(tx::NEW_CARDS), 176.0, 30.0, 28.0, CREAM, INK, 2.0, Align::Center);
    let pose = Pose {
        eat: if s.outcome == Outcome::Done { Some(0.8 + 0.2 * (app.t * 0.8).sin()) } else { None },
        belly: s.peak / 100.0,
        red: s.outcome == Outcome::Hospital,
        ..Pose::idle(Facing::Down, app.t)
    };
    if s.new_cards.is_empty() {
        art::visitor(176.0, 330.0, 0.72, &pose);
        gfx::text(app.tr(tx::NO_NEW_CARDS), 176.0, 360.0, 18.0, CREAM, Align::Center);
    } else if s.new_cards.len() <= 3 {
        let hop = if s.outcome == Outcome::Done { (app.t * 4.0).sin().abs() * 6.0 } else { 0.0 };
        art::visitor(176.0, 420.0 - hop, 0.6, &pose);
    }
    for (i, &c) in s.new_cards.iter().enumerate() {
        let start = Summary::reveal_at(i);
        let k = t - start;
        if k < 0.0 {
            continue;
        }
        let (x, y) = rack_slot(i, s.new_cards.len());
        let slide = ease_out(k / 0.35);
        let yy = y + (1.0 - slide) * 300.0;
        let flip = ((k - 0.25) / 0.3).clamp(0.0, 1.0);
        let face = flip >= 0.5;
        let sx = ((flip * PI).cos()).abs().max(0.04);
        let card = &CARDS[c];
        let lift = if face { 1.0 + 0.08 * (1.0 - ((k - 0.55) / 0.3).clamp(0.0, 1.0)) } else { 1.0 };
        gfx::transformed(x + 48.0, yy + 67.0, 0.0, sx * lift, lift, || {
            if face {
                art::card_face(-48.0, -67.0, 96.0, 134.0, card.art, rgb(card.band), app.tr(card.name), app.t);
            } else {
                art::card_back(-48.0, -67.0, 96.0, 134.0, false);
            }
        });
    }

    // Right: the bill.
    let (x, y, w, h) = (364.0, 22.0, 252.0, 392.0);
    let drop = ease_out(t / 0.4);
    let y = y - (1.0 - drop) * 440.0;
    rrect(x + 4.0, y + 6.0, w, h, 6.0, fade(INK, 0.35));
    rrect(x, y, w, h, 6.0, PAPER);
    for k in 0..14 {
        let zx = x + k as f32 * w / 14.0;
        gfx::tri((zx, y + h - 1.0), (zx + w / 14.0, y + h - 1.0), (zx + w / 28.0, y + h + 7.0), PAPER);
    }
    gfx::text(app.tr(RESTAURANTS[0].name), x + w / 2.0, y + 14.0, 24.0, INK, Align::Center);
    gfx::text(app.tr(tx::BILL), x + w / 2.0, y + 44.0, 16.0, fade(INK, 0.6), Align::Center);
    let dash = |yy: f32| {
        let mut dx = x + 14.0;
        while dx < x + w - 14.0 {
            gfx::rect(dx, yy, 6.0, 2.0, fade(INK, 0.3));
            dx += 11.0;
        }
    };
    dash(y + 70.0);
    let row = |i: usize, k: &str, v: &str, c: Color| {
        let ry = y + 80.0 + i as f32 * 30.0;
        gfx::text(k, x + 18.0, ry, 18.0, fade(INK, 0.8), Align::Left);
        gfx::text(v, x + w - 18.0, ry, 20.0, c, Align::Right);
    };
    let balance = s.coins - RULES.admission;
    row(0, app.tr(tx::FOOD_VALUE), &s.coins.to_string(), INK);
    row(1, app.tr(tx::ADMISSION), &format!("-{}", RULES.admission), INK);
    let bc = if balance >= 0 { rgb(0x2E8B57) } else { RED };
    row(2, app.tr(tx::BALANCE), &format!("{balance:+}"), bc);
    row(3, app.tr(tx::PLATES), &s.plates.to_string(), INK);
    row(4, app.tr(tx::PEAK), &format!("{}%", s.peak.round() as i32), if s.peak > RULES.red_line { RED } else { INK });
    dash(y + 234.0);
    // What was eaten.
    for (i, n) in s.eaten.iter().enumerate() {
        let (cx, cy) = (x + 30.0 + (i % 4) as f32 * 58.0, y + 262.0 + (i / 4) as f32 * 48.0);
        let a = if *n > 0 { 1.0 } else { 0.25 };
        if *n > 0 {
            art::dish(cx, cy, i, 1.15);
        } else {
            art::plate(cx, cy, i, 1.15, a);
        }
        gfx::text(&format!("×{n}"), cx + 20.0, cy + 2.0, 16.0, fade(INK, if *n > 0 { 0.9 } else { 0.35 }), Align::Left);
    }

    // The stamp.
    let k = ((t - 0.5) / 0.25).clamp(0.0, 1.0);
    if k > 0.0 {
        let (word, c) = match s.outcome {
            Outcome::Done => (tx::STAMP_DONE, rgb(0xC0392B)),
            Outcome::Closed => (tx::STAMP_CLOSED, rgb(0x6B5B4B)),
            Outcome::Hospital => (tx::STAMP_HOSPITAL, rgb(0x2F5DA8)),
        };
        let sc = 2.2 - 1.2 * ease_out(k);
        gfx::transformed(x + w / 2.0, y + 360.0, -8.0, sc * 0.9, sc * 0.9, || {
            let a = k;
            gfx::rrect_lines(-62.0, -22.0, 124.0, 44.0, 10.0, 3.5, fade(c, 0.9 * a));
            gfx::rrect_lines(-56.0, -16.0, 112.0, 32.0, 7.0, 1.5, fade(c, 0.9 * a));
            gfx::text(app.tr(word), 0.0, -14.0, 24.0, fade(c, 0.9 * a), Align::Center);
        });
    }

    if app.save.failed {
        gfx::text(app.tr(tx::SAVE_FAILED), 176.0, 420.0, 16.0, rgb(0xFFB0B0), Align::Center);
    }
    let [a, b, x] = app.labels;
    prompts(&[(a, app.tr(tx::AGAIN)), (b, app.tr(tx::STREET)), (x, app.tr(tx::ALBUM))], true);
}

// ---- The album ---------------------------------------------------------

fn album_slot(i: usize) -> (f32, f32) {
    let (w, gap) = (104.0, 32.0);
    if i < 4 {
        (320.0 - (4.0 * w + 3.0 * gap) / 2.0 + i as f32 * (w + gap), 82.0)
    } else {
        (320.0 - (3.0 * w + 2.0 * gap) / 2.0 + (i - 4) as f32 * (w + gap), 252.0)
    }
}

fn album(app: &App) {
    let t = app.t;
    gfx::grad_v(0.0, 0.0, gfx::W, gfx::H, rgb(0xF7EBD3), rgb(0xEAD7B7));
    for row in 0..16 {
        for col in 0..22 {
            gfx::circle(col as f32 * 30.0 + 10.0, row as f32 * 30.0 + 10.0, 1.2, fade(WALNUT, 0.18));
        }
    }
    gfx::text(app.tr(tx::ALBUM_TITLE), 24.0, 18.0, 30.0, INK, Align::Left);
    let owned = CARDS.iter().filter(|c| app.save.has(c.id)).count();
    let tab = format!("{}  {owned}/{}", app.tr(RESTAURANTS[0].name), CARDS.len());
    let tw = gfx::measure(&tab, 18.0) + 28.0;
    rrect(616.0 - tw, 20.0, tw, 32.0, 16.0, TOMATO);
    gfx::text(&tab, 616.0 - tw / 2.0, 25.0, 18.0, CREAM, Align::Center);

    for (i, c) in CARDS.iter().enumerate() {
        let (x, y) = album_slot(i);
        let sel = i == app.album_sel;
        let lift = if sel { -8.0 + (t * 3.0).sin() * 2.0 } else { 0.0 };
        let have = app.save.has(c.id);
        if sel {
            gfx::rrect_lines(x - 6.0, y + lift - 6.0, 116.0, 158.0, 14.0, 3.0, MUSTARD);
        }
        if have {
            art::card_face(x, y + lift, 104.0, 146.0, c.art, rgb(c.band), app.tr(c.name), t);
        } else {
            art::card_back(x, y + lift, 104.0, 146.0, true);
        }
    }

    if let Some(open) = app.album_open {
        let c = &CARDS[app.album_sel];
        let have = app.save.has(c.id);
        let k = ease_back(open / 0.3);
        gfx::rect(0.0, 0.0, gfx::W, gfx::H, fade(INK, 0.65 * (open / 0.2).min(1.0)));
        let (from_x, from_y) = album_slot(app.album_sel);
        let (cx, cy) = (from_x + 52.0 + (190.0 - from_x - 52.0) * k, from_y + 73.0 + (236.0 - from_y - 73.0) * k);
        let s = 1.0 + 0.9 * k;
        let wob = (open * 2.0).sin() * 1.5;
        gfx::transformed(cx, cy, wob, s, s, || {
            if have {
                art::card_face(-52.0, -73.0, 104.0, 146.0, c.art, rgb(c.band), app.tr(c.name), t);
            } else {
                art::card_back(-52.0, -73.0, 104.0, 146.0, true);
            }
        });
        let a = ((open - 0.15) / 0.2).clamp(0.0, 1.0);
        let (px, pw) = (330.0, 270.0);
        rrect(px, 120.0, pw, 220.0, 16.0, fade(PAPER, a));
        let name = if have || !c.hidden { app.tr(c.name) } else { app.tr(tx::HIDDEN_NAME) };
        let cond = if have || !c.hidden { app.tr(c.condition) } else { app.tr(tx::HIDDEN_COND) };
        gfx::text(name, px + 20.0, 138.0, 28.0, fade(INK, a), Align::Left);
        gfx::rect(px + 20.0, 178.0, 40.0, 4.0, fade(rgb(c.band), a));
        gfx::text_wrapped(cond, px + 20.0, 196.0, pw - 40.0, 20.0, 28.0, fade(INK, 0.85 * a), Align::Left);
        let (st, sc) = if have { (tx::OWNED, rgb(0x2E8B57)) } else { (tx::NOT_YET, fade(INK, 0.5)) };
        gfx::text(app.tr(st), px + 20.0, 300.0, 18.0, fade(sc, a), Align::Left);
        let [a, ..] = app.labels;
        prompts(&[(a, app.tr(tx::CLOSE))], true);
    } else {
        let [a, b, _] = app.labels;
        prompts(&[(a, app.tr(tx::VIEW)), (b, app.tr(tx::BACK))], false);
    }
}
