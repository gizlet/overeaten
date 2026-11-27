//! The floor plans of the eight rooms of Carousel Sushi, in centimetres on
//! the floor: x runs west to east, y north (the kitchen wall) to south (the
//! camera). Belts, the counters they sit in, pillars, stools, raised tatami,
//! sliding doors, a pond with its bridge, lanterns and a tilting deck. No
//! drawing here: `scene.rs` builds the 3D room from it and `game.rs` plays
//! on it.

use crate::game::V2;
use std::f32::consts::PI;

/// From a belt's middle line to its counter's outer edge.
pub const RIM: f32 = 42.0;
/// Half a belt's width.
pub const BELT_W: f32 = 25.0;
/// Half the width of the visitor's feet.
pub const BODY: f32 = 28.0;
/// The counter's height above the floor it stands on.
pub const COUNTER_H: f32 = 70.0;

/// The room's walls and the floor the visitor may walk.
pub const ROOM: (f32, f32, f32, f32) = (100.0, 0.0, 1400.0, 1000.0);
pub const WALK: (f32, f32, f32, f32) = (140.0, 120.0, 1360.0, 965.0);

#[derive(Clone, Copy, Debug)]
pub struct Belt {
    pub cx: f32,
    pub cy: f32,
    /// Half the length of the straights.
    pub half: f32,
    /// Radius of the middle line at the ends.
    pub r: f32,
    /// 1 clockwise seen from above, -1 the other way.
    pub dir: f32,
    /// The floor the counter stands on.
    pub floor: f32,
    /// How often each everyday dish comes out here.
    pub weights: [u32; 6],
}

impl Belt {
    pub fn perimeter(&self) -> f32 {
        4.0 * self.half + 2.0 * PI * self.r
    }

    pub fn outer(&self) -> f32 {
        self.r + RIM
    }

    /// The top of the belt.
    pub fn top(&self) -> f32 {
        self.floor + COUNTER_H
    }

    /// The point at travel `s` and the outward normal there. Travel starts
    /// at the far straight's west end and goes the belt's way round.
    pub fn point(&self, s: f32) -> (V2, V2) {
        let p = self.perimeter();
        let s = if self.dir > 0.0 { s.rem_euclid(p) } else { (2.0 * self.half - s).rem_euclid(p) };
        let st = 2.0 * self.half;
        let arc = PI * self.r;
        let (cx, cy, h, r) = (self.cx, self.cy, self.half, self.r);
        if s < st {
            (V2::new(cx - h + s, cy - r), V2::new(0.0, -1.0))
        } else if s < st + arc {
            let a = -PI / 2.0 + (s - st) / r;
            let n = V2::new(a.cos(), a.sin());
            (V2::new(cx + h, cy) + n * r, n)
        } else if s < 2.0 * st + arc {
            (V2::new(cx + h - (s - st - arc), cy + r), V2::new(0.0, 1.0))
        } else {
            let a = PI / 2.0 + (s - 2.0 * st - arc) / r;
            let n = V2::new(a.cos(), a.sin());
            (V2::new(cx - h, cy) + n * r, n)
        }
    }

    /// Where the kitchen puts plates on: the middle of the far straight.
    pub fn serve_s(&self) -> f32 {
        self.half
    }

    /// Wraps travel into one lap.
    pub fn wrap(&self, s: f32) -> f32 {
        s.rem_euclid(self.perimeter())
    }

    /// Signed travel from `a` forward to `b`, within half a lap.
    pub fn gap(&self, a: f32, b: f32) -> f32 {
        let p = self.perimeter();
        let d = (b - a).rem_euclid(p);
        if d > p / 2.0 {
            d - p
        } else {
            d
        }
    }

    /// Pushes `p` out of the counter, a capsule round the straights.
    fn push_out(&self, p: V2, pad: f32) -> V2 {
        let spine = V2::new(p.x.clamp(self.cx - self.half, self.cx + self.half), self.cy);
        let d = p - spine;
        let dist = d.len();
        let min = self.outer() + pad;
        if dist >= min {
            return p;
        }
        let n = if dist < 1e-3 { V2::new(0.0, 1.0) } else { d * (1.0 / dist) };
        spine + n * min
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Prop {
    Pillar,
    Stool,
    LowTable,
    Lantern,
    Plant,
    Pond,
    Crates,
}

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Circle { x: f32, y: f32, r: f32 },
    Rect { x: f32, y: f32, w: f32, h: f32 },
}

#[derive(Clone, Copy, Debug)]
pub struct Obstacle {
    pub shape: Shape,
    pub prop: Prop,
}

/// A raised floor; its edges are steps that slow the visitor down.
#[derive(Clone, Copy, Debug)]
pub struct Platform {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub height: f32,
}

/// A sliding screen beside a stretch of belt that shuts it off for a while.
#[derive(Clone, Copy, Debug)]
pub struct Door {
    pub belt: usize,
    pub x0: f32,
    pub x1: f32,
    /// On the kitchen side of the belt, or the room side.
    pub north: bool,
    pub period: f32,
    /// Seconds of each period it is shut, after `phase`.
    pub shut: f32,
    pub phase: f32,
}

impl Door {
    /// How far shut, 0 open to 1 closed, sliding over 0.6 s.
    pub fn closed(&self, t: f32) -> f32 {
        let k = (t - self.phase).rem_euclid(self.period);
        let slide = 0.6;
        if k < slide {
            k / slide
        } else if k < self.shut {
            1.0
        } else if k < self.shut + slide {
            1.0 - (k - self.shut) / slide
        } else {
            0.0
        }
    }

    /// Its line on the floor: y of the screen's track.
    pub fn y(&self, b: &Belt) -> f32 {
        if self.north {
            b.cy - b.outer() - 12.0
        } else {
            b.cy + b.outer() + 12.0
        }
    }
}

/// A light in a dark room: where it stands, how high and how far it reaches.
#[derive(Clone, Copy, Debug)]
pub struct Lamp {
    pub x: f32,
    pub y: f32,
    pub h: f32,
    pub reach: f32,
}

/// The deck of the ferry rolls to one side and back.
#[derive(Clone, Copy, Debug)]
pub struct Tilt {
    pub period: f32,
    /// Seconds of each period it leans, starting at `period - hold`.
    pub hold: f32,
    pub degrees: f32,
    /// How fast the visitor slides while it leans, cm/s.
    pub slide: f32,
}

impl Tilt {
    /// -1..1: how far it leans and which way (positive is down to the east).
    pub fn lean(&self, t: f32) -> f32 {
        let n = (t / self.period).floor();
        let k = t - n * self.period - (self.period - self.hold);
        if k < 0.0 {
            return 0.0;
        }
        let side = if (n as i64) % 2 == 0 { 1.0 } else { -1.0 };
        side * (k / self.hold * PI).sin()
    }
}

#[derive(Clone, Debug)]
pub struct Layout {
    pub belts: Vec<Belt>,
    pub obstacles: Vec<Obstacle>,
    pub platforms: Vec<Platform>,
    pub doors: Vec<Door>,
    pub lamps: Vec<Lamp>,
    pub tilt: Option<Tilt>,
    /// 0 for daylight, 1 for night.
    pub dark: f32,
    /// The bridge over the pond, as (x, y, w, h).
    pub bridge: Option<(f32, f32, f32, f32)>,
    /// A ferry deck on the water instead of a room.
    pub deck: bool,
    pub start: V2,
}

fn circle(x: f32, y: f32, r: f32, prop: Prop) -> Obstacle {
    Obstacle { shape: Shape::Circle { x, y, r }, prop }
}

fn rect(x: f32, y: f32, w: f32, h: f32, prop: Prop) -> Obstacle {
    Obstacle { shape: Shape::Rect { x, y, w, h }, prop }
}

const EVEN: [u32; 6] = [3, 3, 3, 3, 3, 2];

fn belt(cx: f32, cy: f32, half: f32, r: f32, dir: f32) -> Belt {
    Belt { cx, cy, half, r, dir, floor: 0.0, weights: EVEN }
}

impl Layout {
    fn base(belts: Vec<Belt>) -> Layout {
        Layout {
            belts,
            obstacles: Vec::new(),
            platforms: Vec::new(),
            doors: Vec::new(),
            lamps: Vec::new(),
            tilt: None,
            dark: 0.0,
            bridge: None,
            deck: false,
            start: V2::new(750.0, 840.0),
        }
    }

    /// The floor plan of level `i` (0-based).
    pub fn of(i: usize) -> Layout {
        match i {
            0 => {
                let mut l = Layout::base(vec![belt(750.0, 520.0, 300.0, 86.0, 1.0)]);
                l.obstacles.push(circle(205.0, 900.0, 40.0, Prop::Plant));
                l.obstacles.push(circle(1295.0, 900.0, 40.0, Prop::Plant));
                l
            }
            1 => {
                let mut l = Layout::base(vec![belt(750.0, 520.0, 250.0, 86.0, 1.0)]);
                for (x, y) in [(300.0, 330.0), (1200.0, 330.0), (300.0, 730.0), (1200.0, 730.0), (560.0, 840.0), (940.0, 840.0)]
                {
                    l.obstacles.push(circle(x, y, 30.0, Prop::Pillar));
                }
                let sy = 520.0 + 86.0 + RIM + 24.0;
                for x in [580.0, 650.0, 850.0, 920.0] {
                    l.obstacles.push(circle(x, sy, 21.0, Prop::Stool));
                }
                for x in [620.0, 880.0] {
                    l.obstacles.push(circle(x, 520.0 - 86.0 - RIM - 24.0, 21.0, Prop::Stool));
                }
                l.start = V2::new(750.0, 860.0);
                l
            }
            2 => {
                let mut l = Layout::base(vec![belt(390.0, 560.0, 110.0, 80.0, 1.0), belt(1110.0, 560.0, 110.0, 80.0, -1.0)]);
                l.obstacles.push(rect(690.0, 120.0, 120.0, 405.0, Prop::Pond));
                l.obstacles.push(rect(690.0, 615.0, 120.0, 400.0, Prop::Pond));
                l.bridge = Some((680.0, 525.0, 140.0, 90.0));
                l.start = V2::new(390.0, 860.0);
                l
            }
            3 => {
                let mut upper = belt(750.0, 262.0, 280.0, 70.0, 1.0);
                upper.floor = 40.0;
                upper.weights = [4, 4, 2, 2, 1, 4];
                let mut lower = belt(750.0, 730.0, 280.0, 70.0, -1.0);
                lower.weights = [2, 2, 3, 3, 4, 1];
                let mut l = Layout::base(vec![upper, lower]);
                l.platforms.push(Platform { x: 140.0, y: 120.0, w: 1220.0, h: 315.0, height: 40.0 });
                l.obstacles.push(rect(165.0, 150.0, 120.0, 90.0, Prop::LowTable));
                l.obstacles.push(rect(1215.0, 150.0, 120.0, 90.0, Prop::LowTable));
                l.obstacles.push(circle(205.0, 905.0, 40.0, Prop::Plant));
                l.obstacles.push(circle(1295.0, 905.0, 40.0, Prop::Plant));
                l.start = V2::new(750.0, 905.0);
                l
            }
            4 => {
                let mut l = Layout::base(vec![belt(750.0, 520.0, 300.0, 86.0, 1.0)]);
                l.doors.push(Door { belt: 0, x0: 820.0, x1: 1020.0, north: true, period: 9.0, shut: 4.0, phase: 1.0 });
                l.doors.push(Door { belt: 0, x0: 470.0, x1: 670.0, north: false, period: 9.0, shut: 4.0, phase: 5.5 });
                l.obstacles.push(rect(160.0, 140.0, 110.0, 80.0, Prop::Crates));
                l.obstacles.push(circle(1295.0, 900.0, 40.0, Prop::Plant));
                l
            }
            5 => {
                let mut l = Layout::base(vec![belt(750.0, 520.0, 300.0, 86.0, 1.0)]);
                l.dark = 1.0;
                for (x, y) in [(250.0, 300.0), (1250.0, 300.0), (250.0, 820.0), (1250.0, 820.0), (750.0, 900.0)] {
                    l.lamps.push(Lamp { x, y, h: 170.0, reach: 340.0 });
                    l.obstacles.push(circle(x, y, 22.0, Prop::Lantern));
                }
                l.start = V2::new(600.0, 840.0);
                l
            }
            6 => {
                let mut l = Layout::base(vec![belt(750.0, 520.0, 280.0, 86.0, 1.0)]);
                l.deck = true;
                l.tilt = Some(Tilt { period: 13.0, hold: 4.5, degrees: 6.0, slide: 150.0 });
                l
            }
            _ => {
                let mut l = Layout::base(vec![belt(390.0, 560.0, 110.0, 80.0, 1.0), belt(1110.0, 560.0, 110.0, 80.0, -1.0)]);
                l.obstacles.push(rect(690.0, 120.0, 120.0, 405.0, Prop::Pond));
                l.obstacles.push(rect(690.0, 615.0, 120.0, 400.0, Prop::Pond));
                l.bridge = Some((680.0, 525.0, 140.0, 90.0));
                for (x, y) in [(250.0, 330.0), (1250.0, 330.0), (250.0, 830.0), (1250.0, 830.0)] {
                    l.obstacles.push(circle(x, y, 30.0, Prop::Pillar));
                }
                l.doors.push(Door { belt: 1, x0: 1010.0, x1: 1190.0, north: false, period: 10.0, shut: 4.0, phase: 3.0 });
                l.dark = 0.6;
                for (x, y) in [(560.0, 300.0), (940.0, 300.0), (560.0, 860.0), (940.0, 860.0)] {
                    l.lamps.push(Lamp { x, y, h: 170.0, reach: 380.0 });
                    l.obstacles.push(circle(x, y, 22.0, Prop::Lantern));
                }
                l.start = V2::new(390.0, 860.0);
                l
            }
        }
    }

    /// The floor's height under `p`.
    pub fn floor_at(&self, p: V2) -> f32 {
        self.platforms
            .iter()
            .filter(|f| p.x >= f.x && p.x <= f.x + f.w && p.y >= f.y && p.y <= f.y + f.h)
            .map(|f| f.height)
            .fold(0.0, f32::max)
    }

    /// How fast the visitor walks at `p`, 1 normally: steps slow them down.
    pub fn pace_at(&self, p: V2) -> f32 {
        let near_edge = self.platforms.iter().any(|f| {
            let inside_x = p.x >= f.x - 25.0 && p.x <= f.x + f.w + 25.0;
            let inside_y = p.y >= f.y - 25.0 && p.y <= f.y + f.h + 25.0;
            let deep = p.x >= f.x + 25.0 && p.x <= f.x + f.w - 25.0 && p.y >= f.y + 25.0 && p.y <= f.y + f.h - 25.0;
            inside_x && inside_y && !deep
        });
        if near_edge {
            0.55
        } else if self.bridge.is_some_and(|(x, y, w, h)| p.x >= x && p.x <= x + w && p.y >= y && p.y <= y + h) {
            0.8
        } else {
            1.0
        }
    }

    /// Pushes a point on the floor out of everything solid at time `t`.
    pub fn collide(&self, p: V2, t: f32) -> V2 {
        let mut p = p;
        for _ in 0..2 {
            for b in &self.belts {
                p = b.push_out(p, BODY);
            }
            for o in &self.obstacles {
                p = match o.shape {
                    Shape::Circle { x, y, r } => {
                        let c = V2::new(x, y);
                        let d = p - c;
                        let min = r + BODY;
                        if d.len() < min {
                            c + if d.len() < 1e-3 { V2::new(0.0, 1.0) } else { d.norm() } * min
                        } else {
                            p
                        }
                    }
                    Shape::Rect { x, y, w, h } => out_of_box(p, x - BODY, y - BODY, w + 2.0 * BODY, h + 2.0 * BODY),
                };
            }
            for d in &self.doors {
                if d.closed(t) > 0.5 {
                    let b = &self.belts[d.belt];
                    let y = d.y(b);
                    p = out_of_box(p, d.x0 - BODY, y - 6.0 - BODY, d.x1 - d.x0 + 2.0 * BODY, 12.0 + 2.0 * BODY);
                }
            }
            p = V2::new(p.x.clamp(WALK.0, WALK.2), p.y.clamp(WALK.1, WALK.3));
        }
        p
    }

    pub fn walkable(&self, p: V2, t: f32) -> bool {
        self.collide(p, t).dist(p) < 0.5
    }

    /// Whether a plate at `at` on belt `b` is shut off by a door.
    pub fn behind_door(&self, b: usize, at: V2, t: f32) -> bool {
        self.doors.iter().any(|d| {
            d.belt == b
                && d.closed(t) > 0.3
                && at.x >= d.x0 - 10.0
                && at.x <= d.x1 + 10.0
                && (if d.north { at.y < self.belts[b].cy } else { at.y > self.belts[b].cy })
        })
    }

    /// How much light falls on `p` from the lamps and the visitor's own
    /// lantern at `me`: 1 in full light.
    pub fn light_at(&self, p: V2, me: V2) -> f32 {
        if self.dark <= 0.0 {
            return 1.0;
        }
        let own = (1.0 - p.dist(me) / 230.0).max(0.0);
        let lamps = self.lamps.iter().map(|l| (1.0 - p.dist(V2::new(l.x, l.y)) / l.reach).max(0.0)).fold(0.0, f32::max);
        let base = 1.0 - self.dark;
        (base + own.max(lamps) * 2.0).min(1.0)
    }
}

/// Pushes `p` out of a box through its nearest side.
fn out_of_box(p: V2, x: f32, y: f32, w: f32, h: f32) -> V2 {
    if p.x <= x || p.x >= x + w || p.y <= y || p.y >= y + h {
        return p;
    }
    let sides = [
        (p.x - x, V2::new(x, p.y)),
        (x + w - p.x, V2::new(x + w, p.y)),
        (p.y - y, V2::new(p.x, y)),
        (y + h - p.y, V2::new(p.x, y + h)),
    ];
    sides.iter().min_by(|a, b| a.0.total_cmp(&b.0)).map_or(p, |s| s.1)
}

// ---- Finding a way ---------------------------------------------------------

const CELL: f32 = 20.0;

/// A path over the floor from `from` to `to` avoiding everything solid at
/// time `t`, as waypoints (the last is `to` or the nearest reachable cell).
pub fn path(l: &Layout, from: V2, to: V2, t: f32) -> Vec<V2> {
    let (x0, y0) = (WALK.0, WALK.1);
    let nx = ((WALK.2 - x0) / CELL) as i32 + 1;
    let ny = ((WALK.3 - y0) / CELL) as i32 + 1;
    let cell = |p: V2| {
        (((p.x - x0) / CELL).round().clamp(0.0, (nx - 1) as f32) as i32, ((p.y - y0) / CELL).round().clamp(0.0, (ny - 1) as f32) as i32)
    };
    let centre = |c: (i32, i32)| V2::new(x0 + c.0 as f32 * CELL, y0 + c.1 as f32 * CELL);
    if straight(l, from, to, t) {
        return vec![to];
    }
    let idx = |c: (i32, i32)| (c.1 * nx + c.0) as usize;
    let n = (nx * ny) as usize;
    let mut open_ok = vec![None::<bool>; n];
    let mut ok = |c: (i32, i32)| -> bool {
        let i = idx(c);
        *open_ok[i].get_or_insert_with(|| l.walkable(centre(c), t))
    };
    let start = cell(from);
    let goal = cell(to);
    let mut g = vec![f32::MAX; n];
    let mut came = vec![usize::MAX; n];
    let mut heap = std::collections::BinaryHeap::new();
    #[derive(PartialEq)]
    struct Node(f32, usize);
    impl Eq for Node {}
    impl PartialOrd for Node {
        fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(o))
        }
    }
    impl Ord for Node {
        fn cmp(&self, o: &Self) -> std::cmp::Ordering {
            o.0.total_cmp(&self.0)
        }
    }
    let h = |c: (i32, i32)| (((c.0 - goal.0).pow(2) + (c.1 - goal.1).pow(2)) as f32).sqrt();
    g[idx(start)] = 0.0;
    heap.push(Node(h(start), idx(start)));
    let mut best = (h(start), idx(start));
    let mut found = None;
    while let Some(Node(_, i)) = heap.pop() {
        let c = ((i as i32) % nx, (i as i32) / nx);
        if c == goal {
            found = Some(i);
            break;
        }
        if h(c) < best.0 {
            best = (h(c), i);
        }
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
            let d = (c.0 + dx, c.1 + dy);
            if d.0 < 0 || d.1 < 0 || d.0 >= nx || d.1 >= ny || !ok(d) {
                continue;
            }
            if dx != 0 && dy != 0 && (!ok((c.0 + dx, c.1)) || !ok((c.0, c.1 + dy))) {
                continue;
            }
            let step = if dx != 0 && dy != 0 { 1.414 } else { 1.0 };
            let ng = g[i] + step;
            let j = idx(d);
            if ng < g[j] {
                g[j] = ng;
                came[j] = i;
                heap.push(Node(ng + h(d), j));
            }
        }
    }
    let end = found.unwrap_or(best.1);
    let mut cells = vec![end];
    while let Some(&last) = cells.last() {
        let prev = came[last];
        if prev == usize::MAX {
            break;
        }
        cells.push(prev);
    }
    cells.reverse();
    let mut pts: Vec<V2> = cells.iter().map(|&i| centre(((i as i32) % nx, (i as i32) / nx))).collect();
    if found.is_some() {
        if let Some(last) = pts.last_mut() {
            *last = to;
        }
    }
    // Skip waypoints that can be passed straight.
    let mut out = Vec::new();
    let mut at = from;
    let mut k = 0;
    while k < pts.len() {
        let mut j = pts.len() - 1;
        while j > k && !straight(l, at, pts[j], t) {
            j -= 1;
        }
        out.push(pts[j]);
        at = pts[j];
        k = j + 1;
    }
    out
}

fn straight(l: &Layout, a: V2, b: V2, t: f32) -> bool {
    let n = (a.dist(b) / 10.0).ceil().max(1.0) as usize;
    (0..=n).all(|i| l.walkable(a + (b - a) * (i as f32 / n as f32), t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn belts_are_continuous_both_ways() {
        for dir in [1.0, -1.0] {
            let b = Belt { dir, ..belt(500.0, 500.0, 200.0, 80.0, 1.0) };
            let p = b.perimeter();
            let mut prev = b.point(0.0).0;
            let mut s = 0.5;
            while s <= p {
                let q = b.point(s).0;
                assert!(q.dist(prev) < 0.6, "jump at {s}");
                prev = q;
                s += 0.5;
            }
            let (at, n) = b.point(b.serve_s());
            assert!((at.x - 500.0).abs() < 0.1 && n.y < -0.9, "serving from the kitchen side: {at:?}");
        }
    }

    #[test]
    fn every_room_starts_on_the_floor_and_reaches_every_belt() {
        for i in 0..8 {
            let l = Layout::of(i);
            assert!(l.walkable(l.start, 0.0), "level {i} starts inside something");
            for (k, b) in l.belts.iter().enumerate() {
                for s in [0.0, 0.25, 0.5, 0.75].map(|f| f * b.perimeter()) {
                    let (at, n) = b.point(s);
                    let spot = l.collide(at + n * (RIM + BODY + 6.0), 0.0);
                    let way = path(&l, l.start, spot, 0.0);
                    let end = *way.last().unwrap();
                    assert!(end.dist(spot) < 30.0, "level {i}: belt {k} at {s} out of reach ({end:?} vs {spot:?})");
                }
            }
        }
    }

    #[test]
    fn doors_shut_and_open() {
        let d = Door { belt: 0, x0: 0.0, x1: 1.0, north: true, period: 9.0, shut: 4.0, phase: 0.0 };
        assert_eq!(d.closed(2.0), 1.0);
        assert_eq!(d.closed(6.0), 0.0);
        assert!(d.closed(0.3) > 0.0 && d.closed(0.3) < 1.0);
    }
}
