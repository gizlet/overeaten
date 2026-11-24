//! Short-lived decoration: crumbs, sparkles, floating numbers, banners and
//! screen shake. Drawn in canvas coordinates.

use crate::gfx::{self, fade, rgb, Align};
use raylib_sys::Color;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Crumb,
    Sparkle,
    Sweat,
    Heart,
    Puff,
}

pub struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max: f32,
    size: f32,
    kind: Kind,
    color: Color,
}

pub struct Floater {
    x: f32,
    y: f32,
    text: String,
    color: Color,
    size: f32,
    life: f32,
    max: f32,
}

pub struct Banner {
    pub title: String,
    pub sub: String,
    pub color: Color,
    pub t: f32,
    pub max: f32,
}

#[derive(Default)]
pub struct Fx {
    parts: Vec<Particle>,
    floats: Vec<Floater>,
    pub banner: Option<Banner>,
    shake: f32,
    rng: u32,
    /// 1 right after the coin count changes, easing to 0: for HUD bumps.
    pub bump_coins: f32,
    pub bump_plates: f32,
    pub time: f32,
}

impl Fx {
    pub fn clear(&mut self) {
        self.parts.clear();
        self.floats.clear();
        self.banner = None;
        self.shake = 0.0;
    }

    fn rand(&mut self) -> f32 {
        if self.rng == 0 {
            self.rng = 0x9E37_79B9;
        }
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x % 10_000) as f32 / 10_000.0
    }

    fn spread(&mut self, k: f32) -> f32 {
        (self.rand() * 2.0 - 1.0) * k
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        for p in &mut self.parts {
            p.life += dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            match p.kind {
                Kind::Crumb | Kind::Sweat => p.vy += 420.0 * dt,
                Kind::Puff | Kind::Heart => {
                    p.vx *= 1.0 - 2.0 * dt;
                    p.vy *= 1.0 - 1.5 * dt;
                }
                Kind::Sparkle => {}
            }
        }
        self.parts.retain(|p| p.life < p.max);
        for f in &mut self.floats {
            f.life += dt;
            f.y -= 34.0 * dt * (1.0 - f.life / f.max);
        }
        self.floats.retain(|f| f.life < f.max);
        if let Some(b) = &mut self.banner {
            b.t += dt;
            if b.t > b.max {
                self.banner = None;
            }
        }
        self.shake = (self.shake - dt * 3.0).max(0.0);
        self.bump_coins = (self.bump_coins - dt * 3.0).max(0.0);
        self.bump_plates = (self.bump_plates - dt * 3.0).max(0.0);
    }

    pub fn offset(&mut self) -> (f32, f32) {
        if self.shake <= 0.0 {
            return (0.0, 0.0);
        }
        let k = self.shake * self.shake * 5.0;
        (self.spread(k), self.spread(k))
    }

    pub fn shake(&mut self, amount: f32) {
        self.shake = self.shake.max(amount);
    }

    pub fn crumbs(&mut self, x: f32, y: f32, color: Color, n: usize) {
        for _ in 0..n {
            let (vx, vy, life, size) =
                (self.spread(70.0), -60.0 - self.rand() * 90.0, 0.45 + self.rand() * 0.3, 1.2 + self.rand() * 1.6);
            self.parts.push(Particle { x, y, vx, vy, life: 0.0, max: life, size, kind: Kind::Crumb, color });
        }
    }

    pub fn sparkles(&mut self, x: f32, y: f32, color: Color, n: usize, spread: f32) {
        for _ in 0..n {
            let (dx, dy) = (self.spread(spread), self.spread(spread * 0.7));
            let (vx, vy, max, size) =
                (self.spread(20.0), -20.0 - self.rand() * 30.0, 0.5 + self.rand() * 0.5, 2.5 + self.rand() * 2.5);
            self.parts.push(Particle {
                x: x + dx,
                y: y + dy,
                vx,
                vy,
                life: 0.0,
                max,
                size,
                kind: Kind::Sparkle,
                color,
            });
        }
    }

    pub fn sweat(&mut self, x: f32, y: f32) {
        let vx = self.spread(30.0) + 20.0;
        self.parts.push(Particle {
            x,
            y,
            vx,
            vy: -70.0,
            life: 0.0,
            max: 0.6,
            size: 3.0,
            kind: Kind::Sweat,
            color: rgb(0x7EC8E3),
        });
    }

    pub fn hearts(&mut self, x: f32, y: f32, n: usize) {
        for _ in 0..n {
            let (vx, vy) = (self.spread(40.0), -50.0 - self.rand() * 30.0);
            self.parts.push(Particle {
                x,
                y,
                vx,
                vy,
                life: 0.0,
                max: 0.9,
                size: 4.0,
                kind: Kind::Heart,
                color: rgb(0xFF6B81),
            });
        }
    }

    pub fn puffs(&mut self, x: f32, y: f32, n: usize) {
        for _ in 0..n {
            let (vx, vy) = (self.spread(60.0), self.spread(25.0) - 10.0);
            let size = 5.0 + self.rand() * 4.0;
            self.parts.push(Particle {
                x,
                y,
                vx,
                vy,
                life: 0.0,
                max: 0.5,
                size,
                kind: Kind::Puff,
                color: rgb(0xFFF7EC),
            });
        }
    }

    pub fn float(&mut self, x: f32, y: f32, text: String, color: Color, size: f32) {
        self.floats.push(Floater { x, y, text, color, size, life: 0.0, max: 1.1 });
    }

    pub fn banner(&mut self, title: &str, sub: &str, color: Color, max: f32) {
        self.banner = Some(Banner { title: title.into(), sub: sub.into(), color, t: 0.0, max });
    }

    pub fn draw_world(&self) {
        for p in &self.parts {
            let k = p.life / p.max;
            let a = 1.0 - k * k;
            match p.kind {
                Kind::Crumb => gfx::circle(p.x, p.y, p.size, fade(p.color, a)),
                Kind::Sweat => {
                    gfx::circle(p.x, p.y, p.size, fade(p.color, a));
                    gfx::tri(
                        (p.x - p.size * 0.9, p.y - 0.5),
                        (p.x + p.size * 0.9, p.y - 0.5),
                        (p.x, p.y - p.size * 2.4),
                        fade(p.color, a),
                    );
                }
                Kind::Sparkle => star(p.x, p.y, p.size * (1.0 - k * 0.5), fade(p.color, a)),
                Kind::Heart => heart(p.x, p.y, p.size * (0.6 + k * 0.6), fade(p.color, a)),
                Kind::Puff => gfx::circle(p.x, p.y, p.size * (0.6 + k), fade(p.color, a * 0.8)),
            }
        }
        for f in &self.floats {
            let k = f.life / f.max;
            let pop = gfx::ease_back((f.life / 0.18).min(1.0));
            let a = if k > 0.7 { 1.0 - (k - 0.7) / 0.3 } else { 1.0 };
            let size = f.size * (0.6 + 0.4 * pop);
            gfx::text_outlined(
                &f.text,
                f.x,
                f.y - size / 2.0,
                size,
                fade(f.color, a),
                fade(rgb(0x2B2D42), a),
                2.0,
                Align::Center,
            );
        }
    }

    pub fn draw_banner(&self) {
        let Some(b) = &self.banner else { return };
        let inn = gfx::ease_back(b.t / 0.35);
        let out = ((b.max - b.t) / 0.3).clamp(0.0, 1.0);
        let a = out;
        let y = 196.0;
        let w = 400.0 * (0.4 + 0.6 * inn);
        gfx::rrect(320.0 - w / 2.0, y - 8.0, w, 84.0, 18.0, fade(rgb(0x2B2D42), 0.82 * a));
        gfx::rrect_lines(320.0 - w / 2.0 + 4.0, y - 4.0, w - 8.0, 76.0, 15.0, 2.0, fade(b.color, 0.9 * a));
        let size = 40.0 * (0.5 + 0.5 * inn);
        gfx::text_outlined(
            &b.title,
            320.0,
            y + 2.0 + (40.0 - size) / 2.0,
            size,
            fade(b.color, a),
            fade(rgb(0x2B2D42), a),
            2.5,
            Align::Center,
        );
        if !b.sub.is_empty() {
            gfx::text(&b.sub, 320.0, y + 48.0, 18.0, fade(rgb(0xFFF4E0), a), Align::Center);
        }
    }
}

/// A four-pointed sparkle.
pub fn star(x: f32, y: f32, r: f32, c: Color) {
    let t = r * 0.28;
    gfx::tri((x, y - r), (x - t, y), (x + t, y), c);
    gfx::tri((x, y + r), (x - t, y), (x + t, y), c);
    gfx::tri((x - r, y), (x, y - t), (x, y + t), c);
    gfx::tri((x + r, y), (x, y - t), (x, y + t), c);
}

pub fn heart(x: f32, y: f32, r: f32, c: Color) {
    gfx::circle(x - r * 0.5, y - r * 0.2, r * 0.6, c);
    gfx::circle(x + r * 0.5, y - r * 0.2, r * 0.6, c);
    gfx::tri((x - r * 1.05, y), (x + r * 1.05, y), (x, y + r * 1.1), c);
}
