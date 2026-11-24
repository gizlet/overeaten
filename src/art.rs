//! The painted world and what is drawn around it: the room, the carousel
//! island, the dishes, the visitor, the furniture; plus the cards and HUD
//! icons, drawn from shapes.
//!
//! The room is painted for a 1600×960 stage seen from a high three-quarter
//! view (Tidewheel Sushi's). The game shows its middle 1280 columns at half
//! size on the 640×480 canvas, with the side walls moved in to frame it.

use crate::assets::{self, Assets};
use crate::fx;
use crate::game::{belt_point, floor::*, perimeter, Facing, V2};
use crate::gfx::{self, ellipse, fade, mix, rgb, rrect, Align, WHITE};
use crate::rules::{CardArt, GOLDEN};
use raylib_sys::{Color, Texture2D};
use std::f32::consts::PI;

pub mod pal {
    use crate::gfx::rgb;
    use raylib_sys::Color;
    pub const INK: Color = rgb(0x2B2D42);
    pub const HOUSE: Color = rgb(0x294C4B);
    pub const CREAM: Color = rgb(0xFFF4E0);
    pub const PAPER: Color = rgb(0xFFF8EC);
    pub const TOMATO: Color = rgb(0xE4572E);
    pub const MUSTARD: Color = rgb(0xF2B134);
    pub const MINT: Color = rgb(0x7FC8A9);
    pub const CELADON: Color = rgb(0x9BD3B4);
    pub const SKY: Color = rgb(0x4A6FA5);
    pub const RED: Color = rgb(0xD7263D);
    pub const WALNUT: Color = rgb(0x8B5A3C);
    pub const WALNUT_DARK: Color = rgb(0x5B3A29);
    pub const BLUSH: Color = rgb(0xFF8FA3);
    pub const WHITE: Color = rgb(0xFFFFFF);
    pub const COIN: Color = rgb(0xF2B134);
    pub const COIN_EDGE: Color = rgb(0xC98A1B);
}
use pal::*;

fn a() -> &'static Assets {
    assets::get()
}

// ---- The stage ---------------------------------------------------------

/// Stage pixels per floor centimeter across and down, and per centimeter of height.
const SX: f32 = 0.96;
const SY: f32 = 0.72;
const SH: f32 = 0.63;
/// The floor's north-west corner on the stage: the painted room's tile grid.
const ORIGIN: (f32, f32) = (80.0, 144.0);

/// Stage position of a floor point `h` centimeters up.
pub fn stage(p: V2, h: f32) -> (f32, f32) {
    (ORIGIN.0 + p.x * SX, ORIGIN.1 + p.y * SY - h * SH)
}

/// Canvas position of a floor point `h` centimeters up.
pub fn canvas(p: V2, h: f32) -> (f32, f32) {
    let (x, y) = stage(p, h);
    (x * 0.5 - 80.0, y * 0.5)
}

/// Draws in stage coordinates.
pub fn on_stage(f: impl FnOnce()) {
    gfx::transformed(-80.0, 0.0, 0.0, 0.5, 0.5, f);
}

/// The painted room, its side walls moved in, and slow warm light across the floor (stage).
pub fn room(t: f32) {
    let r = a().art.room;
    gfx::tex(r, (0.0, 0.0, 1600.0, 960.0), (0.0, 0.0, 1600.0, 960.0), (0.0, 0.0), WHITE);
    gfx::tex(r, (0.0, 0.0, 84.0, 960.0), (160.0, 0.0, 84.0, 960.0), (0.0, 0.0), WHITE);
    gfx::tex(r, (1516.0, 0.0, 84.0, 960.0), (1356.0, 0.0, 84.0, 960.0), (0.0, 0.0), WHITE);
    // Shade where the moved walls meet the floor.
    gfx::grad_h(244.0, 144.0, 26.0, 720.0, fade(INK, 0.18), fade(INK, 0.0));
    gfx::grad_h(1330.0, 144.0, 26.0, 720.0, fade(INK, 0.0), fade(INK, 0.18));
    let glow = a().art.glow;
    gfx::additive(|| {
        for k in 0..3 {
            let kf = k as f32;
            let breathe = 0.75 + 0.25 * (t * 0.35 + kf * 1.9).sin();
            let (x, y) = (520.0 + kf * 280.0, 420.0 + kf * 150.0);
            unsafe {
                raylib_sys::DrawTexturePro(
                    glow,
                    raylib_sys::Rectangle { x: 0.0, y: 0.0, width: 128.0, height: 128.0 },
                    raylib_sys::Rectangle { x, y, width: 1250.0, height: 150.0 },
                    gfx::v(625.0, 75.0),
                    24.0,
                    fade(rgb(0xFFC478), 26.0 / 255.0 * breathe),
                );
            }
        }
    });
}

/// The carousel island: the painting stretched in the middle so its rounded ends keep their
/// shape, its shadow, and slats travelling round the steel lane (stage).
pub fn island(belt_travel: f32) {
    let art = &a().art;
    let lane = (CX - HALF - R, CY - R, 2.0 * (HALF + R));
    let (lx, ty) = stage(V2::new(lane.0, lane.1), COUNTER_H);
    let (rx, _) = stage(V2::new(lane.0 + lane.2, lane.1), COUNTER_H);
    // The painting's lane runs from x 58.5 to 995.5 and starts at y 51.5; its ends are drawn at 0.8 across.
    let (left, right, top) = (lx - 58.5 * 0.8, rx + 60.5 * 0.8, ty - 51.5);
    let slice = |t: Texture2D, tint: Color, dy: f32| {
        let cap = 300.0;
        let capw = cap * 0.8;
        gfx::tex(t, (0.0, 0.0, cap, 280.0), (left, top + dy, capw + 1.0, 280.0), (0.0, 0.0), tint);
        gfx::tex(
            t,
            (cap, 0.0, 1056.0 - 2.0 * cap, 280.0),
            (left + capw, top + dy, right - left - 2.0 * capw + 1.0, 280.0),
            (0.0, 0.0),
            tint,
        );
        gfx::tex(t, (1056.0 - cap, 0.0, cap, 280.0), (right - capw, top + dy, capw, 280.0), (0.0, 0.0), tint);
    };
    slice(art.island_shadow, fade(WHITE, 200.0 / 255.0), 10.0);
    slice(art.island, WHITE, 0.0);
    let n = (perimeter() / 26.0) as usize;
    let shift = belt_travel.rem_euclid(perimeter() / n as f32);
    for k in 0..n {
        let (p, nrm) = belt_point(k as f32 * perimeter() / n as f32 + shift);
        let (ax, ay) = stage(p - nrm * BELT, COUNTER_H + 1.0);
        let (bx, by) = stage(p + nrm * BELT, COUNTER_H + 1.0);
        gfx::line(ax, ay + 1.5, bx, by + 1.5, 2.0, Color { r: 30, g: 40, b: 52, a: 40 });
        gfx::line(ax, ay, bx, by, 1.6, Color { r: 255, g: 255, b: 255, a: 46 });
    }
}

/// Furniture standing at `p`, `h` up: `foot` is how far down the art its base is (stage).
pub fn prop(t: Texture2D, p: V2, h: f32, s: f32, foot: f32) {
    let (x, y) = stage(p, h);
    let (w, hh) = (t.width as f32 * s, t.height as f32 * s);
    ellipse(x, y - 2.0, w * 0.36, w * 0.08, Color { r: 20, g: 14, b: 10, a: 50 });
    gfx::tex(t, (0.0, 0.0, t.width as f32, t.height as f32), (x - w / 2.0, y - hh * foot, w, hh), (0.0, 0.0), WHITE);
}

pub fn booth() {
    let (x, y, w, h) = BOOTH;
    prop(a().art.booth, V2::new(x + w / 2.0, y + h), 0.0, 0.42, 0.96);
}

pub fn yuzu() {
    let (x, y, w, h) = YUZU;
    prop(a().art.yuzu, V2::new(x + w / 2.0, y + h - 10.0), 0.0, 0.3, 0.97);
}

pub fn condiments() {
    prop(a().art.condiments, V2::new(CX + 150.0, CY + 14.0), COUNTER_H, 0.2, 0.9);
}

pub fn tea() {
    prop(a().art.tea, V2::new(CX - 150.0, CY + 14.0), COUNTER_H, 0.21, 0.88);
}

// ---- Dishes ------------------------------------------------------------

const GOLD: Color = Color { r: 255, g: 214, b: 120, a: 255 };

/// A dish from its painted layers: shadow, plate, the food's contact shadow and the food. The
/// art is 320×256 with the plate's middle at (160, 150); `s` scales it.
pub fn dish_stack(d: usize, x: f32, y: f32, s: f32, tint: Color, food: bool) {
    let art = &a().art;
    let dd = &art.dishes[d];
    let plate = if d == GOLDEN { GOLD } else { WHITE };
    let at = |t: Texture2D, dy: f32, c: Color| {
        gfx::tex(t, (0.0, 0.0, 320.0, 256.0), (x, y + dy, 320.0 * s, 256.0 * s), (160.0 * s, 150.0 * s), c);
    };
    at(art.plate_shadow, 22.0 * s, fade(WHITE, tint.a as f32 / 255.0 * 0.8));
    let pc = Color {
        r: (plate.r as u16 * tint.r as u16 / 255) as u8,
        g: (plate.g as u16 * tint.g as u16 / 255) as u8,
        b: (plate.b as u16 * tint.b as u16 / 255) as u8,
        a: tint.a,
    };
    at(dd.ceramic, 0.0, pc);
    if food {
        at(dd.contact, 0.0, tint);
        at(dd.food, 0.0, tint);
    }
}

/// A dish on the canvas at roughly the belt's size times `s`.
pub fn dish(x: f32, y: f32, d: usize, s: f32) {
    dish_stack(d, x, y, 0.1 * s, WHITE, true);
}

/// An empty plate on the canvas.
pub fn plate(x: f32, y: f32, d: usize, s: f32, alpha: f32) {
    dish_stack(d, x, y, 0.1 * s, fade(WHITE, alpha), false);
}

/// The bite in the hand, centered on `(x, y)`, `w` wide.
pub fn held(d: usize, x: f32, y: f32, w: f32) {
    let t = a().art.dishes[d].held;
    let s = w / t.width as f32;
    gfx::tex(
        t,
        (0.0, 0.0, t.width as f32, t.height as f32),
        (x, y, t.width as f32 * s, t.height as f32 * s),
        (t.width as f32 * s / 2.0, t.height as f32 * s / 2.0),
        WHITE,
    );
}

// ---- The visitor -------------------------------------------------------

/// What the visitor is doing, for choosing a frame.
#[derive(Clone, Copy)]
pub struct Pose {
    pub facing: Facing,
    pub walking: bool,
    /// Walk cycle phase in radians.
    pub walk: f32,
    /// 0..1 through a bite, while eating.
    pub eat: Option<f32>,
    /// The plate is to the left.
    pub from_left: bool,
    /// Fullness as a fraction of capacity, for the belly.
    pub belly: f32,
    pub red: bool,
    pub t: f32,
}

impl Pose {
    pub fn idle(facing: Facing, t: f32) -> Pose {
        Pose { facing, walking: false, walk: 0.0, eat: None, from_left: false, belly: 0.5, red: false, t }
    }
}

/// The frame, mirrored or not, a hop and a squash for `p`.
fn frame(p: &Pose) -> (Texture2D, bool, f32, f32, f32) {
    let art = &a().art.player;
    let (mut t, mut mirror) = match p.facing {
        Facing::Down => (art.front, false),
        Facing::Up => (art.back, false),
        Facing::Right => (art.right, false),
        Facing::Left => (art.right, true),
    };
    let (mut lift, mut sx, mut sy) = (0.0, 1.0, 1.0);
    if let Some(k) = p.eat {
        t = if k < 0.3 {
            art.reach
        } else if k < 0.65 {
            art.bite
        } else {
            art.savor
        };
        mirror = p.from_left;
        if k >= 0.65 {
            let e = ((k - 0.65) / 0.35 * PI).sin();
            lift = e * 8.0;
            sy = 1.0 + 0.05 * e;
            sx = 1.0 / sy;
        }
    } else if p.walking {
        let cycle = p.walk / PI;
        if cycle.floor() as i32 % 2 == 0 {
            t = match p.facing {
                Facing::Down => art.walk_front,
                Facing::Up => art.walk_back,
                _ => art.walk_right,
            };
        }
        lift = (cycle * PI).sin().abs() * 6.0;
        let k = (cycle * 2.0 * PI).cos() * 0.03;
        sy = 1.0 + k;
        sx = 1.0 - k;
    } else {
        sy = 1.0 + 0.015 * (p.t * 2.6).sin();
    }
    // A rounder figure as the stomach fills.
    sx *= 1.0 + 0.07 * (p.belly - 0.6).clamp(0.0, 0.9);
    (t, mirror, lift, sx, sy)
}

/// The visitor with the soles at `(x, y)`, the art 256×288 drawn at `scale`, with a soft shadow.
pub fn visitor(x: f32, y: f32, scale: f32, p: &Pose) {
    let (t, mirror, lift, sx, sy) = frame(p);
    let lift = lift * scale * 2.0;
    let k = 1.0 - lift / (60.0 * scale);
    ellipse(x, y, 68.0 * scale * k, 22.0 * scale * k, Color { r: 24, g: 16, b: 12, a: 70 });
    ellipse(x, y, 44.0 * scale * k, 14.0 * scale * k, Color { r: 24, g: 16, b: 12, a: 60 });
    let tint = if p.red { mix(WHITE, rgb(0xFFB4AA), 0.5 + 0.5 * (p.t * 7.0).sin()) } else { WHITE };
    let (w, h) = (256.0 * scale * sx, 288.0 * scale * sy);
    let src = if mirror { (0.0, 0.0, -256.0, 288.0) } else { (0.0, 0.0, 256.0, 288.0) };
    gfx::tex(t, src, (x, y - lift, w, h), (w * 0.5, h * 260.0 / 288.0), tint);
}

/// Where the visitor's mouth is, relative to the soles, at `scale`.
pub fn mouth(scale: f32) -> (f32, f32) {
    (0.0, -150.0 * scale)
}

// ---- Icons -------------------------------------------------------------

pub fn coin(x: f32, y: f32, r: f32) {
    gfx::circle(x, y + r * 0.15, r, COIN_EDGE);
    gfx::circle(x, y, r, COIN);
    gfx::ring(x, y, r * 0.62, r * 0.74, 0.0, 360.0, COIN_EDGE);
    gfx::circle(x - r * 0.3, y - r * 0.35, r * 0.2, fade(WHITE, 0.8));
}

pub fn lock(x: f32, y: f32, c: Color) {
    gfx::ring(x, y - 6.0, 5.0, 7.5, 180.0, 360.0, c);
    gfx::rect(x - 7.5, y - 6.0, 2.5, 4.0, c);
    gfx::rect(x + 5.0, y - 6.0, 2.5, 4.0, c);
    rrect(x - 10.0, y - 3.0, 20.0, 15.0, 3.0, c);
    gfx::circle(x, y + 3.0, 2.2, fade(INK, 0.6));
}

/// A rounded button badge with its label, such as "A". Returns its width.
pub fn button_badge(label: &str, x: f32, y: f32, size: f32, fill: Color, ink: Color) -> f32 {
    let w = gfx::measure(label, size * 0.72).max(size * 0.5) + size * 0.55;
    rrect(x, y, w, size, size / 2.0, fill);
    gfx::text(label, x + w / 2.0, y + size * 0.1, size * 0.72, ink, Align::Center);
    w
}

// ---- Cards -------------------------------------------------------------

/// A card's face, its top-left at `(x, y)`.
pub fn card_face(x: f32, y: f32, w: f32, h: f32, art: CardArt, band: Color, name: &str, t: f32) {
    let s = w / 108.0;
    rrect(x + 2.0 * s, y + 4.0 * s, w, h, 9.0 * s, fade(INK, 0.3));
    rrect(x, y, w, h, 9.0 * s, PAPER);
    gfx::rrect_lines(x, y, w, h, 9.0 * s, 3.0 * s, band);
    let (ix, iy, iw, ih) = (x + 7.0 * s, y + 7.0 * s, w - 14.0 * s, h * 0.6);
    rrect(ix, iy, iw, ih, 6.0 * s, mix(band, WHITE, 0.55));
    for row in 0..6 {
        for col in 0..6 {
            let dx = ix + 8.0 * s + col as f32 * iw / 6.0 + if row % 2 == 0 { 0.0 } else { iw / 12.0 };
            let dy = iy + 7.0 * s + row as f32 * ih / 6.0;
            if dx < ix + iw - 4.0 * s {
                gfx::circle(dx, dy, 1.4 * s, fade(WHITE, 0.55));
            }
        }
    }
    card_art(art, ix + iw / 2.0, iy + ih / 2.0 + 2.0 * s, s, t);
    gfx::text_wrapped(name, x + 6.0 * s, iy + ih + 8.0 * s, w - 12.0 * s, 15.0 * s, 18.0 * s, INK, Align::Center);
}

/// A card's back, with a question mark for undiscovered cards.
pub fn card_back(x: f32, y: f32, w: f32, h: f32, unknown: bool) {
    let s = w / 108.0;
    rrect(x + 2.0 * s, y + 4.0 * s, w, h, 9.0 * s, fade(INK, 0.3));
    rrect(x, y, w, h, 9.0 * s, TOMATO);
    gfx::rrect_lines(x + 5.0 * s, y + 5.0 * s, w - 10.0 * s, h - 10.0 * s, 6.0 * s, 2.0 * s, CREAM);
    for row in 0..9 {
        for col in 0..6 {
            let dx = x + 14.0 * s + col as f32 * 16.0 * s + if row % 2 == 0 { 0.0 } else { 8.0 * s };
            let dy = y + 14.0 * s + row as f32 * 15.0 * s;
            if dx < x + w - 10.0 * s && dy < y + h - 10.0 * s {
                gfx::circle(dx, dy, 1.6 * s, fade(CREAM, 0.35));
            }
        }
    }
    let (cx, cy) = (x + w / 2.0, y + h / 2.0);
    gfx::circle(cx, cy, 26.0 * s, CREAM);
    gfx::ring(cx, cy, 22.0 * s, 24.0 * s, 0.0, 360.0, TOMATO);
    let mark = if unknown { "?" } else { "吃" };
    gfx::text(mark, cx, cy - 16.0 * s, 30.0 * s, TOMATO, Align::Center);
}

fn card_art(art: CardArt, cx: f32, cy: f32, s: f32, t: f32) {
    match art {
        CardArt::FirstPlate => {
            dish(cx, cy + 6.0 * s, 0, 1.9 * s);
            gfx::line(cx + 10.0 * s, cy - 26.0 * s, cx + 30.0 * s, cy + 8.0 * s, 3.0 * s, WALNUT);
            gfx::line(cx + 16.0 * s, cy - 28.0 * s, cx + 34.0 * s, cy + 4.0 * s, 3.0 * s, WALNUT);
        }
        CardArt::MoneysWorth => {
            for i in 0..4 {
                let yy = cy + 16.0 * s - i as f32 * 7.0 * s;
                ellipse(cx - 14.0 * s, yy + 2.0 * s, 15.0 * s, 6.0 * s, COIN_EDGE);
                ellipse(cx - 14.0 * s, yy, 15.0 * s, 6.0 * s, COIN);
            }
            ellipse(cx - 14.0 * s, cy - 5.0 * s, 9.0 * s, 3.0 * s, COIN_EDGE);
            rrect(cx + 4.0 * s, cy - 24.0 * s, 28.0 * s, 40.0 * s, 3.0 * s, WHITE);
            for i in 0..4 {
                gfx::line(
                    cx + 9.0 * s,
                    cy - 17.0 * s + i as f32 * 7.0 * s,
                    cx + 27.0 * s,
                    cy - 17.0 * s + i as f32 * 7.0 * s,
                    1.5 * s,
                    rgb(0xC9C2B2),
                );
            }
            gfx::line(cx + 10.0 * s, cy + 6.0 * s, cx + 16.0 * s, cy + 12.0 * s, 3.0 * s, RED);
            gfx::line(cx + 16.0 * s, cy + 12.0 * s, cx + 30.0 * s, cy - 6.0 * s, 3.0 * s, RED);
        }
        CardArt::Rainbow => {
            gfx::ring(cx, cy, 24.0 * s, 32.0 * s, 0.0, 360.0, fade(rgb(0x56606E), 0.85));
            for i in 0..6 {
                let a = t * 0.6 + i as f32 * PI / 3.0;
                let (px, py) = (cx + a.cos() * 28.0 * s, cy + a.sin() * 28.0 * s);
                dish(px, py + 2.0 * s, i, 0.75 * s);
            }
            fx::star(cx, cy, 9.0 * s, MUSTARD);
        }
        CardArt::Golden => {
            gfx::glow(cx, cy, 40.0 * s, fade(rgb(0xFFE08A), 0.8));
            dish(cx, cy + 6.0 * s, GOLDEN, 2.0 * s);
            for (i, (dx, dy)) in [(-30.0, -18.0), (28.0, -22.0), (32.0, 14.0), (-28.0, 18.0)].iter().enumerate() {
                let k = 0.7 + 0.3 * (t * 4.0 + i as f32).sin();
                fx::star(cx + dx * s, cy + dy * s, 6.0 * s * k, WHITE);
            }
        }
        CardArt::LightEater => {
            let hop = (t * 3.0).sin().abs() * 3.0 * s;
            ellipse(cx, cy + 22.0 * s, 16.0 * s, 4.0 * s, fade(INK, 0.15));
            gfx::circle(cx, cy - hop, 17.0 * s, rgb(0xF6D55C));
            ellipse(cx - 4.0 * s, cy + 3.0 * s - hop, 9.0 * s, 6.0 * s, rgb(0xE9B949));
            gfx::circle(cx + 7.0 * s, cy - 6.0 * s - hop, 2.4 * s, INK);
            gfx::tri(
                (cx + 15.0 * s, cy - 4.0 * s - hop),
                (cx + 24.0 * s, cy - 1.0 * s - hop),
                (cx + 15.0 * s, cy + 2.0 * s - hop),
                rgb(0xF28C28),
            );
            ellipse(cx + 8.0 * s, cy + 2.0 * s - hop, 3.0 * s, 1.8 * s, fade(BLUSH, 0.8));
            dish(cx - 26.0 * s, cy + 18.0 * s, 4, 0.7 * s);
        }
        CardArt::Combo => {
            for (i, d) in [4, 3, 0, 2, GOLDEN].iter().enumerate() {
                let yy = cy + 20.0 * s - i as f32 * 7.5 * s;
                let wob = (t * 3.0 + i as f32 * 0.7).sin() * i as f32 * 0.6 * s;
                plate(cx - 10.0 * s + wob, yy, *d, 1.3 * s, 1.0);
            }
            gfx::text_outlined("×5", cx + 22.0 * s, cy - 14.0 * s, 22.0 * s, MUSTARD, INK, 1.5 * s, Align::Center);
        }
        CardArt::Ambulance => {
            let flash = (t * 2.5).sin() > 0.0;
            gfx::glow(cx - 6.0 * s, cy - 22.0 * s, 16.0 * s, fade(if flash { rgb(0x5AA9E6) } else { RED }, 0.8));
            rrect(cx - 30.0 * s, cy - 16.0 * s, 46.0 * s, 30.0 * s, 4.0 * s, WHITE);
            rrect(cx + 12.0 * s, cy - 6.0 * s, 20.0 * s, 20.0 * s, 4.0 * s, WHITE);
            rrect(cx + 17.0 * s, cy - 3.0 * s, 11.0 * s, 8.0 * s, 2.0 * s, rgb(0x9CC9E8));
            gfx::rect(cx - 30.0 * s, cy + 2.0 * s, 62.0 * s, 4.0 * s, RED);
            gfx::rect(cx - 12.0 * s, cy - 12.0 * s, 6.0 * s, 16.0 * s, RED);
            gfx::rect(cx - 17.0 * s, cy - 7.0 * s, 16.0 * s, 6.0 * s, RED);
            rrect(cx - 10.0 * s, cy - 22.0 * s, 8.0 * s, 6.0 * s, 2.0 * s, if flash { rgb(0x5AA9E6) } else { RED });
            for wx in [-18.0, 20.0] {
                gfx::circle(cx + wx * s, cy + 15.0 * s, 6.0 * s, INK);
                gfx::circle(cx + wx * s, cy + 15.0 * s, 2.5 * s, rgb(0xC9C2B2));
            }
        }
    }
}

// ---- Restaurant tickets ------------------------------------------------

/// A small picture of each restaurant for its ticket.
pub fn storefront(i: usize, cx: f32, cy: f32, t: f32) {
    match i {
        0 => {
            rrect(cx - 60.0, cy - 22.0, 120.0, 44.0, 22.0, WALNUT);
            rrect(cx - 54.0, cy - 16.0, 108.0, 32.0, 16.0, rgb(0x56606E));
            rrect(cx - 44.0, cy - 6.0, 88.0, 12.0, 6.0, rgb(0xEAD7B7));
            let per = 2.0 * 76.0 + PI * 21.0;
            for k in 0..6 {
                let s = (t * 22.0 + k as f32 * per / 6.0).rem_euclid(per);
                let (px, py) = if s < 76.0 {
                    (cx - 38.0 + s, cy - 11.0)
                } else if s < 76.0 + PI * 10.5 {
                    let a = -PI / 2.0 + (s - 76.0) / 10.5;
                    (cx + 38.0 + a.cos() * 10.5, cy + a.sin() * 11.0)
                } else if s < 152.0 + PI * 10.5 {
                    (cx + 38.0 - (s - 76.0 - PI * 10.5), cy + 11.0)
                } else {
                    let a = PI / 2.0 + (s - 152.0 - PI * 10.5) / 10.5;
                    (cx - 38.0 + a.cos() * 10.5, cy + a.sin() * 11.0)
                };
                dish(px, py, k, 0.55);
            }
        }
        1 => {
            for k in 0..3 {
                let p = (t * 0.6 + k as f32 / 3.0).fract();
                gfx::circle(
                    cx - 14.0 + k as f32 * 14.0,
                    cy - 22.0 - p * 22.0,
                    6.0 + p * 6.0,
                    fade(WHITE, 0.5 * (1.0 - p)),
                );
            }
            ellipse(cx, cy + 4.0, 44.0, 20.0, rgb(0x6B6F7A));
            ellipse(cx, cy - 2.0, 44.0, 16.0, rgb(0x8D93A0));
            ellipse(cx, cy - 2.0, 38.0, 12.0, rgb(0xC0392B));
            gfx::line(cx, cy - 14.0, cx, cy + 10.0, 3.0, rgb(0x8D93A0));
            ellipse(cx + 19.0, cy - 2.0, 17.0, 10.0, rgb(0xF2E2C4));
            for k in 0..5 {
                let a = t * 1.5 + k as f32 * 1.3;
                gfx::circle(cx - 20.0 + a.cos() * 10.0, cy - 2.0 + a.sin() * 4.0, 2.5, rgb(0xFF7B54));
            }
        }
        _ => {
            rrect(cx - 46.0, cy - 6.0, 92.0, 20.0, 4.0, rgb(0x2F8F83));
            for wx in [-34.0, 34.0] {
                gfx::circle(cx + wx, cy + 18.0, 7.0, INK);
            }
            for k in 0..3 {
                let bx = cx - 26.0 + k as f32 * 26.0;
                for j in 0..2 {
                    let by = cy - 12.0 - j as f32 * 10.0;
                    ellipse(bx, by + 3.0, 12.0, 5.0, rgb(0xB8894F));
                    gfx::rect(bx - 12.0, by - 3.0, 24.0, 6.0, rgb(0xD4A86A));
                    ellipse(bx, by - 3.0, 12.0, 5.0, rgb(0xE6C08A));
                }
            }
            let p = (t * 0.7).fract();
            gfx::circle(cx, cy - 40.0 - p * 12.0, 6.0 + p * 4.0, fade(WHITE, 0.5 * (1.0 - p)));
        }
    }
}

/// One of the painted HUD icons, centered on `(x, y)`, `size` across.
pub fn hud_icon(clock: bool, x: f32, y: f32, size: f32, tint: Color) {
    let t = if clock { a().art.clock } else { a().art.plates };
    gfx::tex(t, (0.0, 0.0, t.width as f32, t.height as f32), (x, y, size, size), (size / 2.0, size / 2.0), tint);
}

/// A HUD panel: deep teal with a thin cream rule inside and a soft drop shadow.
pub fn panel(x: f32, y: f32, w: f32, h: f32) {
    rrect(x + 2.0, y + 4.0, w, h, 12.0, fade(INK, 0.35));
    rrect(x, y, w, h, 12.0, fade(HOUSE, 0.95));
    rrect(x + 2.0, y + 2.0, w - 4.0, h * 0.45, 10.0, fade(WHITE, 0.06));
    gfx::rrect_lines(x + 3.0, y + 3.0, w - 6.0, h - 6.0, 9.0, 1.2, fade(rgb(0xF3E3BF), 0.55));
}
