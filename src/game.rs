//! A meal at the sushi carousel: the belt, the visitor, the stomach and the
//! clock. No drawing here, so the rules can be tested and played by the
//! autopilot.

use crate::rules::*;
use std::f32::consts::PI;
use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

impl V2 {
    pub const fn new(x: f32, y: f32) -> V2 {
        V2 { x, y }
    }
    pub fn len(self) -> f32 {
        self.x.hypot(self.y)
    }
    pub fn dist(self, o: V2) -> f32 {
        (self - o).len()
    }
    pub fn norm(self) -> V2 {
        let l = self.len();
        if l < 1e-5 {
            V2::default()
        } else {
            self * (1.0 / l)
        }
    }
    pub fn lerp(self, o: V2, t: f32) -> V2 {
        self + (o - self) * t
    }
}

impl Add for V2 {
    type Output = V2;
    fn add(self, o: V2) -> V2 {
        V2::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for V2 {
    type Output = V2;
    fn sub(self, o: V2) -> V2 {
        V2::new(self.x - o.x, self.y - o.y)
    }
}

impl Mul<f32> for V2 {
    type Output = V2;
    fn mul(self, k: f32) -> V2 {
        V2::new(self.x * k, self.y * k)
    }
}

/// The floor plan, in 640×480 screen pixels.
pub mod floor {
    /// Middle of the carousel island.
    pub const CX: f32 = 320.0;
    pub const CY: f32 = 284.0;
    /// Half the length of the island's straight sides.
    pub const HALF: f32 = 140.0;
    /// Radius of the belt's middle line around the straight spine.
    pub const R: f32 = 58.0;
    /// Half the belt's width.
    pub const BELT: f32 = 12.0;
    /// Counter lip outside the belt.
    pub const LIP: f32 = 10.0;
    pub const OUTER: f32 = R + BELT + LIP;
    /// Height of the counter's front face.
    pub const SIDE: f32 = 16.0;
    /// Where the visitor's feet may go.
    pub const MIN_X: f32 = 26.0;
    pub const MAX_X: f32 = 614.0;
    pub const MIN_Y: f32 = 160.0;
    pub const MAX_Y: f32 = 434.0;
    pub const BODY: f32 = 10.0;
    /// Where the chef puts new plates on the belt: the middle of the far side.
    pub const SERVE_S: f32 = HALF;
}

use floor::*;

pub fn perimeter() -> f32 {
    4.0 * HALF + 2.0 * PI * R
}

pub fn wrap(s: f32) -> f32 {
    s.rem_euclid(perimeter())
}

/// The belt runs clockwise: along the far side to the right, round the right
/// end, back along the near side and round the left end. Returns the point at
/// arc length `s` and the outward normal there.
pub fn belt_point(s: f32) -> (V2, V2) {
    let s = wrap(s);
    let straight = 2.0 * HALF;
    let arc = PI * R;
    if s < straight {
        (V2::new(CX - HALF + s, CY - R), V2::new(0.0, -1.0))
    } else if s < straight + arc {
        let a = -PI / 2.0 + (s - straight) / R;
        let n = V2::new(a.cos(), a.sin());
        (V2::new(CX + HALF, CY) + n * R, n)
    } else if s < 2.0 * straight + arc {
        (V2::new(CX + HALF - (s - straight - arc), CY + R), V2::new(0.0, 1.0))
    } else {
        let a = PI / 2.0 + (s - 2.0 * straight - arc) / R;
        let n = V2::new(a.cos(), a.sin());
        (V2::new(CX - HALF, CY) + n * R, n)
    }
}

/// Signed belt distance from `a` forward to `b`, in -P/2..P/2.
fn belt_gap(a: f32, b: f32) -> f32 {
    let p = perimeter();
    let d = (b - a).rem_euclid(p);
    if d > p / 2.0 {
        d - p
    } else {
        d
    }
}

/// Pushes a point on the floor out of the island and inside the walls.
pub fn collide(p: V2) -> V2 {
    let spine = V2::new(p.x.clamp(CX - HALF, CX + HALF), CY);
    let d = p - spine;
    let dist = d.len();
    let n = if dist < 1e-3 { V2::new(0.0, 1.0) } else { d * (1.0 / dist) };
    // The counter's front face hangs below its top, so keep clear of it too.
    let min = OUTER + BODY + SIDE * n.y.max(0.0);
    let p = if dist < min { spine + n * min } else { p };
    V2::new(p.x.clamp(MIN_X, MAX_X), p.y.clamp(MIN_Y, MAX_Y))
}

#[derive(Clone, Debug)]
pub struct Plate {
    pub dish: usize,
    pub s: f32,
    pub travelled: f32,
    pub age: f32,
    /// 0 while on the belt; then 0..1 as the kitchen takes it back.
    pub leaving: f32,
}

impl Plate {
    pub fn pos(&self) -> V2 {
        belt_point(self.s).0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Facing {
    Down,
    Up,
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct Eating {
    pub dish: usize,
    pub t: f32,
    /// Where the plate was taken from.
    pub from: V2,
    pub bitten: bool,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub pos: V2,
    pub facing: Facing,
    /// Walk cycle phase in radians.
    pub walk: f32,
    pub moving: bool,
    pub eating: Option<Eating>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    /// The task is done.
    Done,
    /// Closing time came first.
    Closed,
    Hospital,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Served { golden: bool },
    Ate { dish: usize, coins: i32, combo: u32, at: V2 },
    Full,
    Danger,
    Recovered,
    Outcome(Outcome),
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Controls {
    pub mx: f32,
    pub my: f32,
    pub eat: bool,
}

pub struct Round {
    pub t: f32,
    pub plates: Vec<Plate>,
    pub player: Player,
    pub coins: i32,
    pub fullness: f32,
    /// Seconds spent above the red line, while there.
    pub danger: Option<f32>,
    pub eaten: [u32; KINDS],
    pub plates_eaten: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub max_fullness: f32,
    pub outcome: Option<Outcome>,
    /// Seconds since the outcome.
    pub outro: f32,
    pub events: Vec<Event>,
    /// Seconds since the chef last served, for the reaching animation.
    pub chef_served: f32,
    /// Total belt travel, for the slats.
    pub belt_travel: f32,
    last_dish: Option<usize>,
    last_eat_t: f32,
    rng: u32,
    serve_timer: f32,
    golden_next: usize,
    golden_pending: bool,
    full: bool,
}

impl Round {
    pub fn new(seed: u32) -> Round {
        let mut r = Round {
            t: 0.0,
            plates: Vec::new(),
            player: Player {
                pos: V2::new(CX, CY + OUTER + BODY + SIDE + 22.0),
                facing: Facing::Up,
                walk: 0.0,
                moving: false,
                eating: None,
            },
            coins: 0,
            fullness: 0.0,
            danger: None,
            eaten: [0; KINDS],
            plates_eaten: 0,
            combo: 0,
            max_combo: 0,
            max_fullness: 0.0,
            outcome: None,
            outro: 0.0,
            events: Vec::new(),
            chef_served: 9.0,
            belt_travel: 0.0,
            last_dish: None,
            last_eat_t: -99.0,
            rng: seed.max(1),
            serve_timer: 0.0,
            golden_next: 0,
            golden_pending: false,
            full: false,
        };
        // Open with plates already riding, spaced out ahead of the chef.
        let gap = RULES.belt_speed * RULES.serve_every;
        let mut s = SERVE_S;
        for _ in 0..7 {
            s -= gap;
            let dish = r.pick_dish();
            r.add_plate(dish, wrap(s), SERVE_S - s);
        }
        r.events.clear();
        r
    }

    pub fn time_left(&self) -> f32 {
        (RULES.duration - self.t).max(0.0)
    }

    /// Seconds left to keep the combo going with a different dish.
    pub fn combo_left(&self) -> f32 {
        (RULES.combo_window - (self.t - self.last_eat_t)).max(0.0)
    }

    fn rand(&mut self) -> u32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x
    }

    fn pick_dish(&mut self) -> usize {
        let total: u32 = DISHES.iter().map(|d| d.weight).sum();
        let mut roll = self.rand() % total;
        for (i, d) in DISHES.iter().enumerate() {
            if roll < d.weight {
                return i;
            }
            roll -= d.weight;
        }
        0
    }

    fn add_plate(&mut self, dish: usize, s: f32, travelled: f32) {
        self.plates.push(Plate { dish, s, travelled, age: 9.0, leaving: 0.0 });
    }

    /// The plate the visitor would take now, by index.
    pub fn reachable(&self) -> Option<usize> {
        if self.player.eating.is_some() || self.outcome.is_some() {
            return None;
        }
        let hand = self.player.pos + V2::new(0.0, -10.0);
        self.plates
            .iter()
            .enumerate()
            .filter(|(_, p)| p.leaving == 0.0 && p.age > 0.35)
            .map(|(i, p)| (i, p.pos().dist(hand)))
            .filter(|(_, d)| *d < RULES.reach)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }

    pub fn update(&mut self, dt: f32, c: Controls) {
        self.chef_served += dt;
        self.move_belt(dt);
        if self.outcome.is_some() {
            self.outro += dt;
            self.player.moving = false;
            if let Some(e) = &mut self.player.eating {
                e.t += dt;
                if e.t >= RULES.eat_s {
                    self.player.eating = None;
                }
            }
            return;
        }
        self.t += dt;
        self.serve(dt);
        self.move_player(dt, c);
        self.digest(dt);
        if self.coins >= RULES.admission {
            self.finish(Outcome::Done);
        } else if self.danger.is_some_and(|d| d >= RULES.hospital_s) {
            self.finish(Outcome::Hospital);
        } else if self.t >= RULES.duration {
            self.finish(Outcome::Closed);
        }
    }

    fn finish(&mut self, o: Outcome) {
        self.outcome = Some(o);
        self.events.push(Event::Outcome(o));
    }

    fn move_belt(&mut self, dt: f32) {
        let d = RULES.belt_speed * dt;
        self.belt_travel += d;
        let lap = perimeter();
        for p in &mut self.plates {
            p.s = wrap(p.s + d);
            p.travelled += d;
            p.age += dt;
            if p.leaving > 0.0 {
                p.leaving += dt * 2.5;
            } else if p.travelled > RULES.laps * lap && belt_gap(SERVE_S, p.s).abs() < 8.0 {
                // The kitchen takes old plates back as they pass the pass.
                p.leaving = 1e-3;
            }
        }
        self.plates.retain(|p| p.leaving < 1.0);
    }

    fn serve(&mut self, dt: f32) {
        if self.golden_next < RULES.golden_at.len() && self.t >= RULES.golden_at[self.golden_next] {
            self.golden_next += 1;
            self.golden_pending = true;
        }
        self.serve_timer -= dt;
        if self.serve_timer > 0.0 {
            return;
        }
        let blocked = self.plates.iter().any(|p| p.leaving == 0.0 && belt_gap(SERVE_S, p.s).abs() < RULES.min_gap);
        if blocked {
            return;
        }
        let dish = if self.golden_pending {
            self.golden_pending = false;
            GOLDEN
        } else {
            self.pick_dish()
        };
        self.add_plate(dish, SERVE_S, 0.0);
        if let Some(p) = self.plates.last_mut() {
            p.age = 0.0;
        }
        self.serve_timer = RULES.serve_every;
        self.chef_served = 0.0;
        self.events.push(Event::Served { golden: dish == GOLDEN });
    }

    fn move_player(&mut self, dt: f32, c: Controls) {
        if let Some(e) = &mut self.player.eating {
            e.t += dt;
            let (dish, bite) = (e.dish, !e.bitten && e.t >= RULES.bite_s);
            if bite {
                e.bitten = true;
            }
            let done = e.t >= RULES.eat_s;
            if bite {
                self.bite(dish);
            }
            if done {
                self.player.eating = None;
            }
            self.player.moving = false;
            return;
        }
        let mut v = V2::new(c.mx, c.my);
        if v.len() > 1.0 {
            v = v.norm();
        }
        let speed = RULES.walk
            * if self.fullness > RULES.red_line {
                0.55
            } else if self.fullness > RULES.capacity {
                0.75
            } else {
                1.0
            };
        self.player.moving = v.len() > 0.15;
        if self.player.moving {
            self.player.pos = collide(self.player.pos + v * (speed * dt));
            self.player.walk += dt * 11.0 * v.len();
            self.player.facing = if v.x.abs() > v.y.abs() * 1.1 {
                if v.x > 0.0 {
                    Facing::Right
                } else {
                    Facing::Left
                }
            } else if v.y > 0.0 {
                Facing::Down
            } else {
                Facing::Up
            };
        }
        if c.eat {
            if let Some(i) = self.reachable() {
                let p = self.plates.remove(i);
                let from = p.pos();
                self.player.eating = Some(Eating { dish: p.dish, t: 0.0, from, bitten: false });
                self.player.moving = false;
                // Turn toward the plate.
                let d = from - self.player.pos;
                self.player.facing = if d.x.abs() > d.y.abs() {
                    if d.x > 0.0 {
                        Facing::Right
                    } else {
                        Facing::Left
                    }
                } else if d.y > 0.0 {
                    Facing::Down
                } else {
                    Facing::Up
                };
            }
        }
    }

    fn bite(&mut self, dish: usize) {
        let d = &DISHES[dish];
        let chained = self.last_dish.is_some_and(|l| l != dish) && self.t - self.last_eat_t <= RULES.combo_window;
        self.combo = if chained { self.combo + 1 } else { 1 };
        self.max_combo = self.max_combo.max(self.combo);
        self.last_dish = Some(dish);
        self.last_eat_t = self.t;
        let coins = d.coins + (self.combo as i32 - 1).clamp(0, 4);
        self.coins += coins;
        self.fullness += d.fullness as f32;
        self.max_fullness = self.max_fullness.max(self.fullness);
        self.eaten[dish] += 1;
        self.plates_eaten += 1;
        let at = self.player.pos + V2::new(0.0, -40.0);
        self.events.push(Event::Ate { dish, coins, combo: self.combo, at });
    }

    fn digest(&mut self, dt: f32) {
        self.fullness = (self.fullness - RULES.digest * dt).max(0.0);
        let full = self.fullness > RULES.capacity;
        if full && !self.full {
            self.events.push(Event::Full);
        }
        self.full = full;
        if self.fullness > RULES.red_line {
            if self.danger.is_none() {
                self.events.push(Event::Danger);
            }
            self.danger = Some(self.danger.unwrap_or(0.0) + dt);
        } else if self.danger.take().is_some() {
            self.events.push(Event::Recovered);
        }
    }

    /// Cards this meal has earned, by index into `CARDS`.
    pub fn cards_earned(&self) -> Vec<usize> {
        let mut out = Vec::new();
        let done = self.outcome == Some(Outcome::Done);
        let mut give = |art: CardArt, ok: bool| {
            if ok {
                out.push(card_index(art));
            }
        };
        give(CardArt::FirstPlate, self.plates_eaten > 0);
        give(CardArt::MoneysWorth, done);
        give(CardArt::Rainbow, self.eaten[..EVERYDAY].iter().all(|&n| n > 0));
        give(CardArt::Golden, self.eaten[GOLDEN] > 0);
        give(CardArt::LightEater, done && self.max_fullness <= RULES.capacity);
        give(CardArt::Combo, self.max_combo >= 5);
        give(CardArt::Ambulance, self.outcome == Some(Outcome::Hospital));
        out
    }

    /// A simple player for captures and balance checks: walks to the most
    /// worthwhile plate that fits and eats it.
    pub fn autopilot(&self) -> Controls {
        if self.player.eating.is_some() || self.outcome.is_some() {
            return Controls::default();
        }
        let room = RULES.red_line - 4.0 - self.fullness;
        let need = RULES.admission - self.coins;
        let worth = |d: &Dish| d.coins as f32 / d.fullness as f32 >= 0.75 || d.coins >= need;
        if let Some(i) = self.reachable() {
            let d = &DISHES[self.plates[i].dish];
            if (d.fullness as f32) <= room && worth(d) {
                return Controls { eat: true, ..Controls::default() };
            }
        }
        let me = self.player.pos;
        let best = self
            .plates
            .iter()
            .filter(|p| p.leaving == 0.0)
            .filter(|p| {
                let d = &DISHES[p.dish];
                (d.fullness as f32) <= room && worth(d)
            })
            .map(|p| {
                let eta = p.pos().dist(me) / RULES.walk;
                let (at, n) = belt_point(p.s + RULES.belt_speed * (eta + 0.4));
                let stand = collide(at + n * (BELT + LIP + BODY + 4.0));
                let d = &DISHES[p.dish];
                let score = d.coins as f32 / d.fullness as f32 - stand.dist(me) * 0.004;
                (score, stand)
            })
            .max_by(|a, b| a.0.total_cmp(&b.0));
        let Some((_, target)) = best else { return Controls::default() };
        let d = target - me;
        if d.len() < 3.0 {
            return Controls::default();
        }
        let v = d.norm();
        Controls { mx: v.x, my: v.y, eat: false }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn belt_is_continuous() {
        let p = perimeter();
        let mut prev = belt_point(0.0).0;
        let mut s = 0.5;
        while s <= p {
            let q = belt_point(s).0;
            assert!(q.dist(prev) < 0.6, "jump at {s}: {prev:?} -> {q:?}");
            prev = q;
            s += 0.5;
        }
    }

    #[test]
    fn visitor_stays_off_the_island() {
        for x in (0..64).map(|i| i as f32 * 10.0) {
            for y in (0..48).map(|i| i as f32 * 10.0) {
                let p = collide(V2::new(x, y));
                let spine = V2::new(p.x.clamp(CX - HALF, CX + HALF), CY);
                assert!(p.dist(spine) >= OUTER + BODY - 0.01, "{p:?}");
            }
        }
    }

    fn eat_now(r: &mut Round, dish: usize) {
        r.player.eating = Some(Eating { dish, t: 0.0, from: r.player.pos, bitten: false });
        for _ in 0..60 {
            r.update(1.0 / 60.0, Controls::default());
        }
    }

    #[test]
    fn eating_counts_and_chains() {
        let mut r = Round::new(7);
        eat_now(&mut r, 0);
        assert_eq!(r.coins, DISHES[0].coins);
        assert_eq!(r.combo, 1);
        eat_now(&mut r, 1);
        assert_eq!(r.combo, 2);
        assert_eq!(r.coins, DISHES[0].coins + DISHES[1].coins + 1);
        eat_now(&mut r, 1);
        assert_eq!(r.combo, 1, "the same dish twice breaks the chain");
    }

    #[test]
    fn red_zone_sends_to_hospital() {
        let mut r = Round::new(3);
        r.fullness = RULES.red_line + 30.0;
        for _ in 0..(60.0 * (RULES.hospital_s + 0.5)) as usize {
            r.update(1.0 / 60.0, Controls::default());
        }
        assert_eq!(r.outcome, Some(Outcome::Hospital));
        assert!(r.cards_earned().contains(&card_index(CardArt::Ambulance)));
    }

    #[test]
    fn recovering_clears_danger() {
        let mut r = Round::new(3);
        r.fullness = RULES.red_line + 1.0;
        for _ in 0..120 {
            r.update(1.0 / 60.0, Controls::default());
        }
        assert!(r.danger.is_none());
        assert!(r.outcome.is_none());
    }

    #[test]
    fn closing_without_eating() {
        let mut r = Round::new(5);
        for _ in 0..(60.0 * (RULES.duration + 1.0)) as usize {
            r.update(1.0 / 60.0, Controls::default());
        }
        assert_eq!(r.outcome, Some(Outcome::Closed));
        assert!(r.cards_earned().is_empty());
    }

    #[test]
    fn autopilot_can_finish() {
        let mut done = 0;
        for seed in 1..=8 {
            let mut r = Round::new(seed * 977);
            while r.outcome.is_none() {
                let c = r.autopilot();
                r.update(1.0 / 60.0, c);
            }
            assert_ne!(r.outcome, Some(Outcome::Hospital), "seed {seed}");
            if r.outcome == Some(Outcome::Done) {
                done += 1;
            }
        }
        assert!(done >= 6, "the task should be reachable, done {done} of 8");
    }
}
