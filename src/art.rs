//! Everything drawn from shapes: the dishes, the visitor, the chef, the
//! carousel restaurant, the cards and the HUD icons. No image files.

use crate::fx;
use crate::game::{belt_point, floor::*, perimeter, Facing};
use crate::gfx::{self, ellipse, fade, mix, rgb, rrect, Align};
use crate::rules::{CardArt, Rim, Topping, DISHES};
use raylib_sys::Color;
use std::f32::consts::PI;

pub mod pal {
    use crate::gfx::rgb;
    use raylib_sys::Color;
    pub const INK: Color = rgb(0x2B2D42);
    pub const CREAM: Color = rgb(0xFFF4E0);
    pub const PAPER: Color = rgb(0xFFF8EC);
    pub const TOMATO: Color = rgb(0xE4572E);
    pub const MUSTARD: Color = rgb(0xF2B134);
    pub const MINT: Color = rgb(0x7FC8A9);
    pub const SKY: Color = rgb(0x4A6FA5);
    pub const RED: Color = rgb(0xD7263D);
    pub const WALNUT: Color = rgb(0x8B5A3C);
    pub const WALNUT_DARK: Color = rgb(0x5B3A29);
    pub const MAPLE: Color = rgb(0xEAD7B7);
    pub const SKIN: Color = rgb(0xFFDDBE);
    pub const HAIR: Color = rgb(0x3B2A33);
    pub const HOODIE: Color = rgb(0xF28C28);
    pub const HOODIE_LIGHT: Color = rgb(0xF8AE5A);
    pub const HOODIE_DARK: Color = rgb(0xD9711A);
    pub const DENIM: Color = rgb(0x3D4F7C);
    pub const BLUSH: Color = rgb(0xFF8FA3);
    pub const WHITE: Color = rgb(0xFFFFFF);
    pub const NORI: Color = rgb(0x1F3B2D);
    pub const RICE: Color = rgb(0xFFFDF5);
    pub const COIN: Color = rgb(0xF2B134);
    pub const COIN_EDGE: Color = rgb(0xC98A1B);
}
use pal::*;

// ---- Dishes ------------------------------------------------------------

pub fn rim_color(r: Rim) -> Color {
    match r {
        Rim::White => rgb(0xD9D4C7),
        Rim::Green => rgb(0x5DB87A),
        Rim::Red => rgb(0xE4572E),
        Rim::Gold => rgb(0xF4C430),
    }
}

/// One serving's food, centered on `(x, y)`, `s` times its belt size.
pub fn food(x: f32, y: f32, t: Topping, s: f32) {
    let nigiri = |top: Color, seam: Color| {
        rrect(x - 8.0 * s, y - 1.5 * s, 16.0 * s, 7.0 * s, 3.5 * s, rgb(0xE6DCC8));
        rrect(x - 8.0 * s, y - 3.0 * s, 16.0 * s, 7.0 * s, 3.5 * s, RICE);
        rrect(x - 9.5 * s, y - 6.5 * s, 19.0 * s, 7.0 * s, 3.5 * s, top);
        for i in 0..3 {
            let sx = x - 5.0 * s + i as f32 * 4.5 * s;
            gfx::line(sx, y - 6.0 * s, sx - 2.5 * s, y - 0.5 * s, 1.1 * s, seam);
        }
        gfx::line(x - 6.5 * s, y - 5.6 * s, x + 4.0 * s, y - 5.6 * s, 0.9 * s, fade(WHITE, 0.45));
    };
    match t {
        Topping::Salmon => nigiri(rgb(0xF4845F), rgb(0xFFD0B0)),
        Topping::Tuna => nigiri(rgb(0xC8303F), rgb(0xE56B75)),
        Topping::Shrimp => {
            nigiri(rgb(0xFF9A5C), rgb(0xFFF1E6));
            gfx::tri((x + 8.0 * s, y - 4.0 * s), (x + 12.5 * s, y - 8.0 * s), (x + 11.5 * s, y - 2.0 * s), rgb(0xE63946));
            gfx::tri((x + 8.0 * s, y - 3.0 * s), (x + 13.0 * s, y - 1.0 * s), (x + 10.0 * s, y + 0.5 * s), rgb(0xE63946));
        }
        Topping::Tamago => {
            rrect(x - 8.0 * s, y - 1.5 * s, 16.0 * s, 7.0 * s, 3.5 * s, rgb(0xE6DCC8));
            rrect(x - 8.0 * s, y - 3.0 * s, 16.0 * s, 7.0 * s, 3.5 * s, RICE);
            rrect(x - 9.5 * s, y - 8.0 * s, 19.0 * s, 8.5 * s, 2.5 * s, rgb(0xE8A830));
            rrect(x - 9.5 * s, y - 8.5 * s, 19.0 * s, 6.5 * s, 2.5 * s, rgb(0xF7CF4E));
            gfx::line(x - 7.0 * s, y - 6.5 * s, x + 6.0 * s, y - 6.5 * s, 0.9 * s, fade(WHITE, 0.5));
            gfx::rect(x - 2.0 * s, y - 8.8 * s, 4.0 * s, 13.0 * s, NORI);
        }
        Topping::Maki => {
            for dx in [-5.2, 5.2] {
                let cx = x + dx * s;
                ellipse(cx, y + 0.5 * s, 5.2 * s, 4.4 * s, rgb(0x14271E));
                gfx::rect(cx - 5.2 * s, y - 2.5 * s, 10.4 * s, 3.0 * s, rgb(0x14271E));
                ellipse(cx, y - 2.5 * s, 5.2 * s, 4.2 * s, NORI);
                ellipse(cx, y - 2.5 * s, 4.0 * s, 3.2 * s, RICE);
                ellipse(cx, y - 2.5 * s, 1.8 * s, 1.5 * s, rgb(0x6BBF59));
            }
        }
        Topping::Inari => {
            rrect(x - 9.5 * s, y - 6.0 * s, 19.0 * s, 10.5 * s, 5.0 * s, rgb(0xB5763A));
            rrect(x - 9.0 * s, y - 7.0 * s, 18.0 * s, 9.0 * s, 4.5 * s, rgb(0xD4924F));
            ellipse(x, y - 5.5 * s, 7.5 * s, 2.6 * s, rgb(0xF0D9A8));
            for (dx, dy) in [(-3.0, -5.8), (1.5, -5.0), (4.0, -6.0)] {
                ellipse(x + dx * s, y + dy * s, 0.9 * s, 0.6 * s, rgb(0x5A3A1E));
            }
        }
        Topping::Ikura => {
            rrect(x - 9.0 * s, y - 5.0 * s, 18.0 * s, 10.0 * s, 4.5 * s, NORI);
            ellipse(x, y - 5.0 * s, 9.0 * s, 3.6 * s, rgb(0x2C4F3C));
            for (dx, dy) in [(-5.0, -6.0), (-1.5, -7.0), (2.5, -6.5), (5.5, -5.5), (-3.0, -4.0), (1.0, -4.5), (4.5, -3.5)] {
                gfx::circle(x + dx * s, y + dy * s, 2.1 * s, rgb(0xFF7B25));
                gfx::circle(x + (dx - 0.6) * s, y + (dy - 0.7) * s, 0.7 * s, rgb(0xFFE0B8));
            }
        }
    }
}

/// A plate with its rim color, centered on `(x, y)`.
pub fn plate(x: f32, y: f32, rim: Rim, s: f32, alpha: f32) {
    let c = rim_color(rim);
    ellipse(x + 1.0 * s, y + 4.0 * s, 17.0 * s, 8.0 * s, fade(INK, 0.22 * alpha));
    ellipse(x, y + 1.8 * s, 16.0 * s, 8.5 * s, fade(mix(c, INK, 0.25), alpha));
    ellipse(x, y, 16.0 * s, 8.5 * s, fade(c, alpha));
    ellipse(x, y + 0.4 * s, 12.0 * s, 6.0 * s, fade(rgb(0xFFFDF7), alpha));
    ellipse(x, y + 1.2 * s, 9.0 * s, 4.0 * s, fade(rgb(0xF1EBDD), alpha));
}

pub fn dish(x: f32, y: f32, dish: usize, s: f32) {
    let d = &DISHES[dish];
    plate(x, y, d.rim, s, 1.0);
    food(x, y - 1.0 * s, d.topping, s);
}

// ---- The visitor -------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Happy,
    Stuffed,
    Danger,
    Starry,
}

pub struct Pose {
    pub facing: Facing,
    pub walk: f32,
    pub moving: bool,
    /// Fullness as a fraction of capacity.
    pub belly: f32,
    /// 0..1 through the chewing, while chewing.
    pub chew: Option<f32>,
    /// 0..1: the hand going up to the mouth.
    pub reach: f32,
    pub mood: Mood,
    /// Food held in the hand, by dish.
    pub holding: Option<usize>,
    pub t: f32,
}

/// 小馋, the visitor: a small round eater in an orange hoodie with a bib,
/// a bun held by a pair of chopsticks. Feet at `(x, y)`.
pub fn visitor(x: f32, y: f32, p: &Pose) {
    let bw = p.belly.clamp(0.0, 1.5);
    let bob = if p.moving { -(p.walk.sin().abs()) * 2.2 } else { (p.t * 2.2).sin() * 0.6 };
    let side = match p.facing {
        Facing::Left => -1.0,
        Facing::Right => 1.0,
        _ => 0.0,
    };
    ellipse(x, y, 13.0 + bw * 3.0, 4.5, fade(INK, 0.25));

    // Legs and shoes.
    let swing = if p.moving { p.walk.sin() } else { 0.0 };
    if side == 0.0 {
        for (i, lx) in [-4.0f32, 4.0].iter().enumerate() {
            let lift = if i == 0 { swing.max(0.0) } else { (-swing).max(0.0) } * 2.5;
            gfx::rect(x + lx - 2.5, y - 8.0 - lift, 5.0, 7.0, DENIM);
            ellipse(x + lx, y - 1.5 - lift, 3.6, 2.2, INK);
        }
    } else {
        for k in [-1.0f32, 1.0] {
            let fx = x + k * swing * 3.5;
            gfx::rect(fx - 2.5, y - 8.0, 5.0, 7.0, if k < 0.0 { mix(DENIM, INK, 0.3) } else { DENIM });
            ellipse(fx + side * 1.2, y - 1.5, 4.0, 2.2, INK);
        }
    }

    let ty = y - 6.0 + bob;
    let tw = 20.0 + bw * 7.0;
    // Arms behind the body in side view.
    if side != 0.0 {
        let back = (x - side * 2.0 - swing * 2.5, ty - 6.0);
        gfx::line(x - side * 1.0, ty - 16.0, back.0, back.1, 5.5, HOODIE_DARK);
    }
    // Torso and belly.
    rrect(x - tw / 2.0, ty - 21.0, tw, 21.0, 8.0, HOODIE);
    match p.facing {
        Facing::Up => {
            ellipse(x, ty - 19.0, 9.0, 4.0, HOODIE_DARK);
            gfx::line(x - 6.0, ty - 4.0, x + 6.0, ty - 4.0, 1.5, HOODIE_DARK);
        }
        Facing::Down => {
            ellipse(x, ty - 8.0, 6.0 + bw * 6.5, 5.5 + bw * 4.0, HOODIE_LIGHT);
            // The bib.
            gfx::tri((x - 7.0, ty - 20.5), (x + 7.0, ty - 20.5), (x, ty - 9.5), WHITE);
            gfx::circle(x, ty - 15.0, 1.3, TOMATO);
            gfx::circle(x - 3.0, ty - 18.5, 1.0, TOMATO);
            gfx::circle(x + 3.0, ty - 18.5, 1.0, TOMATO);
        }
        _ => {
            ellipse(x + side * (3.0 + bw * 4.0), ty - 8.0, 5.0 + bw * 5.5, 6.0 + bw * 3.5, HOODIE_LIGHT);
            gfx::tri((x + side * 2.0, ty - 20.5), (x + side * 9.0, ty - 19.0), (x + side * 7.0, ty - 11.0), WHITE);
        }
    }

    // Arms in front.
    // Where the mouth is: the head's middle is 33 above the torso's base.
    let mouth_y = ty - 24.0;
    let raise = p.reach.clamp(0.0, 1.0);
    let hand = if side == 0.0 {
        let rest = (x + tw / 2.0 + 1.0, ty - 6.0);
        let up = (x + 8.0, mouth_y + 5.0);
        if p.facing == Facing::Down {
            let lx = x - tw / 2.0 - 1.0;
            gfx::line(x - tw / 2.0 + 3.0, ty - 16.0, lx, ty - 6.0, 5.5, HOODIE_DARK);
            gfx::circle(lx, ty - 5.5, 2.8, SKIN);
        }
        let h = (rest.0 + (up.0 - rest.0) * raise, rest.1 + (up.1 - rest.1) * raise);
        if p.facing == Facing::Down {
            gfx::line(x + tw / 2.0 - 3.0, ty - 16.0, h.0, h.1, 5.5, HOODIE_DARK);
            gfx::circle(h.0, h.1 + 0.5, 2.8, SKIN);
        } else {
            for k in [-1.0, 1.0] {
                gfx::line(x + k * (tw / 2.0 - 3.0), ty - 16.0, x + k * (tw / 2.0 + 1.0), ty - 6.0, 5.5, HOODIE_DARK);
            }
        }
        h
    } else {
        let rest = (x + side * 3.0 + swing * 3.0, ty - 5.0);
        let up = (x + side * 12.0, mouth_y + 4.0);
        let h = (rest.0 + (up.0 - rest.0) * raise, rest.1 + (up.1 - rest.1) * raise);
        gfx::line(x + side * 1.0, ty - 16.0, h.0, h.1, 5.5, HOODIE_DARK);
        gfx::circle(h.0, h.1 + 0.5, 2.8, SKIN);
        h
    };

    head(x, ty - 33.0, p, side);

    if let (Some(d), true) = (p.holding, p.facing != Facing::Up) {
        food(hand.0 - 1.0, hand.1 - 1.0, DISHES[d].topping, 0.62);
    }
}

fn head(x: f32, hy: f32, p: &Pose, side: f32) {
    let danger = p.mood == Mood::Danger;
    let skin = if danger { mix(SKIN, rgb(0xC9E4A6), 0.35 + 0.15 * (p.t * 3.0).sin()) } else { SKIN };
    let chew = p.chew.map(|c| (c * PI * 7.0).sin().abs()).unwrap_or(0.0);
    let puff = p.chew.map(|c| (1.0 - c) * 0.6 + chew * 0.4).unwrap_or(0.0);
    let pins = |bx: f32, by: f32| {
        gfx::line(bx - 7.0, by - 5.0, bx + 8.0, by + 3.0, 1.8, rgb(0xC0392B));
        gfx::line(bx - 5.0, by - 8.0, bx + 6.0, by + 5.0, 1.8, rgb(0xE9C46A));
    };

    if p.facing == Facing::Up {
        gfx::circle(x, hy, 14.0, HAIR);
        gfx::circle(x, hy - 14.0, 6.0, HAIR);
        pins(x, hy - 15.0);
        ellipse(x, hy + 6.0, 9.0, 4.0, mix(HAIR, WHITE, 0.08));
        return;
    }

    if side == 0.0 {
        gfx::circle(x, hy - 1.0, 14.5, HAIR);
        gfx::circle(x, hy + 1.5, 13.0, skin);
        // Cheeks, puffed while chewing.
        for k in [-1.0, 1.0] {
            gfx::circle(x + k * (9.0 + puff * 2.0), hy + 6.0, 4.0 + puff * 2.2, skin);
        }
        gfx::sector(x, hy - 1.0, 14.5, 180.0, 360.0, HAIR);
        for dx in [-9.0, -3.0, 3.0, 9.0] {
            gfx::circle(x + dx, hy - 2.0, 4.0, HAIR);
        }
        rrect(x - 15.0, hy - 4.0, 5.0, 14.0, 2.5, HAIR);
        rrect(x + 10.0, hy - 4.0, 5.0, 14.0, 2.5, HAIR);
        gfx::circle(x + 6.0, hy - 15.0, 6.0, HAIR);
        pins(x + 6.0, hy - 16.0);
        for k in [-1.0, 1.0] {
            eye(x + k * 5.0, hy + 3.5, p.mood, p.t, k);
        }
        for k in [-1.0, 1.0] {
            ellipse(x + k * (8.5 + puff), hy + 7.5, 2.8 + puff, 1.6, fade(BLUSH, 0.75));
        }
        mouth(x, hy + 9.0, p, chew);
    } else {
        gfx::circle(x, hy + 1.0, 13.0, skin);
        gfx::circle(x + side * (8.0 + puff * 1.5), hy + 6.0, 4.0 + puff * 2.4, skin);
        let (a0, a1) = if side > 0.0 { (90.0, 270.0) } else { (-90.0, 90.0) };
        gfx::sector(x, hy, 14.2, a0, a1, HAIR);
        gfx::sector(x, hy - 1.0, 14.2, 180.0, 360.0, HAIR);
        gfx::circle(x + side * 9.0, hy - 3.0, 4.0, HAIR);
        gfx::circle(x - side * 7.0, hy - 13.0, 6.0, HAIR);
        pins(x - side * 7.0, hy - 14.0);
        eye(x + side * 6.0, hy + 3.0, p.mood, p.t, side);
        ellipse(x + side * (7.5 + puff), hy + 7.5, 2.4 + puff, 1.5, fade(BLUSH, 0.75));
        mouth(x + side * 9.5, hy + 8.5, p, chew);
    }
    if p.mood == Mood::Stuffed || danger {
        let k = if side == 0.0 { 1.0 } else { -side };
        let (sx, sy) = (x + k * 13.0, hy - 6.0 + (p.t * 2.0).sin().abs() * 2.0);
        gfx::circle(sx, sy, 2.3, rgb(0x8FD3F4));
        gfx::tri((sx - 2.1, sy - 0.5), (sx + 2.1, sy - 0.5), (sx, sy - 5.5), rgb(0x8FD3F4));
    }
}

fn eye(x: f32, y: f32, mood: Mood, t: f32, k: f32) {
    match mood {
        Mood::Happy => {
            // Blink now and then.
            let blink = (t * 0.7 + k * 0.01).fract() > 0.965;
            if blink {
                gfx::line(x - 2.2, y, x + 2.2, y, 1.4, INK);
            } else {
                ellipse(x, y, 2.1, 2.8, INK);
                gfx::circle(x + 0.7, y - 1.0, 0.85, WHITE);
            }
        }
        Mood::Stuffed => gfx::curve((x - 2.4, y + 0.8), (x, y - 2.2), (x + 2.4, y + 0.8), 1.4, INK),
        Mood::Danger => {
            gfx::line(x - 2.0, y - 2.0, x + 2.0, y + 2.0, 1.4, INK);
            gfx::line(x - 2.0, y + 2.0, x + 2.0, y - 2.0, 1.4, INK);
        }
        Mood::Starry => {
            fx::star(x, y, 3.6 + (t * 8.0).sin() * 0.5, rgb(0xF4C430));
            gfx::circle(x, y, 0.8, WHITE);
        }
    }
}

fn mouth(x: f32, y: f32, p: &Pose, chew: f32) {
    if p.chew.is_some() {
        ellipse(x, y, 2.4, 0.8 + 1.6 * chew, rgb(0x8E2C3B));
    } else if p.reach > 0.2 {
        ellipse(x, y, 2.2 + p.reach * 0.6, 1.2 + p.reach * 1.6, rgb(0x8E2C3B));
    } else if p.mood == Mood::Danger {
        gfx::curve((x - 2.5, y + 1.0), (x, y - 1.5), (x + 2.5, y + 1.0), 1.2, INK);
    } else {
        gfx::curve((x - 2.5, y - 0.8), (x, y + 1.8), (x + 2.5, y - 0.8), 1.2, INK);
    }
}

// ---- The chef ----------------------------------------------------------

/// The carousel's chef, feet at `(x, y)`. `serve` is 1 just as a plate goes out.
pub fn chef(x: f32, y: f32, serve: f32, t: f32) {
    let bob = (t * 1.8).sin() * 0.5;
    ellipse(x, y, 12.0, 4.0, fade(INK, 0.2));
    rrect(x - 11.0, y - 24.0 + bob, 22.0, 22.0, 7.0, rgb(0x2F4858));
    rrect(x - 7.0, y - 18.0 + bob, 14.0, 17.0, 3.0, rgb(0xF4F1EA));
    let up = serve.clamp(0.0, 1.0);
    for k in [-1.0, 1.0] {
        let hand = (x + k * (13.0 - 5.0 * up), y - 10.0 - 26.0 * up + bob);
        gfx::line(x + k * 9.0, y - 20.0 + bob, hand.0, hand.1, 5.0, rgb(0x2F4858));
        gfx::circle(hand.0, hand.1, 2.6, SKIN);
    }
    let hy = y - 33.0 + bob;
    gfx::circle(x, hy, 11.5, SKIN);
    gfx::sector(x, hy - 1.0, 12.0, 180.0, 360.0, rgb(0x2B2B2B));
    gfx::rect(x - 12.0, hy - 5.5, 24.0, 4.0, WHITE);
    gfx::tri((x + 10.0, hy - 4.0), (x + 16.0, hy - 8.0), (x + 15.0, hy - 1.0), WHITE);
    for k in [-1.0, 1.0] {
        gfx::curve((x + k * 4.0 - 2.0, hy + 2.5), (x + k * 4.0, hy + 0.5), (x + k * 4.0 + 2.0, hy + 2.5), 1.3, INK);
    }
    gfx::curve((x - 3.0, hy + 5.5), (x, hy + 8.0), (x + 3.0, hy + 5.5), 1.3, INK);
    // A neat moustache.
    ellipse(x - 2.0, hy + 5.0, 2.4, 1.0, rgb(0x2B2B2B));
    ellipse(x + 2.0, hy + 5.0, 2.4, 1.0, rgb(0x2B2B2B));
}

// ---- The restaurant ----------------------------------------------------

/// The back wall: price board, lanterns, the kitchen pass and a window.
pub fn back_wall(t: f32) {
    gfx::grad_v(0.0, 0.0, 640.0, 104.0, rgb(0xF7E8CC), rgb(0xEBD3AA));
    // Wallpaper dots.
    for row in 0..5 {
        for col in 0..33 {
            let x = col as f32 * 20.0 + if row % 2 == 0 { 0.0 } else { 10.0 };
            gfx::circle(x, 18.0 + row as f32 * 18.0, 1.4, fade(rgb(0xD9B98A), 0.6));
        }
    }
    gfx::rect(0.0, 0.0, 640.0, 9.0, WALNUT_DARK);
    gfx::grad_v(0.0, 9.0, 640.0, 8.0, fade(INK, 0.25), fade(INK, 0.0));
    // Wainscot.
    gfx::rect(0.0, 100.0, 640.0, 5.0, WALNUT);
    gfx::rect(0.0, 105.0, 640.0, 28.0, rgb(0xB5543B));
    for i in 0..21 {
        gfx::rect(i as f32 * 32.0 + 2.0, 109.0, 28.0, 20.0, rgb(0xA84B33));
    }
    gfx::rect(0.0, 131.0, 640.0, 5.0, WALNUT_DARK);

    // Price board on the left: rim colors and their worth in coin pips.
    rrect(18.0, 28.0, 134.0, 70.0, 6.0, WALNUT_DARK);
    rrect(22.0, 32.0, 126.0, 62.0, 4.0, rgb(0x34433B));
    for (i, (rim, pips)) in [(Rim::White, 1), (Rim::Green, 2), (Rim::Red, 3), (Rim::Gold, 5)].iter().enumerate() {
        let (bx, by) = (38.0 + (i % 2) as f32 * 62.0, 48.0 + (i / 2) as f32 * 29.0);
        ellipse(bx, by + 1.0, 10.0, 5.0, fade(INK, 0.4));
        ellipse(bx, by, 10.0, 5.5, rim_color(*rim));
        ellipse(bx, by, 7.0, 3.6, rgb(0xFFFDF7));
        for k in 0..*pips {
            gfx::circle(bx + 16.0 + k as f32 * 6.5, by, 2.5, COIN);
        }
    }

    // Window on the right: an evening street.
    rrect(488.0, 26.0, 134.0, 72.0, 5.0, WALNUT_DARK);
    gfx::grad_v(494.0, 32.0, 122.0, 60.0, rgb(0x1E2A4A), rgb(0x55507A));
    gfx::circle(598.0, 46.0, 8.0, rgb(0xFFF1C1));
    gfx::circle(602.0, 43.0, 7.0, rgb(0x2A3456));
    for (i, (sx, sy)) in [(508.0, 40.0), (532.0, 37.0), (566.0, 48.0), (520.0, 54.0), (580.0, 36.0)].iter().enumerate() {
        let tw = 0.5 + 0.5 * (t * 1.7 + i as f32 * 1.3).sin();
        fx::star(*sx, *sy, 2.0 + tw * 1.5, fade(rgb(0xFFF6D5), 0.5 + 0.5 * tw));
    }
    for (bx, bw, bh) in [(494.0, 20.0, 20.0), (514.0, 16.0, 28.0), (530.0, 24.0, 16.0), (554.0, 18.0, 24.0), (572.0, 26.0, 14.0), (598.0, 18.0, 22.0)] {
        gfx::rect(bx, 92.0 - bh, bw, bh, rgb(0x2A2440));
        for wy in [92.0 - bh + 5.0, 92.0 - bh + 12.0] {
            if wy < 88.0 {
                gfx::rect(bx + 4.0, wy, 3.0, 3.0, rgb(0xFFD27F));
            }
        }
    }
    gfx::rect(553.0, 32.0, 4.0, 60.0, WALNUT_DARK);
    gfx::rect(494.0, 60.0, 122.0, 4.0, WALNUT_DARK);

    // The kitchen pass, with steam rising behind the curtain.
    rrect(232.0, 46.0, 176.0, 62.0, 6.0, WALNUT_DARK);
    gfx::rect(240.0, 52.0, 160.0, 50.0, rgb(0x2A1F1D));
    gfx::grad_v(240.0, 66.0, 160.0, 36.0, fade(rgb(0xFFB56B), 0.0), fade(rgb(0xFFB56B), 0.6));
    for i in 0..4 {
        let k = (t * 0.5 + i as f32 * 0.25).fract();
        let sx = 266.0 + i as f32 * 36.0 + (t * 2.0 + i as f32).sin() * 4.0;
        gfx::circle(sx, 98.0 - k * 40.0, 4.0 + k * 6.0, fade(WHITE, 0.25 * (1.0 - k)));
    }
    noren(240.0, 52.0, 160.0, t);
    gfx::rect(228.0, 100.0, 184.0, 8.0, WALNUT);
    gfx::rect(228.0, 106.0, 184.0, 3.0, WALNUT_DARK);

    lantern(178.0, t, 0.0);
    lantern(462.0, t, 1.7);
}

fn noren(x: f32, y: f32, w: f32, t: f32) {
    let panel = (w - 8.0) / 3.0;
    for i in 0..3 {
        let px = x + 4.0 + i as f32 * panel;
        let sway = (t * 1.4 + i as f32 * 0.8).sin() * 2.0;
        let (h, c) = (30.0, rgb(0x2F4858));
        gfx::tri((px + 1.0, y), (px + panel - 1.0, y), (px + panel - 1.0 + sway, y + h), c);
        gfx::tri((px + 1.0, y), (px + panel - 1.0 + sway, y + h), (px + 1.0 + sway, y + h), c);
        gfx::rect(px + 1.0, y, panel - 2.0, 4.0, rgb(0x223845));
    }
    // A spinning plate emblem across the middle panel.
    let cx = x + w / 2.0 + (t * 1.4 + 0.8).sin();
    gfx::ring(cx, y + 17.0, 6.0, 8.5, 0.0, 360.0, CREAM);
    gfx::circle(cx, y + 17.0, 2.6, CREAM);
    let a = t * 2.0;
    gfx::circle(cx + a.cos() * 12.0, y + 17.0 + a.sin() * 4.5, 1.6, CREAM);
}

fn lantern(x: f32, t: f32, phase: f32) {
    gfx::glow(x, 52.0, 70.0, fade(rgb(0xFFB45E), 0.35));
    gfx::line(x, 9.0, x, 28.0, 1.5, INK);
    let deg = (t * 1.2 + phase).sin() * 4.0;
    gfx::transformed(x, 28.0, deg, 1.0, 1.0, || {
        gfx::rect(-6.0, 0.0, 12.0, 4.0, INK);
        ellipse(0.0, 24.0, 16.0, 20.0, rgb(0xE4572E));
        ellipse(-4.0, 22.0, 7.0, 15.0, fade(rgb(0xFF8A5B), 0.6));
        for k in [-12.0, -4.0, 4.0, 12.0] {
            let w = (1.0 - (k / 20.0f32).powi(2)).sqrt() * 16.0;
            gfx::line(-w, 24.0 + k, w, 24.0 + k, 1.0, fade(rgb(0xB23A1C), 0.8));
        }
        gfx::rect(-6.0, 42.0, 12.0, 4.0, INK);
        gfx::line(0.0, 46.0, 0.0, 54.0, 1.5, MUSTARD);
        gfx::circle(0.0, 55.0, 2.0, MUSTARD);
    });
}

/// The wooden floor, with warm light pooling round the counter.
pub fn floor_boards() {
    let rows = 16;
    for i in 0..rows {
        let y = 136.0 + i as f32 * 22.0;
        let c = if i % 2 == 0 { rgb(0xE2B27B) } else { rgb(0xDBA872) };
        gfx::rect(0.0, y, 640.0, 22.0, c);
        gfx::rect(0.0, y, 640.0, 1.5, rgb(0xC4915A));
        let off = (i * 97 % 160) as f32;
        let mut x = off;
        while x < 640.0 {
            gfx::rect(x, y, 1.5, 22.0, rgb(0xC8955E));
            x += 160.0;
        }
    }
    gfx::grad_v(0.0, 136.0, 640.0, 26.0, fade(INK, 0.28), fade(INK, 0.0));
    gfx::glow(CX, CY + 10.0, 330.0, fade(rgb(0xFFE6BA), 0.35));
    gfx::grad_v(0.0, 400.0, 640.0, 80.0, fade(INK, 0.0), fade(INK, 0.22));
}

/// The carousel counter: its shadow, front, top, the moving belt and the prep space.
pub fn island(belt_travel: f32) {
    let (x0, w) = (CX - HALF - OUTER, 2.0 * (HALF + OUTER));
    rrect(x0 - 8.0, CY - OUTER + 12.0, w + 16.0, 2.0 * OUTER + 14.0, OUTER + 8.0, fade(INK, 0.18));
    rrect(x0, CY - OUTER + SIDE, w, 2.0 * OUTER, OUTER, WALNUT_DARK);
    for i in 0..18 {
        let gx = x0 + 30.0 + i as f32 * (w - 60.0) / 17.0;
        gfx::rect(gx, CY + OUTER - 8.0, 2.0, SIDE + 4.0, fade(rgb(0x4A2E20), 0.8));
    }
    rrect(x0, CY - OUTER, w, 2.0 * OUTER, OUTER, WALNUT);
    gfx::rrect_lines(x0, CY - OUTER, w, 2.0 * OUTER, OUTER, 2.0, rgb(0xA8754F));

    let r_out = R + BELT;
    let r_in = R - BELT;
    rrect(CX - HALF - r_out, CY - r_out, 2.0 * (HALF + r_out), 2.0 * r_out, r_out, rgb(0x56606E));
    // Slats, moving with the belt.
    let step = 12.0;
    let n = (perimeter() / step) as i32;
    let shift = belt_travel.rem_euclid(step);
    for i in 0..n {
        let (p, nrm) = belt_point(i as f32 * step + shift);
        let a = p + nrm * (BELT - 1.0);
        let b = p - nrm * (BELT - 1.0);
        gfx::line(a.x, a.y, b.x, b.y, 1.5, rgb(0x6E7889));
    }
    gfx::rrect_lines(CX - HALF - r_out, CY - r_out, 2.0 * (HALF + r_out), 2.0 * r_out, r_out, 2.0, rgb(0xB8C2CF));
    rrect(CX - HALF - r_in, CY - r_in, 2.0 * (HALF + r_in), 2.0 * r_in, r_in, rgb(0xB8C2CF));
    rrect(CX - HALF - r_in + 2.0, CY - r_in + 2.0, 2.0 * (HALF + r_in) - 4.0, 2.0 * r_in - 4.0, r_in - 2.0, MAPLE);

    // Prep space: a board with a fish, a rice tub and a teapot.
    rrect(CX - 110.0, CY - 16.0, 46.0, 26.0, 4.0, rgb(0xD9A066));
    rrect(CX - 104.0, CY - 10.0, 30.0, 12.0, 5.0, rgb(0xF4845F));
    for i in 0..3 {
        let sx = CX - 98.0 + i as f32 * 8.0;
        gfx::line(sx, CY - 9.0, sx - 3.0, CY + 1.0, 1.2, rgb(0xFFD0B0));
    }
    ellipse(CX + 90.0, CY + 2.0, 20.0, 12.0, rgb(0x9C6B45));
    ellipse(CX + 90.0, CY - 1.0, 20.0, 11.0, rgb(0xB98455));
    ellipse(CX + 90.0, CY - 1.0, 16.0, 8.0, RICE);
    gfx::line(CX + 78.0, CY - 8.0, CX + 104.0, CY - 16.0, 2.0, rgb(0xD9A066));
    ellipse(CX + 132.0, CY + 4.0, 9.0, 7.0, rgb(0x4F7C6B));
    ellipse(CX + 132.0, CY - 2.0, 4.0, 2.0, rgb(0x3E6455));
}

/// A potted plant, its pot standing at `(x, y)`.
pub fn plant(x: f32, y: f32, t: f32) {
    ellipse(x, y, 18.0, 5.0, fade(INK, 0.25));
    let leaf = rgb(0x4E9F5B);
    for (i, a) in [-60.0f32, -30.0, -5.0, 20.0, 50.0, -95.0, 80.0].iter().enumerate() {
        let sway = (t * 1.1 + i as f32).sin() * 3.0;
        let r = (a + sway - 90.0).to_radians();
        let len = 26.0 + (i % 3) as f32 * 6.0;
        let (ex, ey) = (x + r.cos() * len, y - 22.0 + r.sin() * len);
        let c = if i % 2 == 0 { leaf } else { rgb(0x3E8A4B) };
        gfx::curve((x, y - 18.0), ((x + ex) / 2.0 - 4.0, (y - 18.0 + ey) / 2.0 - 6.0), (ex, ey), 5.0, c);
        gfx::circle(ex, ey, 3.0, c);
    }
    gfx::tri((x - 14.0, y - 22.0), (x + 14.0, y - 22.0), (x + 10.0, y), rgb(0xC8643C));
    gfx::tri((x - 14.0, y - 22.0), (x + 10.0, y), (x - 10.0, y), rgb(0xC8643C));
    gfx::rect(x - 16.0, y - 26.0, 32.0, 6.0, rgb(0xB5543B));
}

// ---- Icons -------------------------------------------------------------

pub fn coin(x: f32, y: f32, r: f32) {
    gfx::circle(x, y + r * 0.15, r, COIN_EDGE);
    gfx::circle(x, y, r, COIN);
    gfx::ring(x, y, r * 0.62, r * 0.74, 0.0, 360.0, COIN_EDGE);
    gfx::circle(x - r * 0.3, y - r * 0.35, r * 0.2, fade(WHITE, 0.8));
}

/// A clock face with `left` of the meal remaining, 1..0.
pub fn clock(x: f32, y: f32, r: f32, left: f32, c: Color) {
    gfx::circle(x, y, r, CREAM);
    gfx::sector(x, y, r - 2.0, -90.0, -90.0 + 360.0 * left, fade(c, 0.85));
    gfx::ring(x, y, r - 1.6, r, 0.0, 360.0, INK);
    gfx::circle(x, y, 1.6, INK);
}

pub fn plate_stack(x: f32, y: f32, n: u32) {
    let k = n.clamp(1, 4);
    for i in 0..k {
        let yy = y - i as f32 * 3.5;
        ellipse(x, yy + 1.2, 10.0, 4.6, rgb(0xB9B3A4));
        ellipse(x, yy, 10.0, 4.6, rgb(0xF4F0E6));
    }
    ellipse(x, y - (k - 1) as f32 * 3.5, 6.5, 2.6, rgb(0xE1DACB));
}

/// A round tummy: the fullness gauge's badge.
pub fn tummy(x: f32, y: f32, r: f32, belly: f32) {
    let g = 1.0 + belly.clamp(0.0, 1.5) * 0.12;
    gfx::circle(x, y, r * g, HOODIE);
    gfx::circle(x, y + 1.5, r * 0.62 * g, HOODIE_LIGHT);
    gfx::circle(x, y + 2.0, 1.4, HOODIE_DARK);
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
            dish(cx, cy + 6.0 * s, 0, 2.0 * s);
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
                gfx::line(cx + 9.0 * s, cy - 17.0 * s + i as f32 * 7.0 * s, cx + 27.0 * s, cy - 17.0 * s + i as f32 * 7.0 * s, 1.5 * s, rgb(0xC9C2B2));
            }
            gfx::line(cx + 10.0 * s, cy + 6.0 * s, cx + 16.0 * s, cy + 12.0 * s, 3.0 * s, RED);
            gfx::line(cx + 16.0 * s, cy + 12.0 * s, cx + 30.0 * s, cy - 6.0 * s, 3.0 * s, RED);
        }
        CardArt::Rainbow => {
            gfx::ring(cx, cy, 24.0 * s, 32.0 * s, 0.0, 360.0, fade(rgb(0x56606E), 0.85));
            for i in 0..6 {
                let a = t * 0.6 + i as f32 * PI / 3.0;
                let (px, py) = (cx + a.cos() * 28.0 * s, cy + a.sin() * 28.0 * s);
                food(px, py + 2.0 * s, DISHES[i].topping, 0.9 * s);
            }
            fx::star(cx, cy, 9.0 * s, MUSTARD);
        }
        CardArt::Golden => {
            gfx::glow(cx, cy, 40.0 * s, fade(rgb(0xFFE08A), 0.8));
            dish(cx, cy + 6.0 * s, 6, 2.1 * s);
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
            gfx::tri((cx + 15.0 * s, cy - 4.0 * s - hop), (cx + 24.0 * s, cy - 1.0 * s - hop), (cx + 15.0 * s, cy + 2.0 * s - hop), rgb(0xF28C28));
            ellipse(cx + 8.0 * s, cy + 2.0 * s - hop, 3.0 * s, 1.8 * s, fade(BLUSH, 0.8));
            food(cx - 26.0 * s, cy + 18.0 * s, DISHES[4].topping, 0.9 * s);
        }
        CardArt::Combo => {
            let rims = [Rim::White, Rim::Green, Rim::Red, Rim::Green, Rim::Gold];
            for (i, r) in rims.iter().enumerate() {
                let yy = cy + 20.0 * s - i as f32 * 7.5 * s;
                let wob = (t * 3.0 + i as f32 * 0.7).sin() * i as f32 * 0.6 * s;
                plate(cx - 10.0 * s + wob, yy, *r, 1.3 * s, 1.0);
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
            rrect(cx - 44.0, cy - 6.0, 88.0, 12.0, 6.0, MAPLE);
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
                gfx::circle(cx - 14.0 + k as f32 * 14.0, cy - 22.0 - p * 22.0, 6.0 + p * 6.0, fade(WHITE, 0.5 * (1.0 - p)));
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
