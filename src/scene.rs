//! The rooms in 3D. `Scene::build` turns a floor plan into static meshes,
//! batched by texture (the room, the kitchen, the counters, the furniture);
//! `Scene::draw` adds what moves each frame: the belts' slats, the plates
//! and the sushi on them, the doors, the water and the visitor.
//!
//! World units are metres: x east, y up, z south. The floor plan's
//! centimetres divide by 100.

use crate::assets::{self, Assets};
use crate::game::{Round, V2};
use crate::hero::{Act, Hero};
use crate::level::{Layout, Prop as P, Shape, BELT_W, RIM, ROOM, WALK};
use crate::m3::{v3, M4, V3};
use crate::mesh::{Batch, Builder, Stadium};
use crate::render::{self, rgb, scaled, Camera, Gl, Lighting, Look};
use crate::rules::{Hue, DISHES, EVERYDAY, GOLDEN, LEVELS};
use raylib_sys::{self as ffi, Color, Mesh};
use std::f32::consts::{FRAC_PI_2, PI};

/// No texture: plain vertex colours.
const PLAIN: usize = usize::MAX;

pub fn w(p: V2, h: f32) -> V3 {
    v3(p.x / 100.0, h / 100.0, p.y / 100.0)
}

fn stadium(b: &crate::level::Belt) -> Stadium {
    Stadium { cx: b.cx / 100.0, cz: b.cy / 100.0, half: b.half / 100.0, r: b.r / 100.0 }
}

const WHITE: Color = rgb(0xFFFFFF);

/// Places a model from the pack: on `at`, turned by `yaw`, scaled.
fn put(batch: &mut Batch, a: &Assets, name: &str, at: V3, yaw: f32, s: f32, tint: Color) {
    let prop = a.prop(name);
    let m = M4::scale(s, s, s).then(&M4::yaw(yaw)).then(&M4::translate(at));
    for p in &prop.parts {
        let b = batch.get(p.tex.unwrap_or(PLAIN), p.pos.len() / 3);
        b.part(p, &m, tint);
    }
}

pub struct Scene {
    pub level: usize,
    statics: Vec<(Mesh, usize)>,
    glow: Vec<(Mesh, usize)>,
    belts: Vec<(Mesh, f32)>,
    water: Vec<Mesh>,
    /// Soft dark rings on the floor round the counters.
    shade: Vec<Mesh>,
    door_panel: Mesh,
    ring: Mesh,
    hat: Mesh,
    pub tris: usize,
    pub lighting: Lighting,
    pub cam: Camera,
    /// The room's lean this frame, for the ferry.
    pub root: M4,
    /// Lean angle in radians, for the HUD.
    pub lean: f32,
    pub clear: Color,
    /// 0 by day, 1 at night: dims the windows.
    night: f32,
}

impl Scene {
    pub fn build(level: usize) -> Scene {
        let a = assets::get();
        let l = Layout::of(level);
        let mut st = Batch::default();
        let mut glow = Batch::default();
        let floor_tex = if l.deck { a.tex("deck") } else { a.tex("floor_wood") };
        let plaster = a.tex("plaster");
        let wood = a.tex("counter");
        let (rx0, rz0, rx1, rz1) = (ROOM.0 / 100.0, ROOM.1 / 100.0, ROOM.2 / 100.0, ROOM.3 / 100.0);

        // The floor, with holes where the ponds are.
        let ponds: Vec<(f32, f32, f32, f32)> = l
            .obstacles
            .iter()
            .filter(|o| o.prop == P::Pond)
            .filter_map(|o| match o.shape {
                Shape::Rect { x, y, w, h } => Some((x / 100.0, y / 100.0, (x + w) / 100.0, (y + h) / 100.0)),
                _ => None,
            })
            .collect();
        let mut xs = vec![rx0, rx1];
        let mut zs = vec![rz0, rz1];
        for p in &ponds {
            xs.extend([p.0, p.2]);
            zs.extend([p.1, p.3]);
        }
        xs.sort_by(f32::total_cmp);
        zs.sort_by(f32::total_cmp);
        xs.dedup();
        zs.dedup();
        let tile = 1.6;
        for i in 0..xs.len() - 1 {
            for j in 0..zs.len() - 1 {
                let (x0, x1, z0, z1) = (xs[i], xs[i + 1], zs[j], zs[j + 1]);
                let (cx, cz) = ((x0 + x1) / 2.0, (z0 + z1) / 2.0);
                if ponds.iter().any(|p| cx > p.0 && cx < p.2 && cz > p.1 && cz < p.3) {
                    continue;
                }
                let b = st.get(floor_tex, 4);
                let t = |x: f32, z: f32| (x / tile, z / tile);
                b.quad(
                    [v3(x0, 0.0, z1), v3(x1, 0.0, z1), v3(x1, 0.0, z0), v3(x0, 0.0, z0)],
                    [t(x0, z1), t(x1, z1), t(x1, z0), t(x0, z0)],
                    WHITE,
                );
            }
        }
        if l.deck {
            Self::deck(&mut st, a, &l);
        } else {
            Self::walls(&mut st, &mut glow, a, plaster, wood);
        }
        Self::kitchen(&mut st, a, &l);

        // Ponds: stone edges going down to the water, and the bridge.
        let stone = rgb(0x9C978E);
        for p in &ponds {
            let b = st.get(plaster, 24);
            b.cuboid(v3(p.0, -0.35, p.1), v3(p.0 + 0.06, 0.02, p.3), stone, 1.0, false);
            b.cuboid(v3(p.2 - 0.06, -0.35, p.1), v3(p.2, 0.02, p.3), stone, 1.0, false);
        }
        if let Some((x, y, bw, bh)) = l.bridge {
            let (x0, z0, x1, z1) = (x / 100.0, y / 100.0, (x + bw) / 100.0, (y + bh) / 100.0);
            let deck = a.tex("deck");
            let b = st.get(deck, 400);
            b.cuboid(v3(x0, -0.06, z0), v3(x1, 0.05, z1), rgb(0xC9A27A), 0.8, false);
            for z in [z0 + 0.03, z1 - 0.03] {
                for k in 0..5 {
                    let px = x0 + 0.05 + (x1 - x0 - 0.1) * k as f32 / 4.0;
                    b.cuboid(v3(px - 0.03, 0.05, z - 0.03), v3(px + 0.03, 0.5, z + 0.03), rgb(0x8B5A3C), 0.5, false);
                }
                b.cuboid(v3(x0, 0.46, z - 0.035), v3(x1, 0.52, z + 0.035), rgb(0xA0673F), 0.5, false);
            }
        }

        // Raised tatami: a wooden plinth with mats on top.
        let tatami = a.tex("tatami");
        for f in &l.platforms {
            let (x0, z0, x1, z1, h) = (f.x / 100.0, f.y / 100.0, (f.x + f.w) / 100.0, (f.y + f.h) / 100.0, f.height / 100.0);
            st.get(wood, 24).cuboid(v3(x0, 0.0, z0), v3(x1, h - 0.02, z1), rgb(0xB48357), 0.8, false);
            let b = st.get(tatami, 4);
            let t = |x: f32, z: f32| (x / 0.9, z / 1.8);
            b.quad(
                [v3(x0, h, z1), v3(x1, h, z1), v3(x1, h, z0), v3(x0, h, z0)],
                [t(x0, z1), t(x1, z1), t(x1, z0), t(x0, z0)],
                WHITE,
            );
            // A step along the front.
            st.get(wood, 24).cuboid(v3(x0, 0.0, z1), v3(x1, h * 0.5, z1 + 0.22), rgb(0x9C6B45), 0.8, false);
            // Cushions round the upper belt.
            for k in 0..6 {
                let x = x0 + 1.2 + k as f32 * (x1 - x0 - 2.4) / 5.0;
                put(&mut st, a, if k % 2 == 0 { "pillow_A" } else { "pillow_B" }, v3(x, h + 0.04, z1 - 0.45), 0.0, 0.7, WHITE);
            }
        }

        // The counters the belts run through.
        for b in &l.belts {
            Self::counter(&mut st, a, b, wood);
        }

        // Things standing in the way.
        for o in &l.obstacles {
            match (o.prop, o.shape) {
                (P::Pillar, Shape::Circle { x, y, .. }) => {
                    put(&mut st, a, "pillar_A", w(V2::new(x, y), 0.0), 0.0, 0.75, rgb(0xF0E2CC));
                }
                (P::Stool, Shape::Circle { x, y, .. }) => {
                    put(&mut st, a, "chair_stool", w(V2::new(x, y), 0.0), 0.0, 0.62, WHITE);
                }
                (P::Plant, Shape::Circle { x, y, .. }) => {
                    put(&mut st, a, "cactus_medium_A", w(V2::new(x, y), 0.0), 0.6, 0.95, WHITE);
                }
                (P::Lantern, Shape::Circle { x, y, .. }) => {
                    put(&mut st, a, "post_lantern", w(V2::new(x, y), 0.0), PI, 0.62, WHITE);
                }
                (P::LowTable, Shape::Rect { x, y, w: rw, h: rh }) => {
                    let c = V2::new(x + rw / 2.0, y + rh / 2.0);
                    let floor = l.floor_at(c);
                    put(&mut st, a, "table_low", w(c, floor), 0.0, 0.5, WHITE);
                    put(&mut st, a, "bowl_small", w(c, floor + 26.0), 0.0, 0.3, WHITE);
                }
                (P::Crates, Shape::Rect { x, y, w: rw, h: rh }) => {
                    let c = V2::new(x + rw / 2.0, y + rh / 2.0);
                    put(&mut st, a, "crate", w(c, 0.0), 0.1, 0.5, WHITE);
                    put(&mut st, a, "crate", w(c, 40.0), -0.2, 0.42, WHITE);
                }
                _ => {}
            }
        }

        // The sliding doors' tracks and posts.
        for d in &l.doors {
            let b = &l.belts[d.belt];
            let y = d.y(b) / 100.0;
            let (x0, x1) = (d.x0 / 100.0, d.x1 / 100.0);
            let half = (x1 - x0) / 2.0;
            let bb = st.get(wood, 200);
            bb.cuboid(v3(x0 - half, 0.0, y - 0.05), v3(x1 + half, 0.04, y + 0.05), rgb(0x5B3A29), 0.5, false);
            bb.cuboid(v3(x0 - half, 1.3, y - 0.05), v3(x1 + half, 1.38, y + 0.05), rgb(0x5B3A29), 0.5, false);
            for px in [x0 - half, x1 + half] {
                bb.cuboid(v3(px - 0.05, 0.0, y - 0.05), v3(px + 0.05, 1.38, y + 0.05), rgb(0x6B4430), 0.5, false);
            }
        }

        // Paper lanterns hanging over each belt in the day rooms.
        if l.dark <= 0.0 && !l.deck {
            for b in &l.belts {
                for dx in [-0.55, 0.55] {
                    let at = v3(b.cx / 100.0 + dx * b.half / 100.0, 2.95, b.cy / 100.0);
                    put(&mut st, a, "lantern_hanging", at, 0.0, 0.7, WHITE);
                }
            }
        }
        // Light: daylight, or lanterns at night.
        let mut lighting = Lighting::day();
        let mut clear = rgb(0x2A2026);
        if l.dark > 0.0 {
            let k = 1.0 - l.dark * 0.82;
            lighting.sun = scaled(rgb(0xB8C4FF), k);
            lighting.shade = scaled(rgb(0x3A3560), k);
            lighting.sky = scaled(rgb(0x1E2240), 0.5 + 0.5 * k);
            clear = rgb(0x0D0C16);
            for lamp in &l.lamps {
                lighting.add_light(w(V2::new(lamp.x, lamp.y), lamp.h), lamp.reach / 100.0 * 1.15, rgb(0xFFB25C));
            }
        }
        if l.deck {
            lighting.sun_dir = v3(0.45, 0.75, 0.5).norm();
            lighting.sun = rgb(0xFFE3C4);
            lighting.shade = rgb(0x6A7AA8);
            clear = rgb(0x6FA6C9);
        }

        if l.dark <= 0.0 && !l.deck {
            for b in &l.belts {
                for dx in [-0.55, 0.55] {
                    let at = v3(b.cx / 100.0 + dx * b.half / 100.0, 2.2, b.cy / 100.0);
                    lighting.add_light(at, 3.4, rgb(0xC8925A));
                }
            }
            lighting.sun = rgb(0xF4E2C8);
            lighting.shade = rgb(0x6E6488);
        }
        let upload = |b: &Batch| -> Vec<(Mesh, usize)> {
            b.groups.iter().flat_map(|(t, list)| list.iter().map(move |bb| (bb.upload(false), *t))).collect()
        };
        let statics = upload(&st);
        let glow_meshes = upload(&glow);
        let tris = st.tris() + glow.tris();

        let belts = l
            .belts
            .iter()
            .map(|b| {
                let s = stadium(b);
                let mut bb = Builder::default();
                s.band(&mut bb, -BELT_W / 100.0, BELT_W / 100.0, b.top() / 100.0 + 0.004, WHITE, 0.25);
                (bb.upload(false), b.dir)
            })
            .collect();
        let shade = l
            .belts
            .iter()
            .map(|b| {
                let s = stadium(b);
                let y = b.floor / 100.0 + 0.006;
                let rim = RIM / 100.0;
                let mut bb = Builder::default();
                s.band(&mut bb, rim, rim + 0.1, y, Color { r: 22, g: 14, b: 20, a: 120 }, 1.0);
                s.band(&mut bb, rim + 0.1, rim + 0.3, y, Color { r: 22, g: 14, b: 20, a: 45 }, 1.0);
                bb.upload(false)
            })
            .collect();
        let mut water = Vec::new();
        for p in &ponds {
            let mut b = Builder::default();
            let t = |x: f32, z: f32| (x / 2.0, z / 2.0);
            b.quad(
                [v3(p.0, -0.12, p.3), v3(p.2, -0.12, p.3), v3(p.2, -0.12, p.1), v3(p.0, -0.12, p.1)],
                [t(p.0, p.3), t(p.2, p.3), t(p.2, p.1), t(p.0, p.1)],
                WHITE,
            );
            water.push(b.upload(false));
        }
        if l.deck {
            let mut b = Builder::default();
            let t = |x: f32, z: f32| (x / 3.0, z / 3.0);
            let (x0, z0, x1, z1) = (-30.0, -25.0, 45.0, 40.0);
            b.quad(
                [v3(x0, -0.7, z1), v3(x1, -0.7, z1), v3(x1, -0.7, z0), v3(x0, -0.7, z0)],
                [t(x0, z1), t(x1, z1), t(x1, z0), t(x0, z0)],
                WHITE,
            );
            water.push(b.upload(false));
        }
        let door_panel = {
            let mut b = Builder::default();
            b.cuboid(v3(0.0, 0.0, -0.025), v3(1.0, 1.0, 0.025), WHITE, 1.0, false);
            b.upload(false)
        };
        let ring = {
            let mut b = Builder::default();
            let n = 32;
            for i in 0..n {
                let a0 = i as f32 / n as f32 * std::f32::consts::TAU;
                let a1 = (i + 1) as f32 / n as f32 * std::f32::consts::TAU;
                let p = |a: f32, r: f32| v3(a.cos() * r, 0.0, a.sin() * r);
                b.quad([p(a0, 1.0), p(a0, 0.8), p(a1, 0.8), p(a1, 1.0)], [(0.0, 0.0); 4], WHITE);
            }
            b.upload(false)
        };
        let hat = {
            let mut b = Builder::default();
            let white = rgb(0xFBFAF6);
            b.cylinder(v3(0.0, 0.0, 0.0), 0.1, 0.12, 16, white, white);
            b.cylinder(v3(0.0, 0.1, 0.0), 0.14, 0.1, 16, white, white);
            b.upload(false)
        };
        Scene {
            level,
            statics,
            glow: glow_meshes,
            belts,
            water,
            shade,
            door_panel,
            ring,
            hat,
            tris,
            lighting,
            cam: Self::camera_for(&l, V2::new(750.0, 600.0)),
            root: M4::ID,
            lean: 0.0,
            clear,
            night: l.dark,
        }
    }

    fn walls(st: &mut Batch, glow: &mut Batch, a: &Assets, plaster: usize, wood: usize) {
        let (x0, z0, x1, z1) = (ROOM.0 / 100.0, ROOM.1 / 100.0, ROOM.2 / 100.0, ROOM.3 / 100.0);
        let h = 3.0;
        let cream = rgb(0xF6EAD6);
        let b = st.get(plaster, 64);
        // The kitchen wall and the side walls, seen from inside.
        b.quad([v3(x0, 0.0, z0), v3(x1, 0.0, z0), v3(x1, h, z0), v3(x0, h, z0)], [(x0 / 2.0, 0.0), (x1 / 2.0, 0.0), (x1 / 2.0, h / 2.0), (x0 / 2.0, h / 2.0)], cream);
        b.quad([v3(x0, 0.0, z1), v3(x0, 0.0, z0), v3(x0, h, z0), v3(x0, h, z1)], [(z1 / 2.0, 0.0), (z0 / 2.0, 0.0), (z0 / 2.0, h / 2.0), (z1 / 2.0, h / 2.0)], cream);
        b.quad([v3(x1, 0.0, z0), v3(x1, 0.0, z1), v3(x1, h, z1), v3(x1, h, z0)], [(z0 / 2.0, 0.0), (z1 / 2.0, 0.0), (z1 / 2.0, h / 2.0), (z0 / 2.0, h / 2.0)], cream);
        // Wood below, and a top beam.
        let wb = st.get(wood, 400);
        let trim = rgb(0xA8754E);
        wb.cuboid(v3(x0, 0.0, z0 - 0.02), v3(x1, 0.95, z0 + 0.03), trim, 1.0, false);
        wb.cuboid(v3(x0 - 0.03, 0.0, z0), v3(x0 + 0.03, 0.95, z1), trim, 1.0, false);
        wb.cuboid(v3(x1 - 0.03, 0.0, z0), v3(x1 + 0.03, 0.95, z1), trim, 1.0, false);
        let beam = rgb(0x6B4430);
        wb.cuboid(v3(x0, h - 0.18, z0), v3(x1, h, z0 + 0.12), beam, 1.0, true);
        wb.cuboid(v3(x0, h - 0.18, z0), v3(x0 + 0.12, h, z1), beam, 1.0, true);
        wb.cuboid(v3(x1 - 0.12, h - 0.18, z0), v3(x1, h, z1), beam, 1.0, true);
        for k in 0..4 {
            let z = z0 + 2.0 + k as f32 * 2.4;
            wb.cuboid(v3(x0, 0.0, z - 0.08), v3(x0 + 0.1, h, z + 0.08), beam, 1.0, false);
            wb.cuboid(v3(x1 - 0.1, 0.0, z - 0.08), v3(x1, h, z + 0.08), beam, 1.0, false);
        }
        // Windows of warm evening light on the side walls.
        let g = glow.get(PLAIN, 64);
        for k in 0..3 {
            let z = z0 + 3.2 + k as f32 * 2.4;
            let (za, zb) = (z - 0.75, z + 0.75);
            g.quad([v3(x0 + 0.04, 1.3, zb), v3(x0 + 0.04, 1.3, za), v3(x0 + 0.04, 2.4, za), v3(x0 + 0.04, 2.4, zb)], [(0.0, 0.0); 4], rgb(0xFFD69A));
            g.quad([v3(x1 - 0.04, 1.3, za), v3(x1 - 0.04, 1.3, zb), v3(x1 - 0.04, 2.4, zb), v3(x1 - 0.04, 2.4, za)], [(0.0, 0.0); 4], rgb(0xFFD69A));
            let fb = st.get(wood, 200);
            for (x, inward) in [(x0 + 0.05, 1.0), (x1 - 0.05, -1.0)] {
                let xa = x.min(x + 0.05 * inward);
                let xb = x.max(x + 0.05 * inward);
                fb.cuboid(v3(xa, 1.25, za - 0.05), v3(xb, 2.45, za), beam, 1.0, false);
                fb.cuboid(v3(xa, 1.25, zb), v3(xb, 2.45, zb + 0.05), beam, 1.0, false);
                fb.cuboid(v3(xa, 1.82, za), v3(xb, 1.88, zb), beam, 1.0, false);
                fb.cuboid(v3(xa, 1.25, za), v3(xb, 1.3, zb), beam, 1.0, true);
            }
        }
        // Shelves, pictures and lamps along the walls.
        put(st, a, "pictureframe_large_A", v3(x0 + 0.06, 1.85, z0 + 2.0), FRAC_PI_2, 0.8, WHITE);
        put(st, a, "pictureframe_medium", v3(x1 - 0.06, 1.85, z0 + 2.0), -FRAC_PI_2, 0.8, WHITE);
        put(st, a, "lamp_standing", v3(x0 + 0.35, 0.0, z1 - 0.35), 0.0, 0.7, WHITE);
        put(st, a, "lamp_standing", v3(x1 - 0.35, 0.0, z1 - 0.35), 0.0, 0.7, WHITE);
    }

    fn deck(st: &mut Batch, a: &Assets, _l: &Layout) {
        let (x0, z0, x1, z1) = (WALK.0 / 100.0 - 0.3, ROOM.1 / 100.0, WALK.2 / 100.0 + 0.3, WALK.3 / 100.0 + 0.3);
        let wood = a.tex("counter");
        let b = st.get(wood, 4000);
        let hull = rgb(0x7A4E33);
        // The hull's sides below the deck, and a rail all round.
        b.cuboid(v3(x0 - 0.1, -0.9, z0), v3(x0, 0.0, z1 + 0.1), hull, 1.0, false);
        b.cuboid(v3(x1, -0.9, z0), v3(x1 + 0.1, 0.0, z1 + 0.1), hull, 1.0, false);
        b.cuboid(v3(x0 - 0.1, -0.9, z1), v3(x1 + 0.1, 0.0, z1 + 0.1), hull, 1.0, false);
        let rail = rgb(0xE9DCC6);
        let mut posts = |xa: f32, za: f32, xb: f32, zb: f32| {
            let n = (((xb - xa).abs() + (zb - za).abs()) / 1.0).ceil() as i32;
            for k in 0..=n {
                let t = k as f32 / n as f32;
                let (x, z) = (xa + (xb - xa) * t, za + (zb - za) * t);
                b.cuboid(v3(x - 0.035, 0.0, z - 0.035), v3(x + 0.035, 0.7, z + 0.035), rail, 1.0, false);
            }
            let (lx, hx) = (xa.min(xb) - 0.04, xa.max(xb) + 0.04);
            let (lz, hz) = (za.min(zb) - 0.04, za.max(zb) + 0.04);
            b.cuboid(v3(lx, 0.66, lz), v3(hx, 0.74, hz), rail, 1.0, true);
            b.cuboid(v3(lx, 0.33, lz), v3(hx, 0.37, hz), rail, 1.0, true);
        };
        posts(x0, z0 + 1.2, x0, z1);
        posts(x1, z0 + 1.2, x1, z1);
        posts(x0, z1, x1, z1);
        // The galley's back wall with a lifebuoy-red stripe.
        let plaster = a.tex("plaster");
        let pb = st.get(plaster, 64);
        pb.cuboid(v3(ROOM.0 / 100.0, 0.0, z0 - 0.1), v3(ROOM.2 / 100.0, 2.6, z0), rgb(0xF3EBDD), 2.0, true);
        pb.cuboid(v3(ROOM.0 / 100.0, 2.3, z0), v3(ROOM.2 / 100.0, 2.45, z0 + 0.02), rgb(0xD24B3A), 2.0, false);
    }

    fn kitchen(st: &mut Batch, a: &Assets, l: &Layout) {
        // Counters along the kitchen wall, their backs to it.
        let s = 0.72;
        let z = 0.02 + 1.0 * s;
        let row = [
            "fridge_A_decorated",
            "kitchencounter_straight_decorated",
            "stove_multi_decorated",
            "kitchencounter_sink_backsplash",
            "kitchencounter_straight_A_backsplash",
            "kitchencounter_straight_decorated",
            "stove_multi_decorated",
            "kitchencounter_straight_A_backsplash",
            "fridge_A_decorated",
        ];
        let x0 = ROOM.0 / 100.0 + 0.85;
        let step = (ROOM.2 - ROOM.0) / 100.0 / row.len() as f32 * 0.985;
        for (i, name) in row.iter().enumerate() {
            let x = x0 + i as f32 * step;
            put(st, a, name, v3(x, 0.0, z), 0.0, s, WHITE);
            if name.starts_with("stove") {
                put(st, a, "extractorhood", v3(x, 0.0, 0.02), 0.0, s, WHITE);
            } else if !name.starts_with("fridge") && !l.deck {
                put(st, a, "kitchencabinet", v3(x, 0.0, 0.02), 0.0, s, WHITE);
            }
        }
    }

    fn counter(st: &mut Batch, a: &Assets, b: &crate::level::Belt, wood: usize) {
        let s = stadium(b);
        let f = b.floor / 100.0;
        let top = b.top() / 100.0;
        let (bw, rim) = (BELT_W / 100.0, RIM / 100.0);
        let body = rgb(0xC08A5E);
        let cb = st.get(wood, 3000);
        s.wall(cb, rim, f, top, body, true, 0.8);
        s.wall(cb, rim + 0.02, f, f + 0.08, rgb(0x5B3A29), true, 0.8);
        s.band(cb, bw + 0.02, rim, top, rgb(0xE2B98E), 0.8);
        s.wall(cb, rim, top - 0.04, top + 0.01, rgb(0x7A4B31), true, 0.8);
        // The chef's well inside the belt: the counter's inner face down to a
        // low work shelf, the shelf, and the floor the chef stands on.
        let lip = -bw - 0.02;
        let shelf = lip - 0.14;
        let work = top - 0.16;
        s.wall(cb, lip, top - 0.01, top + 0.03, rgb(0xB07C52), true, 0.8);
        s.wall(cb, lip, work, top + 0.03, rgb(0xC08A5E), false, 0.8);
        s.band(cb, shelf, lip, work, rgb(0xEFDDBF), 0.8);
        s.wall(cb, shelf, f, work, rgb(0xA8754E), false, 0.8);
        s.cap(cb, shelf, f + 0.012, rgb(0x7E5D45), 0.6);
        let steel = a.made.steel;
        let sb = st.get(steel, 3000);
        s.band(sb, bw, bw + 0.025, top + 0.012, WHITE, 0.5);
        s.band(sb, -bw - 0.025, -bw, top + 0.012, WHITE, 0.5);
        s.wall(sb, bw + 0.025, top - 0.01, top + 0.012, WHITE, false, 0.5);
        s.wall(sb, -bw - 0.025, top - 0.01, top + 0.012, WHITE, true, 0.5);
        // On the shelf: chopping boards, sushi ready to go, bowls and soy.
        let sz = s.r + (shelf + lip) / 2.0;
        let n = ((s.half * 2.0) / 0.45).floor().max(1.0) as i32;
        for k in 0..=n {
            let x = s.cx - s.half + k as f32 * (s.half * 2.0) / n as f32;
            for (side, z) in [(0, s.cz - sz), (1, s.cz + sz)] {
                let at = v3(x, work, z);
                match (k + side * 2) % 4 {
                    0 => st.get(wood, 24).cuboid(at + v3(-0.11, 0.0, -0.05), at + v3(0.11, 0.022, 0.05), rgb(0xF0D2A2), 0.4, true),
                    1 => put(st, a, DISHES[(k + side) as usize % EVERYDAY].model, at, FRAC_PI_2, 0.22, WHITE),
                    2 => put(st, a, "bowl_small", at, 0.0, 0.2, WHITE),
                    _ => put(st, a, "soy", at, k as f32, 1.0, WHITE),
                }
            }
        }
        // Rice tubs at both ends of the well.
        let tub = (s.r + shelf - 0.06).clamp(0.1, 0.22);
        for dx in [-s.half, s.half] {
            st.get(PLAIN, 200).cylinder(v3(s.cx + dx, f, s.cz), tub, work - f - 0.06, 18, rgb(0x9C6B45), rgb(0xFBF7EE));
        }
    }

    /// The camera for a room, leaning a little toward `focus`.
    fn camera_for(_l: &Layout, focus: V2) -> Camera {
        let fx = (focus.x / 100.0 - 7.5) * 0.18;
        let fz = (focus.y / 100.0 - 5.6) * 0.1;
        Camera { eye: v3(7.5 + fx, 9.6, 13.9 + fz), target: v3(7.5 + fx, 0.0, 5.25 + fz), fovy: 42f32.to_radians() }
    }

    /// Moves the camera and the deck for this frame.
    pub fn follow(&mut self, r: &Round, dt: f32) {
        let want = Self::camera_for(&r.layout, r.player.pos);
        let k = 1.0 - (-dt * 3.0).exp();
        self.cam.eye = self.cam.eye.lerp(want.eye, k);
        self.cam.target = self.cam.target.lerp(want.target, k);
        let deg = r.layout.tilt.map_or(0.0, |t| t.degrees);
        self.lean = -r.lean * deg.to_radians();
        let c = v3(7.5, 0.0, 5.0);
        self.root = M4::translate(-c).then(&M4::roll(self.lean)).then(&M4::translate(c));
    }

    /// Where a world point lands on the 640×480 canvas, and whether in front.
    pub fn project(&self, p: V3) -> Option<(f32, f32)> {
        let q = self.root.point(p);
        let c = self.cam.view_proj(4.0 / 3.0).project(q);
        if c[3] <= 0.01 {
            return None;
        }
        Some(((c[0] / c[3] + 1.0) * 320.0, (1.0 - c[1] / c[3]) * 240.0))
    }

    pub fn draw_statics(&self, gl: &Gl) {
        let a = assets::get();
        let tex = |t: usize| if t == PLAIN { None } else { Some(a.texture(t)) };
        for &(m, t) in &self.statics {
            gl.draw(m, tex(t), &self.root, &Look::default());
        }
        let dim = mix_color(rgb(0xFFFFFF), rgb(0x2A3A6A), self.night);
        for &(m, t) in &self.glow {
            gl.draw(m, tex(t), &self.root, &Look { color: dim, emissive: 0.9, ..Look::default() });
        }
    }

    /// Draws the room and everything in it.
    pub fn draw(&self, gl: &Gl, r: &Round, hero: &Hero, chef: Option<&Hero>, t: f32) {
        let a = assets::get();
        let mut light = self.lighting;
        if r.layout.dark > 0.0 {
            let p = w(r.player.pos, r.layout.floor_at(r.player.pos) + 150.0);
            light.add_light(self.root.point(p), 2.6, rgb(0xFFC27A));
        }
        gl.light(&light, self.cam.eye);
        self.draw_statics(gl);
        render::blend_alpha(|| {
            for &m in &self.shade {
                gl.draw(m, None, &self.root, &Look::default());
            }
            for o in &r.layout.obstacles {
                if let Shape::Circle { x, y, r: rad } = o.shape {
                    gl.shadow(self.root.point(w(V2::new(x, y), 0.0)), rad / 100.0 * 1.5, rad / 100.0 * 1.3, 0.7);
                }
            }
        });
        // Belts: the slats move along under a scrolling texture.
        for (i, &(m, dir)) in self.belts.iter().enumerate() {
            let shift = -r.belt_travel[i] / 100.0 / 0.25 * dir;
            gl.draw(m, Some(a.texture(a.made.slats)), &self.root, &Look { uv_shift: (shift, 0.0), ..Look::default() });
        }
        for &m in &self.water {
            let look = Look { uv_shift: (t * 0.04, t * 0.025), emissive: 0.15, ..Look::default() };
            gl.draw(m, Some(a.texture(a.made.water)), &self.root, &look);
        }
        // Doors: two paper screens meeting in the middle when shut.
        for d in &r.layout.doors {
            let b = &r.layout.belts[d.belt];
            let k = d.closed(r.t);
            let y = d.y(b) / 100.0;
            let (x0, x1) = (d.x0 / 100.0, d.x1 / 100.0);
            let half = (x1 - x0) / 2.0;
            for (i, rest) in [(0, x0 - half), (1, x1)] {
                let shut = if i == 0 { x0 } else { x0 + half };
                let x = rest + (shut - rest) * k;
                let m = M4::scale(half, 1.25, 1.0).then(&M4::translate(v3(x, 0.05, y))).then(&self.root);
                gl.draw(self.door_panel, Some(a.texture(a.made.paper)), &m, &Look { tint: (rgb(0xFFF4DE), 0.15), ..Look::default() });
            }
        }
        // Plates.
        let reach = r.reachable();
        for (i, p) in r.plates.iter().enumerate() {
            let b = &r.layout.belts[p.belt];
            let (at, n) = b.point(p.s);
            let tangent = V2::new(-n.y, n.x) * b.dir;
            let yaw = tangent.x.atan2(tangent.y);
            let mut h = b.top() + 0.6;
            let mut s = 1.0;
            if p.age < 0.35 {
                let k = p.age / 0.35;
                h += (1.0 - k) * 45.0;
                s = 0.7 + 0.3 * k;
            }
            if p.leaving > 0.0 {
                s *= 1.0 - p.leaving;
                h -= p.leaving * 10.0;
            }
            let seen = r.visible(p);
            self.plate(gl, w(at, h), yaw, s, p.dish, seen, t, Some(i) == reach);
        }
        // The visitor.
        let floor = r.layout.floor_at(r.player.pos);
        let feet = w(r.player.pos, floor);
        let hd = r.hero_def();
        let placement = hero.place(feet, r.player.yaw, hd.height);
        let m = placement.then(&self.root);
        let red = r.danger.is_some();
        let pulse = (t * 6.0).sin() * 0.5 + 0.5;
        let look = Look {
            tint: if red { (rgb(0xFF2A2A), 0.18 + 0.22 * pulse) } else if r.fullness > 100.0 { (rgb(0xFF7A6A), 0.12) } else { (rgb(0), 0.0) },
            rim: 0.6,
            ..Look::default()
        };
        render::blend_alpha(|| gl.shadow(self.root.point(feet), 0.42, 0.32, 0.9));
        hero.draw(gl, &m, &look, Some(1.6));
        // A chef works inside each belt and reaches out as a plate goes on.
        if let Some(c) = chef {
            for b in &r.layout.belts {
                let feet = w(V2::new(b.cx, b.cy), b.floor);
                let cm = c.place(feet, 0.0, 1.62).then(&self.root);
                c.draw(gl, &cm, &Look { rim: 0.4, ..Look::default() }, None);
                let hat = M4::translate(feet + v3(0.0, 1.54, 0.0)).then(&self.root);
                gl.draw(self.hat, None, &hat, &Look::default());
            }
        }
        if let Some(e) = &r.player.eating {
            if e.t < crate::rules::RULES.bite_s {
                let hand = placement.point(hero.hand_pos);
                let from = w(e.from, e.from_h + 1.0);
                let k = (e.t / 0.2).min(1.0);
                let p = from.lerp(hand, k);
                food_at(gl, &self.root, p, r.player.yaw, 0.9, e.dish, true, t, false);
            }
        }
        // Lanterns glow in the dark.
        if r.layout.dark > 0.0 {
            let view = self.cam.view();
            let flicker = 0.9 + 0.1 * (t * 9.0).sin() * (t * 5.3).cos();
            for lamp in &r.layout.lamps {
                let p = self.root.point(w(V2::new(lamp.x, lamp.y), 182.0));
                let a = (200.0 * flicker * r.layout.dark.max(0.5)) as u8;
                gl.halo(&view, p, 1.3, Color { r: 255, g: 176, b: 96, a });
                gl.halo(&view, p, 0.45, Color { r: 255, g: 236, b: 200, a });
            }
        }
        // What the mage sees coming.
        if hd.foresight {
            for (bi, b) in r.layout.belts.iter().enumerate() {
                let (at, _) = b.point(b.serve_s());
                let p = self.root.point(w(at, b.top() + 105.0 + (t * 2.0).sin() * 4.0));
                food_at(gl, &M4::ID, p, t, 0.85, r.next_dish[bi], true, t, true);
            }
        }
    }

    /// A plate with its sushi.
    #[allow(clippy::too_many_arguments)]
    pub fn plate(&self, gl: &Gl, at: V3, yaw: f32, s: f32, dish: usize, seen: bool, t: f32, near: bool) {
        plate_at(gl, &self.root, at, yaw, s, Some(dish), DISHES[dish].hue, seen, t);
        if near {
            let pulse = 0.5 + 0.5 * (t * 7.0).sin();
            let rr = 0.27 + 0.02 * pulse;
            let m = M4::scale(rr, 1.0, rr).then(&M4::translate(at + v3(0.0, 0.006, 0.0))).then(&self.root);
            render::blend_alpha(|| {
                gl.draw(self.ring, None, &m, &Look { color: rgb(0xFFD54A), emissive: 1.4, ..Look::default() });
            });
        }
    }
}

/// A plate glazed in `hue`, with `dish` on it if any, placed under `root`.
#[allow(clippy::too_many_arguments)]
pub fn plate_at(gl: &Gl, root: &M4, at: V3, yaw: f32, s: f32, dish: Option<usize>, hue: Hue, seen: bool, t: f32) {
    let a = assets::get();
    let golden = hue == Hue::Gold;
    let plate = a.prop("plate");
    let ps = 0.42 / plate.size().x.max(0.01) * s;
    let m = M4::scale(ps, ps, ps).then(&M4::yaw(yaw)).then(&M4::translate(at)).then(root);
    let dark = if seen { (rgb(0), 0.0) } else { (rgb(0x0C0B14), 0.82) };
    let look = Look {
        color: rgb(hue.plate()),
        tint: dark,
        emissive: if golden { 0.35 + 0.25 * (t * 5.0).sin() } else { 0.0 },
        rim: if golden { 1.0 } else { 0.2 },
        ..Look::default()
    };
    for &(mesh, tex) in &plate.meshes {
        gl.draw(mesh, tex.map(|i| a.texture(i)), &m, &look);
    }
    if let Some(d) = dish {
        let top = at + v3(0.0, plate.size().y * ps, 0.0);
        food_at(gl, root, top, yaw, s, d, seen, t, false);
    }
}

/// A piece of sushi standing on `at`, placed under `root`.
#[allow(clippy::too_many_arguments)]
pub fn food_at(gl: &Gl, root: &M4, at: V3, yaw: f32, s: f32, dish: usize, seen: bool, t: f32, ghost: bool) {
    let a = assets::get();
    let prop = a.prop(DISHES[dish].model);
    let size = prop.size();
    let k = 0.31 / size.x.max(size.z).max(0.01) * s;
    let m = M4::translate(v3(-(prop.lo.x + prop.hi.x) / 2.0, -prop.lo.y, -(prop.lo.z + prop.hi.z) / 2.0))
        .then(&M4::scale(k, k, k))
        .then(&M4::yaw(yaw + FRAC_PI_2))
        .then(&M4::translate(at))
        .then(root);
    let look = Look {
        tint: if !seen { (rgb(0x0C0B14), 0.85) } else if ghost { (rgb(0xE8F0FF), 0.35) } else { (rgb(0), 0.0) },
        emissive: if ghost { 0.4 + 0.2 * (t * 3.0).sin() } else if dish == GOLDEN { 0.15 } else { 0.0 },
        rim: 0.5,
        ..Look::default()
    };
    for &(mesh, tex) in &prop.meshes {
        gl.draw(mesh, tex.map(|i| a.texture(i)), &m, &look);
    }
}

/// Draws a model from the pack placed by `m`.
pub fn prop_at(gl: &Gl, name: &str, m: &M4, look: &Look) {
    let a = assets::get();
    for &(mesh, tex) in &a.prop(name).meshes {
        gl.draw(mesh, tex.map(|i| a.texture(i)), m, look);
    }
}

fn mix_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color { r: l(a.r, b.r), g: l(a.g, b.g), b: l(a.b, b.b), a: 255 }
}

/// What the visitor's body is doing.
pub fn act_of(r: &Round) -> Act {
    use crate::game::Outcome;
    if r.player.collapsed > 0.0 {
        return if r.player.collapsed > 0.8 { Act::Down } else { Act::Rise };
    }
    match r.outcome {
        Some(Outcome::Hospital) => return Act::Down,
        Some(Outcome::Success) if r.player.eating.is_none() => return Act::Cheer,
        Some(_) if r.player.eating.is_none() => return Act::Hurt,
        _ => {}
    }
    if let Some(e) = &r.player.eating {
        return Act::Eat((e.t / crate::rules::RULES.eat_s).min(1.0));
    }
    if r.player.speed > 20.0 {
        Act::Walk(r.player.speed / 100.0)
    } else {
        Act::Idle
    }
}

/// How round the visitor looks, from the stomach.
pub fn belly_of(r: &Round) -> f32 {
    ((r.fullness - 15.0) / 105.0).clamp(0.0, 1.2)
}

/// A level's name, for logs.
#[allow(dead_code)]
pub fn name(level: usize) -> &'static str {
    LEVELS[level].id
}

impl Drop for Scene {
    fn drop(&mut self) {
        unsafe {
            for &(m, _) in self.statics.iter().chain(self.glow.iter()) {
                ffi::UnloadMesh(m);
            }
            for &(m, _) in &self.belts {
                ffi::UnloadMesh(m);
            }
            for &m in self.water.iter().chain(self.shade.iter()) {
                ffi::UnloadMesh(m);
            }
            ffi::UnloadMesh(self.door_panel);
            ffi::UnloadMesh(self.ring);
            ffi::UnloadMesh(self.hat);
        }
    }
}
