//! Drawing on a 640×480 canvas scaled to the window, and text from atlases
//! that hold only the characters each size has drawn.
//!
//! Everything is drawn in canvas pixels inside a `Camera2D` that scales and
//! letterboxes the canvas, so the R36S's panel draws it 1:1 and a desktop
//! window draws it larger and still sharp. Text atlases are rasterized at the
//! window's pixel size for each text size, so letters stay crisp too.

use raylib_sys::{self as ffi, Camera2D, Color, Font, Rectangle, Vector2};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::ffi::CString;

pub const W: f32 = 640.0;
pub const H: f32 = 480.0;

/// Noto Sans SC Bold, subset to GB2312 and Latin (SIL Open Font License).
static FONT: &[u8] = include_bytes!("../assets/fonts/NotoSansSC-Bold-subset.otf");

pub const fn rgb(hex: u32) -> Color {
    Color { r: (hex >> 16) as u8, g: (hex >> 8) as u8, b: hex as u8, a: 255 }
}

pub fn fade(c: Color, a: f32) -> Color {
    Color { a: (c.a as f32 * a.clamp(0.0, 1.0)) as u8, ..c }
}

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color { r: l(a.r, b.r), g: l(a.g, b.g), b: l(a.b, b.b), a: l(a.a, b.a) }
}

pub fn v(x: f32, y: f32) -> Vector2 {
    Vector2 { x, y }
}

pub fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Overshoots a little and settles: for things that pop in.
pub fn ease_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0) - 1.0;
    1.0 + t * t * (2.7 * t + 1.7)
}

// ---- The view ----------------------------------------------------------

#[derive(Clone, Copy)]
pub struct View {
    pub zoom: f32,
    pub ox: f32,
    pub oy: f32,
    /// Window pixels per canvas pixel, for text atlases.
    pub px: f32,
}

thread_local! {
    static FONTS: RefCell<Fonts> = RefCell::new(Fonts::default());
}

/// The drawing area and window pixels per drawing unit. On macOS a high-DPI
/// window draws in framebuffer pixels; elsewhere raylib scales points itself.
fn surface() -> (f32, f32, f32) {
    unsafe {
        if cfg!(target_os = "macos") {
            (ffi::GetRenderWidth() as f32, ffi::GetRenderHeight() as f32, 1.0)
        } else {
            (ffi::GetScreenWidth() as f32, ffi::GetScreenHeight() as f32, ffi::GetWindowScaleDPI().x.max(1.0))
        }
    }
}

/// Computes this frame's view and rebuilds text atlases that gained characters.
pub fn begin_frame() -> View {
    let (sw, sh, dpi) = surface();
    let zoom = (sw / W).min(sh / H).max(0.05);
    let view = View { zoom, ox: ((sw - W * zoom) / 2.0).floor(), oy: ((sh - H * zoom) / 2.0).floor(), px: zoom * dpi };
    FONTS.with(|f| f.borrow_mut().frame(view.px));
    view
}

pub fn camera(view: View, shake: (f32, f32)) -> Camera2D {
    Camera2D {
        offset: v(view.ox + shake.0 * view.zoom, view.oy + shake.1 * view.zoom),
        target: v(0.0, 0.0),
        rotation: 0.0,
        zoom: view.zoom,
    }
}

/// Covers whatever lies outside the canvas, in window coordinates.
pub fn letterbox(view: View, c: Color) {
    let (sw, sh, _) = surface();
    let (w, h) = (W * view.zoom, H * view.zoom);
    let r = |x: f32, y: f32, w: f32, h: f32| unsafe {
        if w > 0.0 && h > 0.0 {
            ffi::DrawRectangleRec(Rectangle { x, y, width: w, height: h }, c);
        }
    };
    r(0.0, 0.0, view.ox, sh);
    r(view.ox + w, 0.0, sw - view.ox - w, sh);
    r(0.0, 0.0, sw, view.oy);
    r(0.0, view.oy + h, sw, sh - view.oy - h);
}

// ---- Shapes ------------------------------------------------------------

pub fn rect(x: f32, y: f32, w: f32, h: f32, c: Color) {
    unsafe { ffi::DrawRectangleRec(Rectangle { x, y, width: w, height: h }, c) }
}

fn segs(r: f32) -> i32 {
    ((r / 1.5) as i32).clamp(6, 24)
}

pub fn rrect(x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) {
    if w <= 0.0 || h <= 0.0 || c.a == 0 {
        return;
    }
    let roundness = (2.0 * r / w.min(h)).min(1.0);
    unsafe { ffi::DrawRectangleRounded(Rectangle { x, y, width: w, height: h }, roundness, segs(r), c) }
}

pub fn rrect_lines(x: f32, y: f32, w: f32, h: f32, r: f32, thick: f32, c: Color) {
    if w <= thick || h <= thick || c.a == 0 {
        return;
    }
    let (x, y, w, h) = (x + thick / 2.0, y + thick / 2.0, w - thick, h - thick);
    let roundness = (2.0 * (r - thick / 2.0).max(0.5) / w.min(h)).min(1.0);
    unsafe { ffi::DrawRectangleRoundedLinesEx(Rectangle { x, y, width: w, height: h }, roundness, segs(r), thick, c) }
}

pub fn circle(x: f32, y: f32, r: f32, c: Color) {
    unsafe { ffi::DrawCircleV(v(x, y), r, c) }
}

pub fn ellipse(x: f32, y: f32, rx: f32, ry: f32, c: Color) {
    unsafe { ffi::DrawEllipseV(v(x, y), rx, ry, c) }
}

/// A ring between two radii, from `a0` to `a1` degrees (0 is right, clockwise on screen).
pub fn ring(x: f32, y: f32, r0: f32, r1: f32, a0: f32, a1: f32, c: Color) {
    unsafe { ffi::DrawRing(v(x, y), r0, r1, a0, a1, segs(r1) * 2, c) }
}

pub fn line(x1: f32, y1: f32, x2: f32, y2: f32, thick: f32, c: Color) {
    unsafe { ffi::DrawLineEx(v(x1, y1), v(x2, y2), thick, c) }
}

/// A filled triangle in any winding.
pub fn tri(a: (f32, f32), b: (f32, f32), c: (f32, f32), col: Color) {
    let cross = (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0);
    let (b, c) = if cross > 0.0 { (c, b) } else { (b, c) };
    unsafe { ffi::DrawTriangle(v(a.0, a.1), v(b.0, b.1), v(c.0, c.1), col) }
}

/// A quadratic curve through a control point.
pub fn curve(p1: (f32, f32), c: (f32, f32), p2: (f32, f32), thick: f32, col: Color) {
    unsafe { ffi::DrawSplineSegmentBezierQuadratic(v(p1.0, p1.1), v(c.0, c.1), v(p2.0, p2.1), thick, col) }
}

pub fn grad_v(x: f32, y: f32, w: f32, h: f32, top: Color, bottom: Color) {
    unsafe { ffi::DrawRectangleGradientEx(Rectangle { x, y, width: w, height: h }, top, bottom, bottom, top) }
}

pub fn grad_h(x: f32, y: f32, w: f32, h: f32, left: Color, right: Color) {
    unsafe { ffi::DrawRectangleGradientEx(Rectangle { x, y, width: w, height: h }, left, left, right, right) }
}

/// A soft round light that fades to nothing at `r`.
pub fn glow(x: f32, y: f32, r: f32, c: Color) {
    unsafe { ffi::DrawCircleGradient(v(x, y), r, c, fade(c, 0.0)) }
}

/// Draws what `f` draws moved to `(x, y)`, turned by `deg` and scaled.
pub fn transformed(x: f32, y: f32, deg: f32, sx: f32, sy: f32, f: impl FnOnce()) {
    unsafe {
        ffi::rlPushMatrix();
        ffi::rlTranslatef(x, y, 0.0);
        ffi::rlRotatef(deg, 0.0, 0.0, 1.0);
        ffi::rlScalef(sx, sy, 1.0);
    }
    f();
    unsafe { ffi::rlPopMatrix() };
}

// ---- Text --------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

struct Atlas {
    chars: BTreeSet<i32>,
    font: Option<Font>,
    dirty: bool,
}

#[derive(Default)]
struct Fonts {
    px: f32,
    atlases: HashMap<i32, Atlas>,
}

fn load(px: i32, chars: &BTreeSet<i32>) -> Font {
    let cps: Vec<i32> = chars.iter().copied().collect();
    unsafe {
        let f = ffi::LoadFontFromMemory(
            c".otf".as_ptr(),
            FONT.as_ptr(),
            FONT.len() as i32,
            px,
            cps.as_ptr(),
            cps.len() as i32,
        );
        ffi::SetTextureFilter(f.texture, ffi::TextureFilter::TEXTURE_FILTER_BILINEAR as i32);
        f
    }
}

impl Fonts {
    fn frame(&mut self, px: f32) {
        if (px - self.px).abs() > 1e-3 {
            for (_, a) in self.atlases.drain() {
                if let Some(f) = a.font {
                    unsafe { ffi::UnloadFont(f) };
                }
            }
            self.px = px;
        }
        for (&size, a) in self.atlases.iter_mut() {
            if a.dirty {
                if let Some(f) = a.font.take() {
                    unsafe { ffi::UnloadFont(f) };
                }
                a.font = Some(load(size, &a.chars));
                a.dirty = false;
            }
        }
    }

    /// The font for `size` canvas pixels holding `s`'s characters, as far as it can this frame.
    fn get(&mut self, size: f32, s: &str) -> Font {
        let px = ((size * self.px / 2.0).round() as i32 * 2).clamp(8, 240);
        let a =
            self.atlases.entry(px).or_insert_with(|| Atlas { chars: (32..127).collect(), font: None, dirty: false });
        for ch in s.chars() {
            if a.chars.insert(ch as i32) {
                a.dirty = true;
            }
        }
        // A new atlas loads at once; one that gained characters is rebuilt
        // next frame, since this frame's batch may still use its texture.
        if a.font.is_none() {
            a.font = Some(load(px, &a.chars));
            a.dirty = false;
        }
        a.font.unwrap()
    }
}

pub fn measure(s: &str, size: f32) -> f32 {
    let Ok(c) = CString::new(s) else { return 0.0 };
    FONTS.with(|f| {
        let font = f.borrow_mut().get(size, s);
        unsafe { ffi::MeasureTextEx(font, c.as_ptr(), size, 0.0).x }
    })
}

/// Draws `s` with its top at `y`. Returns its width.
pub fn text(s: &str, x: f32, y: f32, size: f32, color: Color, align: Align) -> f32 {
    let Ok(c) = CString::new(s) else { return 0.0 };
    FONTS.with(|f| {
        let font = f.borrow_mut().get(size, s);
        unsafe {
            let w = ffi::MeasureTextEx(font, c.as_ptr(), size, 0.0).x;
            let x = match align {
                Align::Left => x,
                Align::Center => x - w / 2.0,
                Align::Right => x - w,
            };
            ffi::DrawTextEx(font, c.as_ptr(), v(x, y), size, 0.0, color);
            w
        }
    })
}

/// Text with a solid outline, for titles and numbers over busy scenes.
pub fn text_outlined(
    s: &str,
    x: f32,
    y: f32,
    size: f32,
    color: Color,
    outline: Color,
    thick: f32,
    align: Align,
) -> f32 {
    let d = thick;
    for (dx, dy) in [
        (-d, 0.0),
        (d, 0.0),
        (0.0, -d),
        (0.0, d),
        (-d * 0.7, -d * 0.7),
        (d * 0.7, -d * 0.7),
        (-d * 0.7, d * 0.7),
        (d * 0.7, d * 0.7),
    ] {
        text(s, x + dx, y + dy, size, outline, align);
    }
    text(s, x, y, size, color, align)
}

/// Text in a box `w` wide, broken at spaces (English) or between characters
/// (Chinese). Returns the height used.
pub fn text_wrapped(s: &str, x: f32, y: f32, w: f32, size: f32, line_h: f32, color: Color, align: Align) -> f32 {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    let cjk = s.chars().any(|c| c as u32 >= 0x2E80);
    let pieces: Vec<String> =
        if cjk { s.chars().map(String::from).collect() } else { s.split(' ').map(|p| format!("{p} ")).collect() };
    for p in pieces {
        let trial = format!("{cur}{p}");
        if !cur.is_empty() && measure(trial.trim_end(), size) > w {
            lines.push(cur.trim_end().to_string());
            cur = p.trim_start().to_string();
        } else {
            cur = trial;
        }
    }
    if !cur.trim().is_empty() {
        lines.push(cur.trim_end().to_string());
    }
    let ax = match align {
        Align::Left => x,
        Align::Center => x + w / 2.0,
        Align::Right => x + w,
    };
    for (i, l) in lines.iter().enumerate() {
        text(l, ax, y + i as f32 * line_h, size, color, align);
    }
    lines.len() as f32 * line_h
}

// ---- Textures ----------------------------------------------------------

pub const WHITE: Color = rgb(0xFFFFFF);

/// Draws part of a texture into a rectangle; a negative source width mirrors it.
pub fn tex(
    t: raylib_sys::Texture2D,
    src: (f32, f32, f32, f32),
    dst: (f32, f32, f32, f32),
    origin: (f32, f32),
    tint: Color,
) {
    unsafe {
        ffi::DrawTexturePro(
            t,
            Rectangle { x: src.0, y: src.1, width: src.2, height: src.3 },
            Rectangle { x: dst.0, y: dst.1, width: dst.2, height: dst.3 },
            v(origin.0, origin.1),
            0.0,
            tint,
        )
    }
}

/// Draws what `f` draws with additive blending, for light.
pub fn additive(f: impl FnOnce()) {
    unsafe { ffi::BeginBlendMode(ffi::BlendMode::BLEND_ADDITIVE as i32) };
    f();
    unsafe { ffi::EndBlendMode() };
}
