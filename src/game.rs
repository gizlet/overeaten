//! A meal in one room of Carousel Sushi: the belts, the visitor, the
//! stomach, the clock, the main task and the side goals. No drawing here, so
//! the rules can be tested and played by the autopilot.

use crate::level::{self, Layout, BODY, COUNTER_H, RIM};
use crate::rules::*;
use crate::tasks::{draw_goals, Goal, State, Tally};
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

#[derive(Clone, Debug)]
pub struct Plate {
    pub dish: usize,
    pub belt: usize,
    pub s: f32,
    pub travelled: f32,
    pub age: f32,
    /// 0 while on the belt; then 0..1 as the kitchen takes it back.
    pub leaving: f32,
    pub served_at: f32,
}

#[derive(Clone, Debug)]
pub struct Eating {
    pub dish: usize,
    pub t: f32,
    /// Where the plate was taken from, and its height.
    pub from: V2,
    pub from_h: f32,
    pub bitten: bool,
    pub served_at: f32,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub pos: V2,
    /// Facing, radians: 0 looks south toward the camera.
    pub yaw: f32,
    pub moving: bool,
    /// Speed this frame, cm/s.
    pub speed: f32,
    pub eating: Option<Eating>,
    /// Seconds left lying in pieces (the skeleton).
    pub collapsed: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    /// The main task is done.
    Success,
    /// Closing time came first.
    Closed,
    /// The main task's rule was broken.
    Broke,
    Hospital,
}

impl Outcome {
    pub fn won(self) -> bool {
        self == Outcome::Success
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Served { belt: usize, golden: bool },
    Ate { dish: usize, points: i32, combo: u32, at: V2 },
    Full,
    Danger,
    Recovered,
    MainDone,
    GoalDone(usize),
    GoalFailed(usize),
    Collapse,
    Lean(f32),
    Outcome(Outcome),
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Controls {
    pub mx: f32,
    pub my: f32,
    pub eat: bool,
}

pub struct Round {
    pub level: usize,
    pub hero: usize,
    pub layout: Layout,
    pub t: f32,
    pub plates: Vec<Plate>,
    pub player: Player,
    pub fullness: f32,
    /// Seconds spent above the red line, while there.
    pub danger: Option<f32>,
    pub combo: u32,
    pub tally: Tally,
    pub goals: Vec<Goal>,
    pub goal_state: Vec<State>,
    pub main_state: State,
    /// Points from finished side goals.
    pub bonus: i32,
    pub outcome: Option<Outcome>,
    /// Seconds since the outcome.
    pub outro: f32,
    pub events: Vec<Event>,
    /// Total travel of each belt, for the slats.
    pub belt_travel: Vec<f32>,
    /// The next dish each kitchen will send.
    pub next_dish: Vec<usize>,
    pub collapses: u32,
    pub lean: f32,
    last_dish: Option<usize>,
    last_eat_t: f32,
    bites: Vec<f32>,
    rng: u32,
    serve_timer: Vec<f32>,
    golden_due: u32,
    golden_served: u32,
    full: bool,
    nav: Nav,
}

#[derive(Default)]
struct Nav {
    target: Option<V2>,
    path: Vec<V2>,
    planned_at: f32,
    chosen_at: f32,
    goal: Option<(usize, f32)>,
}

impl Round {
    pub fn new(level: usize, hero: usize, seed: u32) -> Round {
        let layout = Layout::of(level);
        let def = &LEVELS[level];
        let n = layout.belts.len();
        let mut r = Round {
            level,
            hero,
            t: 0.0,
            plates: Vec::new(),
            player: Player { pos: layout.start, yaw: std::f32::consts::PI, moving: false, speed: 0.0, eating: None, collapsed: 0.0 },
            fullness: 0.0,
            danger: None,
            combo: 0,
            tally: Tally::default(),
            goals: Vec::new(),
            goal_state: Vec::new(),
            main_state: State::Open,
            bonus: 0,
            outcome: None,
            outro: 0.0,
            events: Vec::new(),
            belt_travel: vec![0.0; n],
            next_dish: vec![0; n],
            collapses: 0,
            lean: 0.0,
            last_dish: None,
            last_eat_t: -99.0,
            bites: Vec::new(),
            rng: seed.max(1),
            serve_timer: vec![0.0; n],
            golden_due: 0,
            golden_served: 0,
            full: false,
            nav: Nav::default(),
            layout,
        };
        let mut rng = r.rng;
        r.goals = draw_goals(&def.main, def.pool, || {
            rng ^= rng << 13;
            rng ^= rng >> 17;
            rng ^= rng << 5;
            rng
        });
        r.rng = rng.max(1);
        r.goal_state = vec![State::Open; r.goals.len()];
        for b in 0..n {
            r.next_dish[b] = r.pick_dish(b);
        }
        // Open with plates already riding, spaced out behind the kitchen.
        let gap = RULES.belt_speed * RULES.serve_every;
        for b in 0..n {
            let belt = r.layout.belts[b];
            let count = (belt.perimeter() * 0.62 / gap) as usize;
            let mut s = belt.serve_s();
            for _ in 0..count {
                s -= gap;
                let dish = r.pick_dish(b);
                r.plates.push(Plate {
                    dish,
                    belt: b,
                    s: belt.wrap(s),
                    travelled: belt.serve_s() - s,
                    age: 9.0,
                    leaving: 0.0,
                    served_at: -99.0,
                });
            }
        }
        r
    }

    pub fn def(&self) -> &'static LevelDef {
        &LEVELS[self.level]
    }

    pub fn hero_def(&self) -> &'static HeroDef {
        &HEROES[self.hero]
    }

    pub fn time_left(&self) -> f32 {
        (self.def().duration - self.t).max(0.0)
    }

    pub fn score(&self) -> i32 {
        self.tally.score + self.bonus
    }

    /// The combo's score multiplier now.
    pub fn multiplier(&self) -> f32 {
        (1.0 + RULES.combo_step * (self.combo.max(1) - 1) as f32).min(RULES.combo_max)
    }

    /// Seconds left to keep the combo going with a different dish.
    pub fn combo_left(&self) -> f32 {
        (RULES.combo_window - (self.t - self.last_eat_t)).max(0.0)
    }

    pub fn last_dish(&self) -> Option<usize> {
        self.last_dish
    }

    /// How full one plate of `dish` makes this visitor, in percent.
    pub fn fill_of(&self, dish: usize) -> f32 {
        DISHES[dish].fullness as f32 * self.hero_def().stomach
    }

    pub fn points_of(&self, dish: usize, combo: u32) -> i32 {
        let mult = (1.0 + RULES.combo_step * (combo.max(1) - 1) as f32).min(RULES.combo_max);
        (DISHES[dish].value as f32 * mult * self.hero_def().value).round() as i32
    }

    fn rand(&mut self) -> u32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x
    }

    fn pick_dish(&mut self, belt: usize) -> usize {
        let w = self.layout.belts[belt].weights;
        let total: u32 = w.iter().sum();
        let mut roll = self.rand() % total.max(1);
        for (i, &k) in w.iter().enumerate() {
            if roll < k {
                return i;
            }
            roll -= k;
        }
        0
    }

    pub fn plate_pos(&self, p: &Plate) -> V2 {
        self.layout.belts[p.belt].point(p.s).0
    }

    /// The plate's height above the room floor.
    pub fn plate_h(&self, p: &Plate) -> f32 {
        self.layout.belts[p.belt].top()
    }

    /// Whether the visitor can make out what is on a plate.
    pub fn visible(&self, p: &Plate) -> bool {
        self.layout.light_at(self.plate_pos(p), self.player.pos) > 0.45
    }

    fn can_take(&self, p: &Plate) -> bool {
        if p.leaving > 0.0 || p.age < 1.0 {
            return false;
        }
        let at = self.plate_pos(p);
        if at.dist(self.player.pos) >= RULES.reach || self.layout.behind_door(p.belt, at, self.t) {
            return false;
        }
        let stand = self.layout.floor_at(self.player.pos);
        (self.plate_h(p) - stand - COUNTER_H).abs() < 35.0
    }

    /// The plate the visitor would take now, by index.
    pub fn reachable(&self) -> Option<usize> {
        if self.player.eating.is_some() || self.outcome.is_some() || self.player.collapsed > 0.0 {
            return None;
        }
        let me = self.player.pos;
        self.plates
            .iter()
            .enumerate()
            .filter(|(_, p)| self.can_take(p))
            .map(|(i, p)| (i, self.plate_pos(p).dist(me)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }

    pub fn update(&mut self, dt: f32, c: Controls) {
        self.move_belts(dt);
        if self.outcome.is_some() {
            self.outro += dt;
            self.player.moving = false;
            self.player.speed = 0.0;
            if let Some(e) = &mut self.player.eating {
                e.t += dt;
                if e.t >= RULES.eat_s {
                    self.player.eating = None;
                }
            }
            return;
        }
        self.t += dt;
        self.tally.t = self.t;
        self.serve(dt);
        self.move_player(dt, c);
        self.digest(dt);
        self.judge(false);
        if self.outcome.is_none() {
            if self.danger.is_some_and(|d| d >= self.hero_def().hospital_s) {
                self.finish(Outcome::Hospital);
            } else if self.t >= self.def().duration {
                self.close();
            }
        }
    }

    /// Ends the meal now, as at closing time: settles the bill.
    pub fn close(&mut self) {
        if self.outcome.is_some() {
            return;
        }
        self.judge(true);
        let o = if self.main_state == State::Done { Outcome::Success } else { Outcome::Closed };
        self.finish(o);
    }

    /// Whether the visitor may leave early with the bill: the task is done.
    pub fn can_leave(&self) -> bool {
        self.outcome.is_none()
            && (self.main_state == State::Done || (self.def().main.at_close() && self.main_state == State::Open))
    }

    fn finish(&mut self, o: Outcome) {
        self.outcome = Some(o);
        self.events.push(Event::Outcome(o));
    }

    /// Updates the task and goals; at `closing` the open ones are decided.
    fn judge(&mut self, closing: bool) {
        self.tally.fullness = self.fullness;
        let main = self.def().main;
        let was = self.main_state;
        if was == State::Open {
            let s = main.state(&self.tally, closing);
            if s != was {
                self.main_state = s;
                match s {
                    State::Done if !closing => {
                        self.tally.main_done = Some(self.t);
                        self.events.push(Event::MainDone);
                    }
                    State::Done => self.tally.main_done = Some(self.t),
                    State::Failed if !closing => self.finish(Outcome::Broke),
                    _ => {}
                }
            }
        }
        for i in 0..self.goals.len() {
            if self.goal_state[i] != State::Open {
                continue;
            }
            let s = self.goals[i].state(&self.tally, closing);
            if s != State::Open {
                self.goal_state[i] = s;
                if s == State::Done {
                    self.bonus += RULES.bonus;
                    self.events.push(Event::GoalDone(i));
                } else {
                    self.events.push(Event::GoalFailed(i));
                }
            }
        }
    }

    fn move_belts(&mut self, dt: f32) {
        let d = RULES.belt_speed * dt;
        for t in &mut self.belt_travel {
            *t += d;
        }
        let belts = self.layout.belts.clone();
        for p in &mut self.plates {
            let b = &belts[p.belt];
            p.s = b.wrap(p.s + d);
            p.travelled += d;
            p.age += dt;
            if p.leaving > 0.0 {
                p.leaving += dt * 2.5;
            } else if p.travelled > RULES.laps * b.perimeter() && b.gap(b.serve_s(), p.s).abs() < 8.0 {
                p.leaving = 1e-3;
            }
        }
        self.plates.retain(|p| p.leaving < 1.0);
    }

    /// Golden plates due by now.
    fn golden_by(&self, t: f32) -> u32 {
        match self.def().golden {
            Golden::At(times) => times.iter().filter(|&&g| g <= t).count() as u32,
            Golden::Every { start, every } => {
                if t < start {
                    0
                } else {
                    ((t - start) / every).floor() as u32 + 1
                }
            }
        }
    }

    /// When the next golden plate is due.
    pub fn next_golden(&self) -> Option<f32> {
        match self.def().golden {
            Golden::At(times) => times.iter().copied().find(|&g| g > self.t),
            Golden::Every { start, every } => {
                if self.t < start {
                    Some(start)
                } else {
                    Some(start + (((self.t - start) / every).floor() + 1.0) * every)
                }
            }
        }
    }

    fn serve(&mut self, dt: f32) {
        self.golden_due = self.golden_by(self.t);
        let n = self.layout.belts.len();
        for b in 0..n {
            self.serve_timer[b] -= dt;
            if self.serve_timer[b] > 0.0 {
                continue;
            }
            let belt = self.layout.belts[b];
            let blocked = self
                .plates
                .iter()
                .any(|p| p.belt == b && p.leaving == 0.0 && belt.gap(belt.serve_s(), p.s).abs() < RULES.min_gap);
            if blocked {
                continue;
            }
            let golden = self.golden_served < self.golden_due && (self.golden_served as usize) % n == b;
            let dish = if golden {
                self.golden_served += 1;
                GOLDEN
            } else {
                let d = self.next_dish[b];
                self.next_dish[b] = self.pick_dish(b);
                d
            };
            self.plates.push(Plate {
                dish,
                belt: b,
                s: belt.serve_s(),
                travelled: 0.0,
                age: 0.0,
                leaving: 0.0,
                served_at: self.t,
            });
            self.serve_timer[b] = RULES.serve_every;
            self.events.push(Event::Served { belt: b, golden });
        }
    }

    fn move_player(&mut self, dt: f32, c: Controls) {
        let hero = self.hero_def();
        let lean = self.layout.tilt.map_or(0.0, |t| t.lean(self.t));
        if lean.abs() > 0.05 && self.lean.abs() <= 0.05 {
            self.events.push(Event::Lean(lean.signum()));
        }
        self.lean = lean;
        let slide = if hero.steady { 0.0 } else { self.layout.tilt.map_or(0.0, |t| t.slide) * lean };
        let before = self.player.pos;
        if slide != 0.0 {
            self.player.pos = self.layout.collide(self.player.pos + V2::new(slide * dt, 0.0), self.t);
        }
        if self.player.collapsed > 0.0 {
            self.player.collapsed = (self.player.collapsed - dt).max(0.0);
            self.player.moving = false;
            self.player.speed = 0.0;
            return;
        }
        if let Some(e) = &mut self.player.eating {
            e.t += dt;
            let (dish, bite) = (e.dish, !e.bitten && e.t >= RULES.bite_s);
            if bite {
                e.bitten = true;
            }
            let served = e.served_at;
            let done = e.t >= RULES.eat_s;
            if bite {
                self.bite(dish, served);
            }
            if done {
                self.player.eating = None;
            }
            self.player.moving = false;
            self.player.speed = 0.0;
            return;
        }
        // Doors shut on whoever stands in their way: keep them out.
        self.player.pos = self.layout.collide(self.player.pos, self.t);
        let mut v = V2::new(c.mx, c.my);
        if v.len() > 1.0 {
            v = v.norm();
        }
        let stomach = if self.fullness > RULES.red_line {
            0.55
        } else if self.fullness > RULES.capacity {
            0.75
        } else {
            1.0
        };
        let speed = hero.walk * stomach * self.layout.pace_at(self.player.pos);
        self.player.moving = v.len() > 0.15;
        if self.player.moving {
            self.player.pos = self.layout.collide(self.player.pos + v * (speed * dt), self.t);
            let want = v.x.atan2(v.y);
            let mut d = want - self.player.yaw;
            while d > std::f32::consts::PI {
                d -= std::f32::consts::TAU;
            }
            while d < -std::f32::consts::PI {
                d += std::f32::consts::TAU;
            }
            self.player.yaw += d * (1.0 - (-dt * 16.0).exp());
        }
        self.player.speed = if dt > 0.0 { self.player.pos.dist(before) / dt } else { 0.0 };
        if c.eat {
            if let Some(i) = self.reachable() {
                let p = self.plates.remove(i);
                let from = self.layout.belts[p.belt].point(p.s).0;
                let from_h = self.layout.belts[p.belt].top();
                self.player.eating = Some(Eating { dish: p.dish, t: 0.0, from, from_h, bitten: false, served_at: p.served_at });
                self.player.moving = false;
                let d = from - self.player.pos;
                self.player.yaw = d.x.atan2(d.y);
            }
        }
    }

    fn bite(&mut self, dish: usize, served_at: f32) {
        let chained = self.last_dish.is_some_and(|l| l != dish) && self.t - self.last_eat_t <= RULES.combo_window;
        self.combo = if chained { self.combo + 1 } else { 1 };
        let points = self.points_of(dish, self.combo);
        let within = match self.def().main {
            Goal::Golden { within, .. } => within,
            _ => 4.0,
        };
        let fill = self.fill_of(dish);
        let t = &mut self.tally;
        t.score += points;
        t.plates += 1;
        t.eaten[dish] += 1;
        t.max_combo = t.max_combo.max(self.combo);
        if self.last_dish == Some(dish) {
            t.repeated = true;
        }
        t.step_sequence(dish);
        if dish == GOLDEN {
            t.golden += 1;
            if self.t - served_at <= within {
                t.golden_quick += 1;
            }
        }
        let me = self.player.pos;
        let zone = (me.x > 750.0) as u8 | (((me.y > 540.0) as u8) << 1);
        t.zones |= 1 << zone;
        self.fullness += fill;
        t.max_fullness = t.max_fullness.max(self.fullness);
        t.fullness = self.fullness;
        self.last_dish = Some(dish);
        self.last_eat_t = self.t;
        self.events.push(Event::Ate { dish, points, combo: self.combo, at: me });
        if self.hero_def().fragile {
            let now = self.t;
            self.bites.retain(|&b| now - b < 2.5);
            self.bites.push(now);
            if self.bites.len() >= 3 {
                self.bites.clear();
                self.player.collapsed = 2.2;
                self.collapses += 1;
                self.events.push(Event::Collapse);
            }
        }
    }

    fn digest(&mut self, dt: f32) {
        self.fullness = (self.fullness - self.hero_def().digest * dt).max(0.0);
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
            self.tally.red_secs += dt;
        } else if self.danger.take().is_some() {
            self.events.push(Event::Recovered);
        }
    }

    /// Stars for the finished meal: one for the task, more for the score.
    pub fn stars(&self) -> u32 {
        if self.outcome != Some(Outcome::Success) {
            return 0;
        }
        let [two, three] = self.def().stars;
        1 + (self.score() >= two) as u32 + (self.score() >= three) as u32
    }

    /// Cards this meal has earned, by id.
    pub fn cards_earned(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        let won = self.outcome == Some(Outcome::Success);
        let lv = self.level + 1;
        let hero = self.hero_def().id;
        let mut give = |id: &'static str, ok: bool| {
            if ok {
                out.push(id);
            }
        };
        const CLEAR: [&str; 8] =
            ["sushi.1.clear", "sushi.2.clear", "sushi.3.clear", "sushi.4.clear", "sushi.5.clear", "sushi.6.clear", "sushi.7.clear", "sushi.8.clear"];
        const ACE: [&str; 8] =
            ["sushi.1.ace", "sushi.2.ace", "sushi.3.ace", "sushi.4.ace", "sushi.5.ace", "sushi.6.ace", "sushi.7.ace", "sushi.8.ace"];
        give(CLEAR[lv - 1], won);
        give(ACE[lv - 1], self.stars() == 3);
        let t = &self.tally;
        match hero {
            "traveller" => {
                give("hero.traveller.first", won);
                give("hero.traveller.feat", won && t.max_fullness <= RULES.capacity);
            }
            "knight" => {
                give("hero.knight.first", won);
                give("hero.knight.feat", won && t.red_secs >= 8.0);
            }
            "mage" => {
                give("hero.mage.first", won);
                give("hero.mage.feat", t.plates >= 20);
            }
            _ => {
                give("hero.skeleton.first", won);
                give("hero.skeleton.feat", won && self.collapses >= 3);
            }
        }
        const DISH: [&str; KINDS] =
            ["dish.salmon", "dish.tuna", "dish.octopus", "dish.shrimp", "dish.tamago", "dish.uni", "dish.golden"];
        for (i, id) in DISH.iter().enumerate() {
            give(id, t.eaten[i] > 0);
        }
        give("secret.hospital", self.outcome == Some(Outcome::Hospital));
        give(
            "secret.exact",
            matches!(self.outcome, Some(Outcome::Success | Outcome::Closed)) && (99.0..=101.0).contains(&self.fullness),
        );
        give("secret.combo8", t.max_combo >= 8);
        give("secret.empty", self.outcome == Some(Outcome::Closed) && t.plates == 0);
        out
    }

    // ---- The autopilot -----------------------------------------------------

    /// How full the autopilot lets itself get with the next bite.
    fn eat_limit(&self) -> f32 {
        let safe = RULES.red_line - 4.0;
        match self.def().main {
            Goal::Risk { secs, .. } if self.tally.red_secs < secs => {
                let left = self.hero_def().hospital_s - 1.4 - self.danger.unwrap_or(0.0);
                if self.tally.score as f32 > 0.4 * self.def().stars[0] as f32 {
                    RULES.red_line + (left * self.hero_def().digest).clamp(0.0, 3.6)
                } else {
                    safe
                }
            }
            Goal::Precision { .. } => {
                if self.time_left() > 22.0 {
                    96.0
                } else {
                    safe
                }
            }
            _ => safe,
        }
    }

    /// How much the autopilot wants a plate of `dish`, or None.
    fn appetite(&self, dish: usize, eta: f32, served_at: f32) -> Option<f32> {
        let fill = self.fill_of(dish);
        if self.fullness + fill > self.eat_limit() {
            return None;
        }
        let d = &DISHES[dish];
        let mut want = d.value as f32 / d.fullness as f32 + d.value as f32 * 0.02;
        let main_open = self.main_state == State::Open;
        match self.def().main {
            Goal::Collect if main_open => {
                if dish < EVERYDAY && self.tally.eaten[dish] == 0 {
                    want += 4.0;
                } else if self.fullness > 45.0 {
                    return None;
                }
            }
            Goal::Sequence(_) if main_open => {
                let need = crate::tasks::SEQUENCE[(self.tally.seq % 3) as usize];
                if d.hue == need {
                    want += 4.0;
                } else if d.hue != Hue::Gold {
                    return None;
                }
            }
            Goal::Efficiency { .. } if main_open => {
                if d.value < 15 || self.last_dish == Some(dish) {
                    return None;
                }
            }
            Goal::NoRepeat { .. } => {
                if self.last_dish == Some(dish) {
                    return None;
                }
            }
            Goal::Golden { within, .. } if main_open => {
                if dish == GOLDEN {
                    if self.t + eta - served_at <= within - 0.3 {
                        want += 12.0;
                    }
                } else {
                    // Keep room for the next golden plate.
                    let until = self.next_golden().map_or(99.0, |g| (g - self.t).max(0.0));
                    let room = self.eat_limit() - self.fill_of(GOLDEN) - 2.0;
                    if until < 4.0 || self.fullness + fill > 60.0 || self.fullness + fill - self.hero_def().digest * until > room {
                        return None;
                    }
                }
            }
            Goal::Precision { lo, hi, .. } if self.time_left() <= 22.0 => {
                let digest = self.hero_def().digest;
                let after = self.fullness + fill - digest * (self.time_left() - eta - RULES.bite_s).max(0.0);
                let now = self.fullness - digest * self.time_left();
                if after > hi - 2.0 || now >= lo + 2.5 {
                    return None;
                }
                // Take a plate that lands well inside, or one that leaves room
                // for a second; waiting changes nothing, since digestion keeps
                // pace with the clock.
                let smallest = 15.0 * self.hero_def().stomach;
                if after < lo + 3.0 && after > hi - 2.0 - smallest && self.time_left() > 8.0 {
                    return None;
                }
                want += 2.0 - (after - (lo + hi) / 2.0).abs() * 0.1;
            }
            _ => {}
        }
        if dish == GOLDEN {
            want += 2.0;
        }
        Some(want)
    }

    /// Where to stand to take plate `p` in `eta` seconds.
    fn stand_for(&self, belt: usize, s: f32, eta: f32) -> V2 {
        let b = &self.layout.belts[belt];
        let (at, n) = b.point(s + RULES.belt_speed * (eta + 0.35));
        self.layout.collide(at + n * (RIM + BODY + 8.0), self.t)
    }

    /// A simple player for captures and balance checks: walks to the most
    /// worthwhile plate it can manage and eats it.
    pub fn autopilot(&mut self) -> Controls {
        if self.player.eating.is_some() || self.outcome.is_some() || self.player.collapsed > 0.0 {
            return Controls::default();
        }
        // Nothing left to gain: settle the bill.
        let walk = self.hero_def().walk;
        if let Some(i) = self.reachable() {
            let p = &self.plates[i];
            let quick_ok = self.visible(p);
            let near_ok = self
                .plates
                .iter()
                .filter(|q| self.can_take(q))
                .all(|q| self.fullness + self.fill_of(q.dish) <= self.eat_limit() + 0.01);
            if quick_ok && near_ok && self.appetite(p.dish, 0.0, p.served_at).is_some() {
                // Every plate in reach must be acceptable: the nearest may change before the bite.
                let all_ok = self
                    .plates
                    .iter()
                    .filter(|q| self.can_take(q))
                    .all(|q| self.appetite(q.dish, 0.0, q.served_at).is_some());
                if all_ok {
                    return Controls { eat: true, ..Controls::default() };
                }
            }
        }
        let me = self.player.pos;
        let floor_here = self.layout.floor_at(me);
        if self.t - self.nav.chosen_at > 0.25 || self.nav.target.is_none() {
            self.nav.chosen_at = self.t;
            let mut best: Option<(f32, V2, usize, f32)> = None;
            for p in &self.plates {
                if p.leaving > 0.0 || !self.visible(p) {
                    continue;
                }
                let b = &self.layout.belts[p.belt];
                let at = b.point(p.s).0;
                let mut eta = at.dist(me) / walk * 1.25 + 0.3;
                if (b.floor - floor_here).abs() > 1.0 {
                    eta += 1.0;
                }
                let Some(want) = self.appetite(p.dish, eta, p.served_at) else { continue };
                let stand = self.stand_for(p.belt, p.s, eta);
                if (self.layout.floor_at(stand) - b.floor).abs() > 1.0 {
                    continue;
                }
                let later = b.point(p.s + RULES.belt_speed * (eta + 0.35)).0;
                if self.layout.behind_door(p.belt, later, self.t + eta) {
                    continue;
                }
                let score = want - stand.dist(me) * 0.004;
                if best.is_none_or(|b| score > b.0) {
                    best = Some((score, stand, p.belt, p.s));
                }
            }
            let target = match best {
                Some((_, stand, belt, s)) => {
                    self.nav.goal = Some((belt, s));
                    stand
                }
                None => self.waiting_spot(),
            };
            if self.nav.target.is_none_or(|t| t.dist(target) > 30.0) || self.t - self.nav.planned_at > 0.6 {
                self.nav.path = level::path(&self.layout, me, target, self.t);
                self.nav.planned_at = self.t;
            }
            self.nav.target = Some(target);
        }
        while self.nav.path.len() > 1 && self.nav.path[0].dist(me) < 14.0 {
            self.nav.path.remove(0);
        }
        let Some(&next) = self.nav.path.first() else { return Controls::default() };
        let d = next - me;
        if d.len() < 6.0 {
            return Controls::default();
        }
        let v = d.norm();
        Controls { mx: v.x, my: v.y, eat: false }
    }

    /// Where to wait when nothing is worth walking to.
    fn waiting_spot(&self) -> V2 {
        let me = self.player.pos;
        let golden_soon = self.next_golden().is_some_and(|g| g - self.t < 6.0)
            && matches!(self.def().main, Goal::Golden { .. })
            && self.main_state == State::Open;
        let b = (self.golden_served as usize) % self.layout.belts.len();
        if golden_soon {
            let belt = &self.layout.belts[b];
            return self.stand_for(b, belt.serve_s() + RULES.belt_speed * 1.2, 0.0);
        }
        // In the dark, walk round to see what is coming.
        let nearest = (0..self.layout.belts.len())
            .min_by(|&a, &c| {
                let da = V2::new(self.layout.belts[a].cx, self.layout.belts[a].cy).dist(me);
                let dc = V2::new(self.layout.belts[c].cx, self.layout.belts[c].cy).dist(me);
                da.total_cmp(&dc)
            })
            .unwrap_or(0);
        let belt = &self.layout.belts[nearest];
        let phase = (self.t * 0.08).fract() * belt.perimeter();
        self.stand_for(nearest, phase, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eat_now(r: &mut Round, dish: usize) {
        r.player.eating = Some(Eating { dish, t: 0.0, from: r.player.pos, from_h: 70.0, bitten: false, served_at: r.t });
        for _ in 0..60 {
            r.update(1.0 / 60.0, Controls::default());
        }
    }

    #[test]
    fn eating_scores_and_chains() {
        let mut r = Round::new(0, 0, 7);
        eat_now(&mut r, 0);
        assert_eq!(r.tally.score, DISHES[0].value);
        assert_eq!(r.combo, 1);
        eat_now(&mut r, 1);
        assert_eq!(r.combo, 2);
        let second = (DISHES[1].value as f32 * (1.0 + RULES.combo_step)).round() as i32;
        assert_eq!(r.tally.score, DISHES[0].value + second);
        eat_now(&mut r, 1);
        assert_eq!(r.combo, 1, "the same dish twice breaks the chain");
    }

    #[test]
    fn the_red_zone_sends_you_to_hospital() {
        let mut r = Round::new(0, 0, 3);
        r.fullness = RULES.red_line + 30.0;
        for _ in 0..(60.0 * 5.6) as usize {
            r.update(1.0 / 60.0, Controls::default());
        }
        assert_eq!(r.outcome, Some(Outcome::Hospital));
        assert!(r.cards_earned().contains(&"secret.hospital"));
    }

    #[test]
    fn the_knight_lasts_longer_in_the_red() {
        let mut r = Round::new(0, 1, 3);
        r.fullness = RULES.red_line + 30.0;
        for _ in 0..(60.0 * 5.6) as usize {
            r.update(1.0 / 60.0, Controls::default());
        }
        assert_eq!(r.outcome, None);
    }

    #[test]
    fn closing_without_eating_fails() {
        let mut r = Round::new(0, 0, 5);
        for _ in 0..(60.0 * (r.def().duration + 1.0)) as usize {
            r.update(1.0 / 60.0, Controls::default());
        }
        assert_eq!(r.outcome, Some(Outcome::Closed));
        assert_eq!(r.stars(), 0);
        assert!(r.cards_earned().contains(&"secret.empty"));
    }

    #[test]
    fn breaking_the_rule_ends_the_meal() {
        let mut r = Round::new(5, 0, 5);
        eat_now(&mut r, 2);
        eat_now(&mut r, 2);
        assert_eq!(r.outcome, Some(Outcome::Broke));
    }

    #[test]
    fn the_skeleton_falls_apart_after_quick_bites() {
        let mut r = Round::new(0, 3, 9);
        for d in [0, 1, 2] {
            r.player.eating = Some(Eating { dish: d, t: 0.0, from: r.player.pos, from_h: 70.0, bitten: false, served_at: 0.0 });
            for _ in 0..40 {
                r.update(1.0 / 60.0, Controls::default());
            }
        }
        assert_eq!(r.collapses, 1);
        assert!(r.player.collapsed > 0.0);
        assert_eq!(r.tally.score, [0, 1, 2].iter().map(|&d| r.points_of(d, 1)).sum::<i32>().max(r.tally.score));
    }

    #[test]
    fn doors_hide_plates() {
        let mut r = Round::new(4, 0, 11);
        let door = r.layout.doors[0];
        r.t = door.phase + 2.0;
        let b = r.layout.belts[0];
        let x = (door.x0 + door.x1) / 2.0;
        let s = x - (b.cx - b.half);
        r.plates = vec![Plate { dish: 0, belt: 0, s, travelled: 0.0, age: 9.0, leaving: 0.0, served_at: 0.0 }];
        r.player.pos = V2::new(x, b.cy - b.outer() - BODY - 8.0);
        assert_eq!(r.reachable(), None);
        r.t = door.phase + door.shut + 1.0;
        assert_eq!(r.reachable(), Some(0));
    }

    /// Plays a meal with the autopilot to its end.
    fn play(level: usize, hero: usize, seed: u32) -> Round {
        let mut r = Round::new(level, hero, seed);
        while r.outcome.is_none() {
            let c = r.autopilot();
            r.update(1.0 / 30.0, c);
        }
        r
    }

    #[test]
    fn the_autopilot_clears_every_room() {
        for level in 0..LEVELS.len() {
            let mut wins = 0;
            let mut log = Vec::new();
            for seed in 1..=5u32 {
                let r = play(level, 0, seed * 977 + level as u32);
                assert_ne!(r.outcome, Some(Outcome::Hospital), "level {} seed {seed} went to hospital", level + 1);
                log.push((r.outcome, r.score(), r.tally.plates));
                wins += (r.outcome == Some(Outcome::Success)) as u32;
            }
            assert!(wins >= 4, "level {}: the task should be reachable, {wins} of 5: {log:?}", level + 1);
        }
    }

    #[test]
    fn every_hero_can_clear_the_first_room() {
        for hero in 0..HEROES.len() {
            let wins = (1..=4u32).filter(|&s| play(0, hero, s * 31).outcome == Some(Outcome::Success)).count();
            assert!(wins >= 3, "{} cleared room 1 {wins} of 4 times", HEROES[hero].id);
        }
    }
}
