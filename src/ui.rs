//! Interface pieces shared by the screens: the palette, button badges and
//! prompts, panels, bars, stars and the lock.

use crate::gfx::{self, fade, mix, rgb, rrect, Align};
use raylib_sys::Color;

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
    pub const RED: Color = rgb(0xD7263D);
    pub const WALNUT: Color = rgb(0x8B5A3C);
    pub const BLUSH: Color = rgb(0xFF8FA3);
    pub const WHITE: Color = rgb(0xFFFFFF);
    pub const GOLD: Color = rgb(0xFFC93C);
}
use pal::*;

/// A rounded button badge with its label, such as "A". Returns its width.
pub fn button_badge(label: &str, x: f32, y: f32, size: f32, fill: Color, ink: Color) -> f32 {
    let w = gfx::measure(label, size * 0.72).max(size * 0.5) + size * 0.55;
    rrect(x, y, w, size, size / 2.0, fill);
    gfx::text(label, x + w / 2.0, y + size * 0.1, size * 0.72, ink, Align::Center);
    w
}

/// Button prompts along the bottom of the safe area, right-aligned.
pub fn prompts(items: &[(&str, &str)], light: bool) {
    let size = 22.0;
    let (fill, ink, label) = if light { (CREAM, INK, CREAM) } else { (INK, CREAM, INK) };
    let widths: Vec<f32> = items
        .iter()
        .map(|(b, a)| gfx::measure(b, size * 0.72).max(size * 0.5) + size * 0.55 + 5.0 + gfx::measure(a, 16.0))
        .collect();
    let total: f32 = widths.iter().sum::<f32>() + 14.0 * (items.len() as f32 - 1.0);
    let mut x = 624.0 - total;
    rrect(x - 10.0, 440.0, total + 20.0, 32.0, 16.0, fade(if light { INK } else { CREAM }, 0.55));
    for ((b, a), w) in items.iter().zip(widths) {
        let bw = button_badge(b, x, 445.0, size, fill, ink);
        gfx::text(a, x + bw + 5.0, 447.0, 16.0, label, Align::Left);
        x += w + 14.0;
    }
}

/// A HUD panel: deep teal with a thin cream rule inside and a soft drop shadow.
pub fn panel(x: f32, y: f32, w: f32, h: f32) {
    panel_in(x, y, w, h, HOUSE, 0.92);
}

pub fn panel_in(x: f32, y: f32, w: f32, h: f32, c: Color, a: f32) {
    rrect(x + 2.0, y + 4.0, w, h, 12.0, fade(INK, 0.35 * a));
    rrect(x, y, w, h, 12.0, fade(c, a));
    rrect(x + 2.0, y + 2.0, w - 4.0, h * 0.45, 10.0, fade(WHITE, 0.06 * a));
    gfx::rrect_lines(x + 3.0, y + 3.0, w - 6.0, h - 6.0, 9.0, 1.2, fade(rgb(0xF3E3BF), 0.5 * a));
}

/// A rounded progress bar, `k` 0..1 full.
pub fn bar(x: f32, y: f32, w: f32, h: f32, k: f32, fill: Color) {
    rrect(x, y, w, h, h / 2.0, fade(INK, 0.55));
    let k = k.clamp(0.0, 1.0);
    if k > 0.0 {
        rrect(x, y, (w * k).max(h), h, h / 2.0, fill);
        rrect(x + 2.0, y + 1.5, (w * k - 4.0).max(h - 4.0), h * 0.35, h / 4.0, fade(WHITE, 0.25));
    }
}

/// A five-pointed star; `fill` 0..1 fills it from the left.
pub fn star(x: f32, y: f32, r: f32, fill: f32, on: Color, off: Color) {
    let pts: Vec<(f32, f32)> = (0..10)
        .map(|i| {
            let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
            let rr = if i % 2 == 0 { r } else { r * 0.45 };
            (x + a.cos() * rr, y + a.sin() * rr)
        })
        .collect();
    let draw = |c: Color, shrink: f32| {
        for i in 0..10 {
            let p = pts[i];
            let q = pts[(i + 1) % 10];
            let s = |pt: (f32, f32)| (x + (pt.0 - x) * shrink, y + (pt.1 - y) * shrink);
            gfx::tri((x, y), s(p), s(q), c);
        }
    };
    draw(mix(off, INK, 0.3), 1.12);
    draw(off, 1.0);
    if fill > 0.0 {
        draw(mix(off, on, fill.clamp(0.0, 1.0)), 1.0 * fill.clamp(0.0, 1.0).sqrt());
    }
}

pub fn lock(x: f32, y: f32, c: Color) {
    gfx::ring(x, y - 6.0, 5.0, 7.5, 180.0, 360.0, c);
    gfx::rect(x - 7.5, y - 6.0, 2.5, 4.0, c);
    gfx::rect(x + 5.0, y - 6.0, 2.5, 4.0, c);
    rrect(x - 10.0, y - 3.0, 20.0, 15.0, 3.0, c);
    gfx::circle(x, y + 3.0, 2.2, fade(INK, 0.6));
}

/// A tick or a cross in a circle.
pub fn mark(x: f32, y: f32, r: f32, ok: Option<bool>) {
    match ok {
        Some(true) => {
            gfx::circle(x, y, r, MINT);
            gfx::line(x - r * 0.45, y, x - r * 0.1, y + r * 0.4, r * 0.28, INK);
            gfx::line(x - r * 0.1, y + r * 0.4, x + r * 0.5, y - r * 0.4, r * 0.28, INK);
        }
        Some(false) => {
            gfx::circle(x, y, r, RED);
            gfx::line(x - r * 0.4, y - r * 0.4, x + r * 0.4, y + r * 0.4, r * 0.28, CREAM);
            gfx::line(x - r * 0.4, y + r * 0.4, x + r * 0.4, y - r * 0.4, r * 0.28, CREAM);
        }
        None => {
            gfx::ring(x, y, r * 0.7, r, 0.0, 360.0, fade(CREAM, 0.7));
        }
    }
}

/// A soft gradient with drifting dots, behind menus.
pub fn dotted_backdrop(top: Color, bottom: Color, t: f32) {
    gfx::grad_v(0.0, 0.0, gfx::W, gfx::H, top, bottom);
    let off = (t * 8.0).rem_euclid(32.0);
    for row in -1..17 {
        for col in -1..22 {
            let x = col as f32 * 32.0 + off + if row % 2 == 0 { 0.0 } else { 16.0 };
            let y = row as f32 * 32.0 + off;
            gfx::circle(x, y, 2.2, fade(CREAM, 0.1));
        }
    }
}
