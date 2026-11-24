//! Drawing each screen on the 640×480 canvas.

use crate::app::{App, Screen, Summary};
use crate::art::{self, pal::*, Mood, Pose};
use crate::game::{floor::*, Facing, Outcome, Round};
use crate::gfx::{self, ease_back, ease_out, fade, mix, rgb, rrect, Align};
use crate::rules::*;
use crate::text as tx;
use raylib_sys::Color;
use std::f32::consts::PI;

pub fn crumb_color(t: Topping) -> Color {
    match t {
        Topping::Salmon => rgb(0xF4845F),
        Topping::Tuna => rgb(0xC8303F),
        Topping::Shrimp => rgb(0xFF9A5C),
        Topping::Tamago => rgb(0xF7CF4E),
        Topping::Maki => rgb(0x6BBF59),
        Topping::Inari => rgb(0xD4924F),
        Topping::Ikura => rgb(0xFF7B25),
    }
}

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

/// The app icon: 小馋 mid-bite, in the 480×480 square in the middle of the canvas.
fn icon(t: f32) {
    dotted_backdrop(rgb(0xEA6A3F), rgb(0xC4402A), 0.0);
    gfx::glow(320.0, 230.0, 230.0, fade(rgb(0xFFD08A), 0.55));
    let pose = Pose {
        facing: Facing::Down,
        walk: 0.0,
        moving: false,
        belly: 0.9,
        chew: None,
        reach: 0.9,
        mood: Mood::Happy,
        holding: Some(0),
        t,
    };
    gfx::transformed(320.0, 398.0, 0.0, 5.4, 5.4, || art::visitor(0.0, 0.0, &pose));
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
    let chew = (t * 0.8).fract();
    let pose = Pose {
        facing: Facing::Down,
        walk: 0.0,
        moving: false,
        belly: 0.6 + 0.2 * (t * 0.5).sin(),
        chew: if chew < 0.6 { Some(chew / 0.6) } else { None },
        reach: if chew >= 0.75 { (chew - 0.75) / 0.25 } else { 0.0 },
        mood: Mood::Happy,
        holding: if chew >= 0.75 { Some(((t * 0.8) as usize) % 6) } else { None },
        t,
    };
    gfx::transformed(cx, cy + 20.0, 0.0, 2.2, 2.2, || art::visitor(0.0, 0.0, &pose));
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
    Chef,
    Plate(usize),
    Visitor,
    Plant(f32, f32),
}

fn dining(app: &App) {
    let Some(r) = &app.round else { return };
    let t = app.t;
    art::floor_boards();
    art::back_wall(t);

    let mut things: Vec<(f32, Thing)> = vec![
        (CY, Thing::Island),
        (CY + 14.5, Thing::Chef),
        (r.player.pos.y, Thing::Visitor),
        (150.0, Thing::Plant(30.0, 150.0)),
        (150.0, Thing::Plant(610.0, 150.0)),
        (478.0, Thing::Plant(20.0, 478.0)),
        (478.0, Thing::Plant(620.0, 478.0)),
    ];
    for (i, p) in r.plates.iter().enumerate() {
        let y = p.pos().y;
        // Plates ride on the counter: never under it, and behind the chef on the far side.
        things.push((if y < CY + 14.0 { CY + 0.5 + y * 1e-3 } else { y }, Thing::Plate(i)));
    }
    things.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (_, th) in &things {
        match th {
            Thing::Island => art::island(r.belt_travel),
            Thing::Chef => art::chef(CX, CY + 14.0, 1.0 - r.chef_served / 0.5, t),
            Thing::Plate(i) => {
                let p = &r.plates[*i];
                let pos = p.pos();
                let appear = ease_back((p.age / 0.3).min(1.0));
                let leave = 1.0 - p.leaving;
                let s = appear * leave.max(0.0);
                if DISHES[p.dish].rim == Rim::Gold {
                    gfx::glow(pos.x, pos.y, 26.0, fade(rgb(0xFFE08A), 0.6 + 0.3 * (t * 5.0).sin()));
                }
                if s > 0.02 {
                    art::dish(pos.x, pos.y - (1.0 - appear) * 10.0, p.dish, s);
                }
            }
            Thing::Visitor => visitor(app, r),
            Thing::Plant(x, y) => art::plant(*x, *y, t),
        }
    }

    // The plate within reach.
    if let Some(i) = r.reachable() {
        let p = r.plates[i].pos();
        let pulse = 0.5 + 0.5 * (t * 6.0).sin();
        gfx::ellipse(p.x, p.y, 20.0 + pulse * 2.0, 11.0 + pulse, fade(CREAM, 0.0));
        for k in 0..2 {
            let a = 0.9 - k as f32 * 0.4;
            gfx::ring(p.x, p.y, 19.0 + pulse * 2.0 + k as f32 * 2.0, 20.5 + pulse * 2.0 + k as f32 * 2.0, 0.0, 360.0, fade(MUSTARD, a));
        }
        // A bubble naming the dish and its worth, kept on screen.
        let d = &DISHES[r.plates[i].dish];
        let by = p.y - 42.0 - (t * 5.0).sin().abs() * 3.0;
        let [a, ..] = app.labels;
        let name = app.tr(d.name);
        let worth = format!("+{}", d.coins);
        let w = gfx::measure(a, 14.0).max(10.0) + 11.0 + gfx::measure(name, 16.0) + gfx::measure(&worth, 16.0) + 24.0;
        let bx = (p.x - w / 2.0).clamp(8.0, gfx::W - 8.0 - w);
        rrect(bx, by, w, 26.0, 13.0, fade(INK, 0.92));
        gfx::tri((p.x - 5.0, by + 25.0), (p.x + 5.0, by + 25.0), (p.x, by + 31.0), fade(INK, 0.92));
        let aw = art::button_badge(a, bx + 4.0, by + 4.0, 18.0, MUSTARD, INK);
        let nw = gfx::text(name, bx + aw + 9.0, by + 3.0, 16.0, CREAM, Align::Left);
        gfx::text(&worth, bx + aw + 14.0 + nw, by + 3.0, 16.0, MUSTARD, Align::Left);
    }

    app.fx.draw_world();

    // Danger: a slow red pulse at the edges and the countdown overhead.
    if let Some(d) = r.danger {
        let pulse = 0.5 + 0.5 * (t * PI * 2.0).sin();
        let a = 0.25 + 0.2 * pulse;
        let red = rgb(0xD7263D);
        gfx::grad_v(0.0, 0.0, gfx::W, 70.0, fade(red, a), fade(red, 0.0));
        gfx::grad_v(0.0, 410.0, gfx::W, 70.0, fade(red, 0.0), fade(red, a));
        gfx::grad_h(0.0, 0.0, 70.0, gfx::H, fade(red, a), fade(red, 0.0));
        gfx::grad_h(570.0, 0.0, 70.0, gfx::H, fade(red, 0.0), fade(red, a));
        let left = (RULES.hospital_s - d).max(0.0);
        let p = r.player.pos;
        let (hx, hy) = (p.x, p.y - 92.0);
        gfx::circle(hx, hy, 20.0, fade(INK, 0.85));
        gfx::ring(hx, hy, 17.0, 20.0, -90.0, -90.0 + 360.0 * left / RULES.hospital_s, red);
        let n = format!("{}", left.ceil() as i32);
        let pop = ease_back(1.0 - left.fract());
        gfx::text_outlined(&n, hx, hy - 13.0 * pop, 26.0 * pop, CREAM, INK, 2.0, Align::Center);
    }

    // Outros: confetti, dimmed lights or the ambulance.
    if let (Some(o), t) = (r.outcome, r.outro) {
        match o {
            Outcome::Closed => gfx::rect(0.0, 0.0, gfx::W, gfx::H, fade(INK, (t / 1.5).min(1.0) * 0.45)),
            Outcome::Hospital => {
                let p = r.player.pos;
                let x = 720.0 - ease_out(t / 1.6) * (720.0 - p.x - 40.0);
                gfx::transformed(x, p.y - 12.0, 0.0, 1.5, 1.5, || ambulance(t));
            }
            Outcome::Done => {
                for i in 0..24 {
                    let k = (t * 0.6 + i as f32 * 0.37).fract();
                    let x = (i as f32 * 53.0 + (t + i as f32).sin() * 20.0).rem_euclid(gfx::W);
                    let c = [MUSTARD, TOMATO, MINT, SKY, CREAM][i % 5];
                    gfx::transformed(x, k * 500.0 - 20.0, t * 200.0 + i as f32 * 40.0, 1.0, 1.0, || gfx::rect(-3.0, -2.0, 6.0, 4.0, c));
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
    let belly = app.shown_fullness / RULES.capacity;
    let (chew, reach, holding) = match &pl.eating {
        Some(e) if e.t < RULES.bite_s => (None, e.t / RULES.bite_s, None),
        Some(e) => (Some(((e.t - RULES.bite_s) / (RULES.eat_s - RULES.bite_s)).min(1.0)), 0.0, None),
        None => (None, 0.0, None::<usize>),
    };
    let golden = pl.eating.as_ref().is_some_and(|e| e.dish == GOLDEN);
    let mood = if r.danger.is_some() || r.outcome == Some(Outcome::Hospital) {
        Mood::Danger
    } else if golden || r.outcome == Some(Outcome::Done) {
        Mood::Starry
    } else if r.fullness > RULES.capacity {
        Mood::Stuffed
    } else {
        Mood::Happy
    };
    let pose = Pose { facing: pl.facing, walk: pl.walk, moving: pl.moving, belly, chew, reach, mood, holding, t: app.t };
    // A wobble when stuffed or in danger.
    let wobble = if mood == Mood::Danger { (app.t * 9.0).sin() * 3.0 } else { 0.0 };
    let hop = if r.outcome == Some(Outcome::Done) { (r.outro * 8.0).sin().abs() * 8.0 } else { 0.0 };
    gfx::transformed(pl.pos.x, pl.pos.y - hop, wobble, 1.0, 1.0, || art::visitor(0.0, 0.0, &pose));
    // The food on its way from the plate to the mouth.
    if let Some(e) = &pl.eating {
        if e.t < RULES.bite_s {
            let k = ease_out(e.t / RULES.bite_s);
            let mouth = pl.pos + crate::game::V2::new(0.0, -36.0);
            let p = e.from.lerp(mouth, k);
            let arc = (k * PI).sin() * 22.0;
            art::food(p.x, p.y - arc, DISHES[e.dish].topping, 1.0 - 0.3 * k);
        }
    }
}

fn hud(app: &App, r: &Round) {
    let t = app.t;
    let chip = fade(INK, 0.86);
    // Time to closing, bottom left.
    let left = r.time_left();
    let urgent = left < 15.0 && r.outcome.is_none();
    let pulse = if urgent { 1.0 + 0.06 * (t * 6.0).sin() } else { 1.0 };
    rrect(14.0, 436.0, 100.0, 38.0, 19.0, chip);
    art::clock(35.0, 455.0, 12.0, left / RULES.duration, if urgent { TOMATO } else { MINT });
    let secs = left.ceil() as i32;
    let tc = if urgent { rgb(0xFF8A6B) } else { CREAM };
    gfx::transformed(80.0, 455.0, 0.0, pulse, pulse, || {
        gfx::text(&format!("{}:{:02}", secs / 60, secs % 60), 0.0, -14.0, 24.0, tc, Align::Center);
    });

    // The task, top middle: coins eaten against admission.
    let (x0, w) = (196.0, 248.0);
    rrect(x0, 8.0, w, 40.0, 20.0, chip);
    let bump = 1.0 + 0.3 * app.fx.bump_coins;
    art::coin(x0 + 22.0, 28.0, 11.0 * bump);
    let (bx, bw) = (x0 + 42.0, w - 56.0);
    rrect(bx, 17.0, bw, 22.0, 11.0, rgb(0x3A3D55));
    let frac = (app.shown_coins / RULES.admission as f32).clamp(0.0, 1.0);
    if frac > 0.0 {
        rrect(bx, 17.0, (bw * frac).max(22.0), 22.0, 11.0, MUSTARD);
        rrect(bx + 4.0, 19.0, (bw * frac - 8.0).max(14.0), 6.0, 3.0, fade(WHITE, 0.35));
    }
    let label = format!("{} / {}", app.shown_coins.round() as i32, RULES.admission);
    gfx::text_outlined(&label, bx + bw / 2.0, 16.0, 20.0, CREAM, INK, 1.6, Align::Center);

    // Plates eaten and the combo, bottom right.
    rrect(526.0, 436.0, 100.0, 38.0, 19.0, chip);
    let pb = 1.0 + 0.25 * app.fx.bump_plates;
    gfx::transformed(551.0, 460.0, 0.0, pb, pb, || art::plate_stack(0.0, 0.0, r.plates_eaten));
    gfx::text(&format!("×{}", r.plates_eaten), 570.0, 441.0, 24.0, CREAM, Align::Left);
    if r.combo >= 2 && r.combo_left() > 0.0 {
        rrect(526.0, 402.0, 100.0, 28.0, 14.0, fade(rgb(0xFF8FA3), 0.95));
        rrect(536.0, 424.0, 80.0 * r.combo_left() / RULES.combo_window, 3.0, 1.5, fade(INK, 0.5));
        gfx::text(&format!("{} ×{}", app.tr(tx::COMBO), r.combo), 576.0, 405.0, 18.0, INK, Align::Center);
    }

    // The stomach.
    let danger = r.danger.is_some();
    let (gx, gw) = (120.0, 400.0);
    rrect(gx, 436.0, gw, 38.0, 19.0, chip);
    if danger {
        let a = 0.5 + 0.5 * (t * PI * 2.0).sin();
        gfx::rrect_lines(gx - 2.0, 434.0, gw + 4.0, 42.0, 21.0, 3.0, fade(RED, a));
    }
    let belly = app.shown_fullness / RULES.capacity;
    art::tummy(gx + 22.0, 455.0, 12.0, belly);
    let (bx, bw, by, bh) = (gx + 42.0, 294.0 - 2.0, 446.0, 18.0);
    let per = bw / 150.0;
    rrect(bx, by, bw, bh, 9.0, rgb(0x3A3D55));
    gfx::rect(bx + 100.0 * per, by, 20.0 * per, bh, fade(rgb(0xF2B134), 0.25));
    gfx::transformed(0.0, 0.0, 0.0, 1.0, 1.0, || {
        let x1 = bx + 120.0 * per;
        rrect(x1, by, bw - 120.0 * per, bh, 9.0, fade(RED, 0.35));
        let mut sx = x1 + 2.0;
        while sx < bx + bw - 6.0 {
            gfx::line(sx, by + bh - 1.0, sx + 6.0, by + 1.0, 2.0, fade(RED, 0.5));
            sx += 9.0;
        }
    });
    let f = app.shown_fullness.clamp(0.0, 150.0);
    let fill = if f >= RULES.red_line {
        RED
    } else if f >= RULES.capacity {
        rgb(0xF28C28)
    } else if f >= 70.0 {
        MUSTARD
    } else {
        MINT
    };
    if f > 0.5 {
        rrect(bx, by, (f * per).max(bh), bh, 9.0, fill);
        rrect(bx + 4.0, by + 2.0, (f * per - 8.0).max(10.0), 5.0, 2.5, fade(WHITE, 0.35));
    }
    // What the plate in reach would do.
    if let Some(i) = r.reachable() {
        let add = DISHES[r.plates[i].dish].fullness as f32;
        let to = (r.fullness + add).min(150.0);
        let over = r.fullness + add > RULES.red_line;
        let a = 0.45 + 0.25 * (t * 6.0).sin();
        let c = if over { RED } else { CREAM };
        gfx::rect(bx + r.fullness.min(150.0) * per, by + 2.0, (to - r.fullness.min(150.0)) * per, bh - 4.0, fade(c, a));
    }
    gfx::rect(bx + 100.0 * per - 1.0, by - 4.0, 2.0, bh + 8.0, CREAM);
    gfx::rect(bx + 120.0 * per - 1.0, by - 4.0, 2.0, bh + 8.0, RED);
    let pc = if danger { rgb(0xFF8A8A) } else { CREAM };
    gfx::text(&format!("{}%", f.round() as i32), gx + gw - 14.0, 443.0, 22.0, pc, Align::Right);
}

fn pause(app: &App, r: &Round, sel: usize) {
    gfx::rect(0.0, 0.0, gfx::W, gfx::H, fade(INK, 0.6));
    let (x, y, w, h) = (170.0, 110.0, 300.0, 260.0);
    rrect(x + 4.0, y + 8.0, w, h, 18.0, fade(INK, 0.5));
    rrect(x, y, w, h, 18.0, PAPER);
    gfx::text(app.tr(tx::PAUSED), 320.0, y + 18.0, 30.0, INK, Align::Center);
    let secs = r.time_left().ceil() as i32;
    let info = format!("{} {}:{:02}   ·   {} / {}", app.tr(tx::TIME_LEFT), secs / 60, secs % 60, r.coins, RULES.admission);
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
    let mood = match s.outcome {
        Outcome::Hospital => Mood::Danger,
        Outcome::Done => Mood::Starry,
        Outcome::Closed => Mood::Stuffed,
    };
    let pose = Pose { facing: Facing::Down, walk: 0.0, moving: false, belly: s.peak / 100.0, chew: None, reach: 0.0, mood, holding: None, t: app.t };
    if s.new_cards.is_empty() {
        gfx::transformed(176.0, 330.0, 0.0, 2.6, 2.6, || art::visitor(0.0, 0.0, &pose));
        gfx::text(app.tr(tx::NO_NEW_CARDS), 176.0, 360.0, 18.0, CREAM, Align::Center);
    } else if s.new_cards.len() <= 3 {
        let hop = if s.outcome == Outcome::Done { (app.t * 4.0).sin().abs() * 6.0 } else { 0.0 };
        gfx::transformed(176.0, 412.0 - hop, 0.0, 2.1, 2.1, || art::visitor(0.0, 0.0, &pose));
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
            art::plate(cx, cy, DISHES[i].rim, 1.15, a);
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
