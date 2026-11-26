//! Meshes: the baked model pack, a builder for shapes made in code, and the
//! upload to raylib.
//!
//! `assets/models/props.mesh` (written by `scripts/assets.py bake`) is
//! little-endian: `OEM1`, a texture count and each texture's file name (a
//! u16 length and UTF-8 bytes), then a part count and each part: its model's
//! name, a texture index (-1 for none), vertex and index counts, positions,
//! normals and UVs as f32, colours as RGBA bytes and u16 indices padded to
//! four bytes. A model is every part carrying its name.

use crate::m3::{v3, M4, V3};
use raylib_sys::{self as ffi, Color, Mesh};
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

/// One part of a model, on the CPU.
#[derive(Clone, Default)]
pub struct Part {
    /// Index into `Pack::textures`, if textured.
    pub tex: Option<usize>,
    pub pos: Vec<f32>,
    pub nrm: Vec<f32>,
    pub uv: Vec<f32>,
    pub col: Vec<u8>,
    pub idx: Vec<u16>,
}

pub struct Pack {
    pub textures: Vec<String>,
    pub models: HashMap<String, Vec<Part>>,
}

struct Reader<'a> {
    b: &'a [u8],
    o: usize,
}

impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let s = self.b.get(self.o..self.o + n).ok_or("the model pack is cut short")?;
        self.o += n;
        Ok(s)
    }
    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn str(&mut self) -> Result<String, String> {
        let n = self.u16()? as usize;
        Ok(String::from_utf8_lossy(self.take(n)?).into_owned())
    }
    fn f32s(&mut self, n: usize) -> Result<Vec<f32>, String> {
        Ok(self.take(n * 4)?.chunks(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect())
    }
}

impl Pack {
    pub fn parse(b: &[u8]) -> Result<Pack, String> {
        let mut r = Reader { b, o: 0 };
        if r.take(4)? != b"OEM1" {
            return Err("not a model pack".into());
        }
        let nt = r.u32()? as usize;
        let textures = (0..nt).map(|_| r.str()).collect::<Result<Vec<_>, _>>()?;
        let n = r.u32()? as usize;
        let mut models: HashMap<String, Vec<Part>> = HashMap::new();
        for _ in 0..n {
            let name = r.str()?;
            let tex = i32::from_le_bytes(r.take(4)?.try_into().unwrap());
            let (nv, ni) = (r.u32()? as usize, r.u32()? as usize);
            let pos = r.f32s(nv * 3)?;
            let nrm = r.f32s(nv * 3)?;
            let uv = r.f32s(nv * 2)?;
            let col = r.take(nv * 4)?.to_vec();
            let idx = r.take(ni * 2)?.chunks(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
            if ni % 2 == 1 {
                r.take(2)?;
            }
            let tex = (tex >= 0).then_some(tex as usize);
            models.entry(name).or_default().push(Part { tex, pos, nrm, uv, col, idx });
        }
        Ok(Pack { textures, models })
    }
}

/// The extent of some parts.
pub fn bounds(parts: &[Part]) -> (V3, V3) {
    let mut lo = v3(f32::MAX, f32::MAX, f32::MAX);
    let mut hi = v3(f32::MIN, f32::MIN, f32::MIN);
    for p in parts {
        for c in p.pos.chunks(3) {
            lo = v3(lo.x.min(c[0]), lo.y.min(c[1]), lo.z.min(c[2]));
            hi = v3(hi.x.max(c[0]), hi.y.max(c[1]), hi.z.max(c[2]));
        }
    }
    (lo, hi)
}

// ---- Building ------------------------------------------------------------

/// Vertices and triangles on the CPU, before upload.
#[derive(Clone, Default)]
pub struct Builder {
    pub pos: Vec<f32>,
    pub nrm: Vec<f32>,
    pub uv: Vec<f32>,
    pub col: Vec<u8>,
    pub idx: Vec<u16>,
}

impl Builder {
    pub fn verts(&self) -> usize {
        self.pos.len() / 3
    }

    pub fn tris(&self) -> usize {
        self.idx.len() / 3
    }

    pub fn vert(&mut self, p: V3, n: V3, uv: (f32, f32), c: Color) -> u16 {
        let i = self.verts() as u16;
        self.pos.extend(p.arr());
        self.nrm.extend(n.arr());
        self.uv.extend([uv.0, uv.1]);
        self.col.extend([c.r, c.g, c.b, c.a]);
        i
    }

    /// A flat quad `a b c d` (counter-clockwise seen from its front).
    pub fn quad(&mut self, q: [V3; 4], uv: [(f32, f32); 4], c: Color) {
        let n = (q[1] - q[0]).cross(q[2] - q[0]).norm();
        let i: Vec<u16> = (0..4).map(|k| self.vert(q[k], n, uv[k], c)).collect();
        self.idx.extend([i[0], i[1], i[2], i[0], i[2], i[3]]);
    }

    /// A box from `lo` to `hi`; UVs are world metres over `tile`, so tiling
    /// textures keep their scale on every face. `bottom` adds the underside.
    pub fn cuboid(&mut self, lo: V3, hi: V3, c: Color, tile: f32, bottom: bool) {
        let (a, b) = (lo, hi);
        let t = |u: f32, v: f32| (u / tile, v / tile);
        // Top, then the four sides, then the bottom.
        self.quad(
            [v3(a.x, b.y, b.z), v3(b.x, b.y, b.z), v3(b.x, b.y, a.z), v3(a.x, b.y, a.z)],
            [t(a.x, b.z), t(b.x, b.z), t(b.x, a.z), t(a.x, a.z)],
            c,
        );
        self.quad(
            [v3(a.x, a.y, b.z), v3(b.x, a.y, b.z), v3(b.x, b.y, b.z), v3(a.x, b.y, b.z)],
            [t(a.x, a.y), t(b.x, a.y), t(b.x, b.y), t(a.x, b.y)],
            c,
        );
        self.quad(
            [v3(b.x, a.y, a.z), v3(a.x, a.y, a.z), v3(a.x, b.y, a.z), v3(b.x, b.y, a.z)],
            [t(b.x, a.y), t(a.x, a.y), t(a.x, b.y), t(b.x, b.y)],
            c,
        );
        self.quad(
            [v3(b.x, a.y, b.z), v3(b.x, a.y, a.z), v3(b.x, b.y, a.z), v3(b.x, b.y, b.z)],
            [t(b.z, a.y), t(a.z, a.y), t(a.z, b.y), t(b.z, b.y)],
            c,
        );
        self.quad(
            [v3(a.x, a.y, a.z), v3(a.x, a.y, b.z), v3(a.x, b.y, b.z), v3(a.x, b.y, a.z)],
            [t(a.z, a.y), t(b.z, a.y), t(b.z, b.y), t(a.z, b.y)],
            c,
        );
        if bottom {
            self.quad(
                [v3(a.x, a.y, a.z), v3(b.x, a.y, a.z), v3(b.x, a.y, b.z), v3(a.x, a.y, b.z)],
                [t(a.x, a.z), t(b.x, a.z), t(b.x, b.z), t(a.x, b.z)],
                c,
            );
        }
    }

    /// An upright cylinder standing on `base`, with a top cap.
    pub fn cylinder(&mut self, base: V3, r: f32, h: f32, sides: usize, c: Color, top: Color) {
        let ring: Vec<(f32, f32)> = (0..=sides).map(|i| (i as f32 / sides as f32 * TAU).sin_cos()).collect();
        for i in 0..sides {
            let (s0, c0) = ring[i];
            let (s1, c1) = ring[i + 1];
            let (n0, n1) = (v3(s0, 0.0, c0), v3(s1, 0.0, c1));
            let u0 = i as f32 / sides as f32;
            let u1 = (i + 1) as f32 / sides as f32;
            let a = self.vert(base + n0 * r, n0, (u0 * r * TAU, 0.0), c);
            let b = self.vert(base + n1 * r, n1, (u1 * r * TAU, 0.0), c);
            let d = self.vert(base + n1 * r + v3(0.0, h, 0.0), n1, (u1 * r * TAU, h), c);
            let e = self.vert(base + n0 * r + v3(0.0, h, 0.0), n0, (u0 * r * TAU, h), c);
            self.idx.extend([a, b, d, a, d, e]);
        }
        let up = v3(0.0, 1.0, 0.0);
        let centre = self.vert(base + v3(0.0, h, 0.0), up, (0.5, 0.5), top);
        for i in 0..sides {
            let (s0, c0) = ring[i];
            let (s1, c1) = ring[i + 1];
            let a = self.vert(base + v3(s0 * r, h, c0 * r), up, (0.5 + s0 * 0.5, 0.5 + c0 * 0.5), top);
            let b = self.vert(base + v3(s1 * r, h, c1 * r), up, (0.5 + s1 * 0.5, 0.5 + c1 * 0.5), top);
            self.idx.extend([centre, a, b]);
        }
    }

    /// Adds a baked part, moved by `m` and its colours multiplied by `tint`.
    pub fn part(&mut self, p: &Part, m: &M4, tint: Color) {
        let base = self.verts() as u16;
        for k in 0..p.pos.len() / 3 {
            let q = m.point(v3(p.pos[k * 3], p.pos[k * 3 + 1], p.pos[k * 3 + 2]));
            let n = m.dir(v3(p.nrm[k * 3], p.nrm[k * 3 + 1], p.nrm[k * 3 + 2])).norm();
            let c = &p.col[k * 4..k * 4 + 4];
            let mul = |a: u8, b: u8| ((a as u32 * b as u32) / 255) as u8;
            let col = Color { r: mul(c[0], tint.r), g: mul(c[1], tint.g), b: mul(c[2], tint.b), a: mul(c[3], tint.a) };
            self.vert(q, n, (p.uv[k * 2], p.uv[k * 2 + 1]), col);
        }
        let flip = {
            let x = m.dir(v3(1.0, 0.0, 0.0));
            let y = m.dir(v3(0.0, 1.0, 0.0));
            let z = m.dir(v3(0.0, 0.0, 1.0));
            x.cross(y).dot(z) < 0.0
        };
        for t in p.idx.chunks(3) {
            if flip {
                self.idx.extend([base + t[0], base + t[2], base + t[1]]);
            } else {
                self.idx.extend([base + t[0], base + t[1], base + t[2]]);
            }
        }
    }

    pub fn upload(&self, dynamic: bool) -> Mesh {
        unsafe {
            let mut m: Mesh = std::mem::zeroed();
            let n = self.verts();
            if n == 0 {
                return m;
            }
            m.vertexCount = n as i32;
            m.triangleCount = (self.idx.len() / 3) as i32;
            m.vertices = copy(&self.pos);
            m.normals = copy(&self.nrm);
            m.texcoords = copy(&self.uv);
            m.colors = copy(&self.col);
            m.indices = copy(&self.idx);
            ffi::UploadMesh(&mut m, dynamic);
            m
        }
    }
}

/// Copies into raylib's allocator, which UnloadMesh frees.
unsafe fn copy<T: Copy>(src: &[T]) -> *mut T {
    let bytes = std::mem::size_of_val(src).max(1);
    let p = ffi::MemAlloc(bytes as u32) as *mut T;
    std::ptr::copy_nonoverlapping(src.as_ptr(), p, src.len());
    p
}

/// Static geometry sorted by texture, split so no mesh needs more than
/// 16-bit indices.
#[derive(Default)]
pub struct Batch {
    pub groups: Vec<(usize, Vec<Builder>)>,
}

impl Batch {
    /// The builder for texture `tex` with room for `verts` more vertices.
    pub fn get(&mut self, tex: usize, verts: usize) -> &mut Builder {
        let gi = match self.groups.iter().position(|(t, _)| *t == tex) {
            Some(i) => i,
            None => {
                self.groups.push((tex, vec![Builder::default()]));
                self.groups.len() - 1
            }
        };
        let list = &mut self.groups[gi].1;
        if list.last().is_none_or(|b| b.verts() + verts > 60_000) {
            list.push(Builder::default());
        }
        list.last_mut().unwrap()
    }

    pub fn tris(&self) -> usize {
        self.groups.iter().flat_map(|(_, l)| l.iter()).map(Builder::tris).sum()
    }
}

// ---- Shapes along the belt -------------------------------------------------

/// A stadium (two straights and two half circles) in the floor plane, in
/// metres: centre, half the straight's length and the end radius.
#[derive(Clone, Copy, Debug)]
pub struct Stadium {
    pub cx: f32,
    pub cz: f32,
    pub half: f32,
    pub r: f32,
}

impl Stadium {
    pub fn perimeter(&self) -> f32 {
        4.0 * self.half + TAU * self.r
    }

    /// The point at arc length `s` (clockwise seen from above, starting at
    /// the far straight's west end) and its outward normal, offset outward by
    /// `off` metres.
    pub fn at(&self, s: f32, off: f32) -> (V3, V3) {
        let p = self.perimeter();
        let s = s.rem_euclid(p);
        let st = 2.0 * self.half;
        let arc = PI * self.r;
        let (cx, cz, h, r) = (self.cx, self.cz, self.half, self.r + off);
        if s < st {
            (v3(cx - h + s, 0.0, cz - r), v3(0.0, 0.0, -1.0))
        } else if s < st + arc {
            let a = -PI / 2.0 + (s - st) / self.r;
            let n = v3(a.cos(), 0.0, a.sin());
            (v3(cx + h, 0.0, cz) + n * r, n)
        } else if s < 2.0 * st + arc {
            (v3(cx + h - (s - st - arc), 0.0, cz + r), v3(0.0, 0.0, 1.0))
        } else {
            let a = PI / 2.0 + (s - 2.0 * st - arc) / self.r;
            let n = v3(a.cos(), 0.0, a.sin());
            (v3(cx - h, 0.0, cz) + n * r, n)
        }
    }

    /// Sample positions along the path: dense on the curves, ends of straights.
    fn samples(&self) -> Vec<f32> {
        let st = 2.0 * self.half;
        let arc = PI * self.r;
        let n = 14;
        let mut s = vec![0.0];
        for (start, len, steps) in [(0.0, st, 1), (st, arc, n), (st + arc, st, 1), (2.0 * st + arc, arc, n)] {
            for i in 1..=steps {
                s.push(start + len * i as f32 / steps as f32);
            }
        }
        s
    }

    /// A flat band between offsets `a` and `b` at height `y`; `u` runs along
    /// the path in metres over `tile`, `v` across it.
    pub fn band(&self, out: &mut Builder, a: f32, b: f32, y: f32, c: Color, tile: f32) {
        let s = self.samples();
        let up = v3(0.0, 1.0, 0.0);
        for w in s.windows(2) {
            let (p0, n0) = self.at(w[0], 0.0);
            let (p1, n1) = self.at(w[1], 0.0);
            let q = [
                p0 + n0 * a + v3(0.0, y, 0.0),
                p0 + n0 * b + v3(0.0, y, 0.0),
                p1 + n1 * b + v3(0.0, y, 0.0),
                p1 + n1 * a + v3(0.0, y, 0.0),
            ];
            let uv = [(w[0] / tile, 0.0), (w[0] / tile, 1.0), (w[1] / tile, 1.0), (w[1] / tile, 0.0)];
            let i: Vec<u16> = (0..4).map(|k| out.vert(q[k], up, uv[k], c)).collect();
            // Wound to face up whichever way the offsets run.
            if a < b {
                out.idx.extend([i[0], i[2], i[1], i[0], i[3], i[2]]);
            } else {
                out.idx.extend([i[0], i[1], i[2], i[0], i[2], i[3]]);
            }
        }
    }

    /// An upright wall along the path at offset `off`, from `y0` to `y1`,
    /// facing outward (`out_face`) or inward.
    pub fn wall(&self, out: &mut Builder, off: f32, y0: f32, y1: f32, c: Color, out_face: bool, tile: f32) {
        let s = self.samples();
        for w in s.windows(2) {
            let (p0, n0) = self.at(w[0], off);
            let (p1, n1) = self.at(w[1], off);
            let (n0, n1) = if out_face { (n0, n1) } else { (-n0, -n1) };
            let i = [
                out.vert(p0 + v3(0.0, y0, 0.0), n0, (w[0] / tile, y0 / tile), c),
                out.vert(p1 + v3(0.0, y0, 0.0), n1, (w[1] / tile, y0 / tile), c),
                out.vert(p1 + v3(0.0, y1, 0.0), n1, (w[1] / tile, y1 / tile), c),
                out.vert(p0 + v3(0.0, y1, 0.0), n0, (w[0] / tile, y1 / tile), c),
            ];
            if out_face {
                out.idx.extend([i[0], i[2], i[1], i[0], i[3], i[2]]);
            } else {
                out.idx.extend([i[0], i[1], i[2], i[0], i[2], i[3]]);
            }
        }
    }

    /// A flat cap filling the inside of offset `off` at height `y`.
    pub fn cap(&self, out: &mut Builder, off: f32, y: f32, c: Color, tile: f32) {
        let s = self.samples();
        let up = v3(0.0, 1.0, 0.0);
        let centre = v3(self.cx, y, self.cz);
        let ci = out.vert(centre, up, (self.cx / tile, self.cz / tile), c);
        for w in s.windows(2) {
            let (p0, _) = self.at(w[0], off);
            let (p1, _) = self.at(w[1], off);
            let a = out.vert(p0 + v3(0.0, y, 0.0), up, (p0.x / tile, p0.z / tile), c);
            let b = out.vert(p1 + v3(0.0, y, 0.0), up, (p1.x / tile, p1.z / tile), c);
            out.idx.extend([ci, b, a]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stadium_is_continuous() {
        let st = Stadium { cx: 1.0, cz: 2.0, half: 3.0, r: 0.8 };
        let p = st.perimeter();
        let mut prev = st.at(0.0, 0.0).0;
        let mut s = 0.01;
        while s <= p {
            let q = st.at(s, 0.0).0;
            assert!((q - prev).len() < 0.02, "jump at {s}");
            prev = q;
            s += 0.01;
        }
    }

    #[test]
    fn bands_face_up() {
        let st = Stadium { cx: 0.0, cz: 0.0, half: 1.0, r: 0.5 };
        let mut b = Builder::default();
        st.band(&mut b, -0.1, 0.1, 0.0, Color { r: 255, g: 255, b: 255, a: 255 }, 1.0);
        for t in b.idx.chunks(3) {
            let p = |i: u16| v3(b.pos[i as usize * 3], b.pos[i as usize * 3 + 1], b.pos[i as usize * 3 + 2]);
            let n = (p(t[1]) - p(t[0])).cross(p(t[2]) - p(t[0]));
            assert!(n.y >= -1e-6, "a triangle faces down: {n:?}");
        }
    }
}
