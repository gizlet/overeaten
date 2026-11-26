//! The cards: 3D portraits rendered once into textures, and the faces and
//! backs drawn around them. Each rarity has its own frame: walnut for
//! common cards, steel with gems for rare ones, gold with a moving rainbow
//! sheen for holographic ones and violet with a glitch for hidden ones.

use crate::gfx::{self, fade, mix, rgb, rrect, Align};
use crate::hero::{Act, Hero};
use crate::m3::{v3, M4};
use crate::render::{self, begin3d, end3d, Camera, Gl, Lighting, Look};
use crate::rules::{Art, Book, Card, Hue, Lang, Rarity, DISHES, GOLDEN};
use crate::scene::{food_at, plate_at, prop_at};
use crate::ui::pal::*;
use raylib_sys::{self as ffi, Color, Rectangle, RenderTexture2D, Texture2D, Vector2};
use std::f32::consts::PI;

/// Portrait size in pixels: a card's art window at 2× the canvas.
pub const PW: i32 = 192;
pub const PH: i32 = 176;

pub struct Portraits {
    made: Vec<(Art, RenderTexture2D)>,
    detail: Option<RenderTexture2D>,
}

/// Every card face's picture.
fn arts() -> Vec<Art> {
    let mut v: Vec<Art> = (0..8).map(Art::Level).collect();
    v.extend((0..4).map(Art::Hero));
    v.extend((0..DISHES.len()).map(Art::Dish));
    v.extend([Art::Ambulance, Art::Exact, Art::Combo, Art::Empty]);
    v
}

impl Portraits {
    pub fn new() -> Portraits {
        Portraits { made: Vec::new(), detail: None }
    }

    /// Renders every portrait; call outside drawing.
    pub fn render_all(&mut self, gl: &Gl, heroes: &mut [Hero]) {
        for h in heroes.iter_mut() {
            h.update(0.5, Act::Idle, 0.0);
        }
        for art in arts() {
            let rt = unsafe { ffi::LoadRenderTexture(PW, PH) };
            unsafe { ffi::SetTextureFilter(rt.texture, ffi::TextureFilter::TEXTURE_FILTER_BILINEAR as i32) };
            paint(gl, heroes, art, 0.0, 0.0, rt);
            self.made.push((art, rt));
        }
    }

    pub fn get(&self, art: Art) -> Option<Texture2D> {
        self.made.iter().find(|(a, _)| *a == art).map(|(_, rt)| rt.texture)
    }

    /// Renders the enlarged card's picture turning; call outside drawing.
    pub fn render_detail(&mut self, gl: &Gl, heroes: &[Hero], art: Art, turn: f32, t: f32) -> Texture2D {
        let rt = *self.detail.get_or_insert_with(|| unsafe {
            let rt = ffi::LoadRenderTexture(PW * 2, PH * 2);
            ffi::SetTextureFilter(rt.texture, ffi::TextureFilter::TEXTURE_FILTER_BILINEAR as i32);
            rt
        });
        paint(gl, heroes, art, turn, t, rt);
        rt.texture
    }

    pub fn detail(&self) -> Option<Texture2D> {
        self.detail.map(|rt| rt.texture)
    }
}

/// Draws `art` into `rt`, the model turned by `turn` radians.
fn paint(gl: &Gl, heroes: &[Hero], art: Art, turn: f32, t: f32, rt: RenderTexture2D) {
    let (w, h) = (rt.texture.width, rt.texture.height);
    let mut light = Lighting::day();
    light.sun_dir = v3(-0.4, 0.8, 0.6).norm();
    light.add_light(v3(0.8, 1.2, 1.4), 3.0, rgb(0x6A5A50));
    let spin = M4::yaw(turn);
    let (cam, fit) = match art {
        Art::Hero(_) | Art::Ambulance => (Camera { eye: v3(0.0, 1.05, 3.1), target: v3(0.0, 0.78, 0.0), fovy: 0.62 }, 1.0),
        Art::Combo => (Camera { eye: v3(0.0, 0.75, 1.1), target: v3(0.0, 0.16, 0.0), fovy: 0.6 }, 1.0),
        Art::Level(_) => (Camera { eye: v3(0.0, 0.85, 1.25), target: v3(0.0, 0.14, 0.0), fovy: 0.62 }, 1.0),
        _ => (Camera { eye: v3(0.0, 0.48, 0.72), target: v3(0.0, 0.06, 0.0), fovy: 0.58 }, 1.0),
    };
    let _ = fit;
    unsafe {
        ffi::BeginTextureMode(rt);
        ffi::ClearBackground(Color { r: 0, g: 0, b: 0, a: 0 });
    }
    begin3d(&cam, 0, 0, w, h, h);
    gl.light(&light, cam.eye);
    match art {
        Art::Dish(d) => plate_at(gl, &spin, v3(0.0, 0.0, 0.0), 0.3, 1.0, Some(d), DISHES[d].hue, true, t),
        Art::Hero(i) => {
            let hero = &heroes[i];
            let m = hero.place(v3(0.0, 0.0, 0.0), 0.35, 1.55).then(&spin);
            hero.draw(gl, &m, &Look { rim: 0.6, ..Look::default() }, None);
        }
        Art::Level(i) => level_art(gl, &spin, i, t),
        Art::Ambulance => {
            let hero = &heroes[0];
            let m = hero.place(v3(0.0, 0.0, 0.0), 0.0, 1.4).then(&M4::pitch(-PI / 2.0)).then(&M4::translate(v3(0.0, 0.55, -0.3))).then(&spin);
            hero.draw(gl, &m, &Look { tint: (rgb(0xFF6A6A), 0.25), ..Look::default() }, None);
            let cross = M4::scale(0.12, 0.12, 0.12).then(&M4::translate(v3(0.0, 1.25, 0.0)));
            prop_at(gl, "bowl_small", &cross, &Look { color: rgb(0xE23B3B), ..Look::default() });
        }
        Art::Exact => {
            plate_at(gl, &spin, v3(-0.12, 0.0, 0.0), 0.3, 0.8, Some(4), Hue::Yellow, true, t);
            plate_at(gl, &spin, v3(0.14, 0.0, 0.06), -0.4, 0.8, Some(2), Hue::White, true, t);
        }
        Art::Combo => {
            let hues = [Hue::Red, Hue::White, Hue::Yellow, Hue::Red, Hue::White, Hue::Yellow, Hue::Red, Hue::Gold];
            for (k, hue) in hues.iter().enumerate() {
                let at = v3((k as f32 * 1.7).sin() * 0.015, k as f32 * 0.032, 0.0);
                plate_at(gl, &spin, at, k as f32 * 0.3, 0.85, None, *hue, true, t);
            }
            food_at(gl, &spin, v3(0.0, 8.0 * 0.032 + 0.01, 0.0), 0.2, 0.85, GOLDEN, true, t, false);
        }
        Art::Empty => {
            plate_at(gl, &spin, v3(0.0, 0.0, 0.0), 0.0, 1.0, None, Hue::White, true, t);
            let soy = M4::scale(1.3, 1.3, 1.3).then(&M4::translate(v3(0.2, 0.0, 0.12))).then(&spin);
            prop_at(gl, "soy", &soy, &Look::default());
        }
    }
    end3d(w, h);
    unsafe { ffi::EndTextureMode() };
}

/// A room's card: its signature piece beside a plate.
fn level_art(gl: &Gl, spin: &M4, i: usize, t: f32) {
    let (name, s, at, dish) = match i {
        0 => ("menu", 0.42, v3(-0.2, 0.0, -0.1), 0),
        1 => ("pillar_A", 0.12, v3(-0.24, 0.0, -0.12), 1),
        2 => ("bowl_small", 0.45, v3(-0.22, 0.0, -0.05), 2),
        3 => ("table_low", 0.2, v3(-0.05, 0.0, -0.18), 3),
        4 => ("lantern_standing", 0.0, v3(0.0, 0.0, 0.0), GOLDEN),
        5 => ("lantern_standing", 0.45, v3(-0.24, 0.0, -0.1), 5),
        6 => ("crate", 0.16, v3(-0.24, 0.0, -0.12), 0),
        _ => ("jar_C_large", 0.5, v3(-0.24, 0.0, -0.1), GOLDEN),
    };
    if s > 0.0 {
        let m = M4::scale(s, s, s).then(&M4::translate(at)).then(spin);
        prop_at(gl, name, &m, &Look { rim: 0.4, ..Look::default() });
    }
    plate_at(gl, spin, v3(0.12, if i == 3 { 0.1 } else { 0.0 }, 0.08), 0.5, 1.0, Some(dish), DISHES[dish].hue, true, t);
    if i == 2 || i == 7 {
        plate_at(gl, spin, v3(-0.02, 0.0, 0.26), -0.4, 0.85, Some(1), Hue::Red, true, t);
    }
}

fn rarity_color(r: Rarity) -> (Color, Color) {
    match r {
        Rarity::Common => (WALNUT, rgb(0xC08A5E)),
        Rarity::Rare => (rgb(0x41608A), rgb(0xBFD3EA)),
        Rarity::Holo => (rgb(0xC9971C), rgb(0xFFE38A)),
        Rarity::Hidden => (rgb(0x3E2160), rgb(0xA77BD6)),
    }
}

fn book_color(b: Book) -> (Color, Color) {
    match b {
        Book::Levels => (rgb(0xF6C28B), rgb(0xE4572E)),
        Book::Heroes => (rgb(0xA8DCD1), rgb(0x2F8F83)),
        Book::Dishes => (rgb(0xFFE3D8), rgb(0xF08A7E)),
        Book::Secrets => (rgb(0x55407A), rgb(0x1B1430)),
    }
}

/// Draws a texture into a rectangle; render textures are stored upside down.
pub fn blit(t: Texture2D, x: f32, y: f32, w: f32, h: f32, flipped: bool, tint: Color) {
    let src = Rectangle {
        x: 0.0,
        y: 0.0,
        width: t.width as f32,
        height: if flipped { -(t.height as f32) } else { t.height as f32 },
    };
    unsafe {
        ffi::DrawTexturePro(t, src, Rectangle { x, y, width: w, height: h }, Vector2 { x: 0.0, y: 0.0 }, 0.0, tint);
    }
}

/// A card's face, its top-left at `(x, y)`; `pic` is its portrait.
#[allow(clippy::too_many_arguments)]
pub fn face(gl: &Gl, x: f32, y: f32, w: f32, h: f32, card: &Card, pic: Option<Texture2D>, lang: Lang, t: f32) {
    let s = w / 108.0;
    let (edge, light) = rarity_color(card.rarity);
    rrect(x + 2.0 * s, y + 4.0 * s, w, h, 9.0 * s, fade(INK, 0.35));
    rrect(x, y, w, h, 9.0 * s, edge);
    // The frame's bevel: lighter at the top.
    rrect(x + 1.5 * s, y + 1.5 * s, w - 3.0 * s, h * 0.4, 8.0 * s, fade(light, 0.35));
    let paper = if card.rarity == Rarity::Hidden { rgb(0x241B33) } else { PAPER };
    rrect(x + 4.0 * s, y + 4.0 * s, w - 8.0 * s, h - 8.0 * s, 6.0 * s, paper);
    let (ix, iy, iw, ih) = (x + 7.0 * s, y + 7.0 * s, w - 14.0 * s, h * 0.6);
    let (top, bottom) = book_color(card.book);
    gfx::grad_v(ix, iy, iw, ih, top, mix(bottom, top, 0.35));
    // A soft light behind the picture.
    gfx::glow(ix + iw / 2.0, iy + ih * 0.55, iw.min(ih) * 0.55, fade(WHITE, 0.55 + 0.1 * (t * 1.5).sin()));
    if let Some(p) = pic {
        let (pw, ph) = (iw, iw * PH as f32 / PW as f32);
        let py = iy + ih - ph;
        if card.rarity == Rarity::Holo {
            let u = &gl.holo;
            u.f("time", t);
            u.f("strength", 0.55);
            u.f("tilt", x * 0.004);
            unsafe { ffi::BeginShaderMode(u.shader) };
            blit(p, ix, py, pw, ph, true, WHITE);
            unsafe { ffi::EndShaderMode() };
        } else {
            blit(p, ix, py, pw, ph, true, WHITE);
        }
    }
    gfx::rrect_lines(ix, iy, iw, ih, 4.0 * s, 1.5 * s, fade(edge, 0.8));
    // Rarity touches.
    match card.rarity {
        Rarity::Rare => {
            for (gx, gy) in [(ix, iy), (ix + iw, iy), (ix, iy + ih), (ix + iw, iy + ih)] {
                gfx::transformed(gx, gy, 45.0, 1.0, 1.0, || {
                    gfx::rect(-3.5 * s, -3.5 * s, 7.0 * s, 7.0 * s, light);
                    gfx::rect(-2.0 * s, -2.0 * s, 4.0 * s, 4.0 * s, rgb(0x7FB3E8));
                });
            }
        }
        Rarity::Holo => {
            for k in 0..4 {
                let ph = t * 1.7 + k as f32 * 1.9;
                let a = (ph.sin() * 0.5 + 0.5).powi(3);
                let (sx, sy) = (ix + iw * (0.15 + 0.23 * k as f32), iy + ih * (0.2 + 0.18 * ((k * 7) % 4) as f32));
                crate::fx::star(sx, sy, 3.0 * s * (0.5 + a), fade(WHITE, a));
            }
        }
        Rarity::Hidden => {
            let k = ((t * 3.0).sin() * 0.5 + 0.5) * 0.25;
            for row in 0..3 {
                let yy = iy + ih * (0.2 + row as f32 * 0.3 + (t * 0.7 + row as f32).sin() * 0.04);
                gfx::rect(ix, yy, iw, 1.5 * s, fade(light, k));
            }
        }
        Rarity::Common => {}
    }
    let name = card.name.get(lang);
    let ink = if card.rarity == Rarity::Hidden { CREAM } else { INK };
    let size = (13.5 * s).max(13.0);
    gfx::text_wrapped(name, x + 5.0 * s, iy + ih + 5.0 * s, w - 10.0 * s, size, size * 1.15, ink, Align::Center);
    // Pips: one to four for the rarity.
    let pips = card.rarity as usize + 1;
    for k in 0..pips {
        let px = x + w / 2.0 + (k as f32 - (pips - 1) as f32 / 2.0) * 9.0 * s;
        crate::ui::star(px, y + h - 9.0 * s, 3.6 * s, 1.0, light, fade(edge, 0.6));
    }
}

/// A card's back, with a question mark for undiscovered cards.
pub fn back(x: f32, y: f32, w: f32, h: f32, unknown: bool) {
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

/// A card turning over: `k` 0 shows the back, 1 the face.
#[allow(clippy::too_many_arguments)]
pub fn flipping(gl: &Gl, x: f32, y: f32, w: f32, h: f32, card: &Card, pic: Option<Texture2D>, lang: Lang, t: f32, k: f32) {
    let k = k.clamp(0.0, 1.0);
    let sx = (k * PI).cos().abs().max(0.02);
    let cx = x + w / 2.0;
    gfx::transformed(cx, y, 0.0, sx, 1.0, || {
        if k < 0.5 {
            back(-w / 2.0, 0.0, w, h, false);
        } else {
            face(gl, -w / 2.0, 0.0, w, h, card, pic, lang, t);
        }
    });
    let _ = render::rgb;
}
