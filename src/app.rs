//! Screens and what moves between them: the title (with a meal playing by
//! itself behind it), choosing a hero, choosing a room by its ticket, the
//! meal, the bill and the card album.

use crate::cards::Portraits;
use crate::fx::Fx;
use crate::game::{Event, Outcome, Round, V2};
use crate::gfx::rgb;
use crate::hero::{Act, Hero};
use crate::input::{Button, Controls};
use crate::render::Gl;
use crate::rules::*;
use crate::save::Save;
use crate::scene::{self, Scene};
use crate::stats::Stats;
use crate::tasks::{draw_goals, Goal, State};
use crate::text as tx;
use crate::ui::pal;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Title,
    Heroes,
    Levels,
    Dining,
    Result,
    Album,
}

/// The bill of a finished meal.
pub struct Summary {
    pub level: usize,
    pub outcome: Outcome,
    pub score: i32,
    /// Points of food before combos.
    pub food: i32,
    pub combo_bonus: i32,
    pub goals: Vec<(Goal, State)>,
    pub bonus: i32,
    pub stars: u32,
    pub best_before: Option<i32>,
    pub plates: u32,
    pub peak: f32,
    /// Cards this meal earned for the first time, by index into `CARDS`.
    pub new_cards: Vec<usize>,
    pub new_heroes: Vec<usize>,
    /// Seconds since the bill came.
    pub t: f32,
}

impl Summary {
    /// When card `i` of the rack starts turning over.
    pub fn reveal_at(i: usize) -> f32 {
        1.6 + i as f32 * 0.32
    }

    pub fn all_revealed(&self) -> bool {
        self.t >= Summary::reveal_at(self.new_cards.len()) + 0.3
    }
}

pub struct App {
    pub lang: Lang,
    pub screen: Screen,
    pub t: f32,
    pub screen_t: f32,
    /// 1 right after a screen change, fading to 0.
    pub fade: f32,
    pub save: Save,
    pub heroes: Vec<Hero>,
    /// The sushi chef inside the belts.
    pub chef: Option<Hero>,
    pub hero_sel: usize,
    pub level_sel: usize,
    /// Eases toward `level_sel`, for sliding the tickets.
    pub level_pos: f32,
    pub hero_pos: f32,
    /// The meal being played or shown: the real one, the title's or a room's preview.
    pub round: Option<Round>,
    pub scene: Option<Scene>,
    pub ticket_seed: u32,
    pub ticket_goals: Vec<Goal>,
    pub fx: Fx,
    pub paused: Option<usize>,
    pub summary: Option<Summary>,
    pub album_book: usize,
    pub album_sel: usize,
    /// Seconds the selected card has been open, while open.
    pub album_open: Option<f32>,
    album_back: Screen,
    pub autopilot: bool,
    /// Draws the app icon instead of a screen, for `OVEREATEN_SCENE=icon`.
    pub icon: bool,
    /// The gauge and counters ease toward the real numbers.
    pub shown_fullness: f32,
    pub shown_score: f32,
    seed: u32,
    /// Labels of the A, B and X buttons after the A/B swap.
    pub labels: [&'static str; 3],
    pub stats_on: bool,
    pub stats: Stats,
    pub portraits: Portraits,
    pub gl: Gl,
    pub bump_score: f32,
    music: bool,
}

fn load_heroes() -> Result<Vec<Hero>, String> {
    let dir = crate::assets::get().dir.join("heroes");
    HEROES.iter().map(|h| Hero::load(&dir.join(format!("{}.glb", h.model)))).collect()
}

impl App {
    pub fn new(save: Save, controls: &Controls) -> Result<App, String> {
        let lang = match save.lang.as_str() {
            "en" => Lang::En,
            "zh" => Lang::Zh,
            _ => default_lang(),
        };
        let gl = Gl::new();
        let mut heroes = load_heroes()?;
        let chef = Hero::load(&crate::assets::get().dir.join("heroes/Chef.glb"))
            .map_err(|e| eprintln!("overeaten: no chef: {e}"))
            .ok();
        let mut portraits = Portraits::new();
        portraits.render_all(&gl, &mut heroes);
        let hero_sel = if save.hero_unlocked(save.hero) { save.hero } else { 0 };
        let mut app = App {
            lang,
            screen: Screen::Title,
            t: 0.0,
            screen_t: 0.0,
            fade: 1.0,
            heroes,
            chef,
            hero_sel,
            level_sel: 0,
            level_pos: 0.0,
            hero_pos: hero_sel as f32,
            round: None,
            scene: None,
            ticket_seed: 1,
            ticket_goals: Vec::new(),
            fx: Fx::default(),
            paused: None,
            summary: None,
            album_book: 0,
            album_sel: 0,
            album_open: None,
            album_back: Screen::Title,
            autopilot: false,
            icon: false,
            shown_fullness: 0.0,
            shown_score: 0.0,
            seed: 0x2545_F491,
            labels: [controls.label(Button::A), controls.label(Button::B), controls.label(Button::X)],
            stats_on: std::env::var("OVEREATEN_STATS").is_ok_and(|v| v == "1"),
            stats: Stats::new(),
            portraits,
            gl,
            bump_score: 0.0,
            music: false,
            save,
        };
        // Start on the first room still waiting for its stars.
        app.level_sel = (0..LEVELS.len()).find(|&i| app.save.stars_of(i) == 0 && app.save.level_open(i)).unwrap_or(0);
        app.level_pos = app.level_sel as f32;
        app.go(Screen::Title);
        Ok(app)
    }

    pub fn tr(&self, t: Text) -> &'static str {
        t.get(self.lang)
    }

    fn next_seed(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453) ^ (self.t * 1000.0) as u32;
        self.seed.max(1)
    }

    /// Shows level `i` behind the screen, played by the autopilot.
    fn show_room(&mut self, level: usize, hero: usize) {
        let seed = self.next_seed();
        if self.scene.as_ref().is_none_or(|s| s.level != level) {
            self.scene = Some(Scene::build(level));
        }
        let mut r = Round::new(level, hero, seed);
        // Play a little, so the belt is busy and the visitor is somewhere.
        for _ in 0..90 {
            let c = r.autopilot();
            r.update(1.0 / 30.0, c);
        }
        r.events.clear();
        self.round = Some(r);
    }

    fn roll_ticket(&mut self) {
        self.ticket_seed = self.next_seed();
        let mut rng = self.ticket_seed;
        let def = &LEVELS[self.level_sel];
        self.ticket_goals = draw_goals(&def.main, def.pool, || {
            rng ^= rng << 13;
            rng ^= rng >> 17;
            rng ^= rng << 5;
            rng
        });
    }

    pub fn go(&mut self, s: Screen) {
        if s == Screen::Album && self.screen != Screen::Album {
            self.album_back = self.screen;
            self.album_open = None;
        }
        let from = self.screen;
        self.screen = s;
        self.screen_t = 0.0;
        self.fade = 1.0;
        match s {
            Screen::Title => {
                self.summary = None;
                self.show_room(0, 0);
            }
            Screen::Levels => {
                self.summary = None;
                self.roll_ticket();
                if from != Screen::Album || self.round.as_ref().is_none_or(|r| r.level != self.level_sel) {
                    self.show_room(self.level_sel, self.hero_sel);
                }
            }
            _ => {}
        }
    }

    pub fn start_meal(&mut self) {
        let level = self.level_sel;
        if self.scene.as_ref().is_none_or(|s| s.level != level) {
            self.scene = Some(Scene::build(level));
        }
        self.round = Some(Round::new(level, self.hero_sel, self.ticket_seed));
        self.fx.clear();
        self.shown_fullness = 0.0;
        self.shown_score = 0.0;
        self.paused = None;
        self.summary = None;
        crate::assets::play(|s| s.admission);
        let lang = self.lang;
        let main = LEVELS[level].main.text(lang);
        self.fx.banner(tx::DIG_IN.get(lang), &main, pal::MUSTARD, 2.4);
        self.save.hero = self.hero_sel;
        self.go(Screen::Dining);
    }

    /// Plays the title's and rooms' meals with the autopilot, round and round.
    fn update_backdrop(&mut self, dt: f32) {
        let restart = self.round.as_ref().is_some_and(|r| r.outcome.is_some() && r.outro > 3.0);
        if restart {
            let (level, hero) = self.round.as_ref().map(|r| (r.level, r.hero)).unwrap_or((0, 0));
            self.show_room(level, hero);
        }
        if let Some(r) = &mut self.round {
            let c = r.autopilot();
            r.update(dt, c);
            r.events.clear();
        }
    }

    pub fn update(&mut self, dt: f32, c: &Controls) {
        self.t += dt;
        self.screen_t += dt;
        self.fade = (self.fade - dt * 4.0).max(0.0);
        self.bump_score = (self.bump_score - dt * 4.0).max(0.0);
        self.fx.update(dt);
        self.stats.update(dt);
        if c.pressed(Button::Y) {
            self.stats_on = !self.stats_on;
        }
        match self.screen {
            Screen::Title => {
                self.update_backdrop(dt);
                if c.pressed(Button::A) {
                    self.go(Screen::Heroes);
                } else if c.pressed(Button::X) {
                    self.go(Screen::Album);
                } else if c.pressed(Button::Select) {
                    self.toggle_lang();
                }
            }
            Screen::Heroes => {
                if c.nav(Button::Left) && self.hero_sel > 0 {
                    self.hero_sel -= 1;
                }
                if c.nav(Button::Right) && self.hero_sel + 1 < HEROES.len() {
                    self.hero_sel += 1;
                }
                if c.pressed(Button::A) && self.save.hero_unlocked(self.hero_sel) {
                    self.save.hero = self.hero_sel;
                    self.save.store();
                    self.go(Screen::Levels);
                } else if c.pressed(Button::B) {
                    self.go(Screen::Title);
                } else if c.pressed(Button::X) {
                    self.go(Screen::Album);
                } else if c.pressed(Button::Select) {
                    self.toggle_lang();
                }
            }
            Screen::Levels => {
                self.update_backdrop(dt);
                let before = self.level_sel;
                if c.nav(Button::Left) && self.level_sel > 0 {
                    self.level_sel -= 1;
                }
                if c.nav(Button::Right) && self.level_sel + 1 < LEVELS.len() {
                    self.level_sel += 1;
                }
                if before != self.level_sel {
                    self.roll_ticket();
                    self.show_room(self.level_sel, self.hero_sel);
                }
                if c.pressed(Button::A) && self.save.level_open(self.level_sel) {
                    self.start_meal();
                } else if c.pressed(Button::B) {
                    self.go(Screen::Heroes);
                } else if c.pressed(Button::X) {
                    self.go(Screen::Album);
                } else if c.pressed(Button::Select) {
                    self.toggle_lang();
                }
            }
            Screen::Dining => self.update_dining(dt, c),
            Screen::Result => self.update_result(dt, c),
            Screen::Album => self.update_album(dt, c),
        }
        self.level_pos += (self.level_sel as f32 - self.level_pos) * (1.0 - (-dt * 12.0).exp());
        self.hero_pos += (self.hero_sel as f32 - self.hero_pos) * (1.0 - (-dt * 10.0).exp());
        // The visitors move to the meal on screen.
        if let (Some(r), Some(sc)) = (&self.round, &mut self.scene) {
            sc.follow(r, dt);
        }
        let shown = self.round.as_ref().map(|r| (r.hero, scene::act_of(r), scene::belly_of(r)));
        for (i, h) in self.heroes.iter_mut().enumerate() {
            match shown {
                Some((hero, act, belly)) if hero == i && self.screen != Screen::Heroes => h.update(dt, act, belly),
                _ if self.screen == Screen::Heroes => {
                    let act = if i == self.hero_sel && self.save.hero_unlocked(i) { Act::Cheer } else { Act::Idle };
                    h.update(dt, act, 0.0);
                }
                _ => {}
            }
        }
        if let Some(c) = &mut self.chef {
            let since = self.round.as_ref().map_or(9.0, |r| r.t - r.last_served);
            let act = if since < 0.8 { Act::Eat(since / 0.8) } else { Act::Idle };
            c.update(dt, act, 0.0);
        }
        self.update_music();
    }

    fn update_music(&mut self) {
        let Some(m) = crate::assets::get().music else { return };
        let want = matches!(self.screen, Screen::Dining | Screen::Title | Screen::Levels) && self.paused.is_none();
        unsafe {
            if want && !self.music {
                raylib_sys::PlayMusicStream(m);
            } else if !want && self.music {
                raylib_sys::PauseMusicStream(m);
            }
            if want {
                raylib_sys::UpdateMusicStream(m);
            }
        }
        self.music = want;
    }

    fn toggle_lang(&mut self) {
        self.lang = if self.lang == Lang::Zh { Lang::En } else { Lang::Zh };
        self.save.lang = if self.lang == Lang::Zh { "zh" } else { "en" }.into();
        self.save.store();
    }

    /// The pause menu's entries.
    pub fn pause_items(&self) -> Vec<Text> {
        let mut v = vec![tx::CONTINUE];
        if self.round.as_ref().is_some_and(|r| r.can_leave()) {
            v.push(tx::SETTLE);
        }
        v.push(tx::LEAVE);
        v
    }

    /// A point on the canvas above floor point `p` at height `h` centimetres.
    pub fn on_canvas(&self, p: V2, h: f32) -> (f32, f32) {
        self.scene.as_ref().and_then(|s| s.project(scene::w(p, h))).unwrap_or((320.0, 240.0))
    }

    fn update_dining(&mut self, dt: f32, c: &Controls) {
        let lang = self.lang;
        if let Some(sel) = self.paused {
            let items = self.pause_items();
            let n = items.len();
            if c.nav(Button::Up) {
                self.paused = Some((sel + n - 1) % n);
            }
            if c.nav(Button::Down) {
                self.paused = Some((sel + 1) % n);
            }
            let sel = self.paused.unwrap_or(0);
            if c.pressed(Button::Start) || c.pressed(Button::B) || (c.pressed(Button::A) && items[sel] == tx::CONTINUE) {
                self.paused = None;
            } else if c.pressed(Button::A) && items[sel] == tx::SETTLE {
                self.paused = None;
                if let Some(r) = &mut self.round {
                    r.close();
                }
            } else if c.pressed(Button::A) {
                self.paused = None;
                self.fx.clear();
                self.go(Screen::Levels);
            }
            return;
        }
        let Some(round) = &mut self.round else { return };
        if c.pressed(Button::Start) && round.outcome.is_none() {
            self.paused = Some(0);
            return;
        }
        let controls = if self.autopilot {
            round.autopilot()
        } else {
            let (mx, my) = c.movement();
            crate::game::Controls { mx, my, eat: c.pressed(Button::A) }
        };
        round.update(dt, controls);
        let k = 1.0 - (-dt * 8.0).exp();
        self.shown_fullness += (round.fullness - self.shown_fullness) * k;
        self.shown_score += (round.score() as f32 - self.shown_score) * k;
        if (round.score() as f32 - self.shown_score).abs() < 0.5 {
            self.shown_score = round.score() as f32;
        }
        let events: Vec<Event> = std::mem::take(&mut round.events);
        let outro = round.outcome.map(|o| (o, round.outro));
        let me = round.player.pos;
        let floor = round.layout.floor_at(me);
        let stuffed = round.fullness > RULES.capacity;
        let (hx, hy) = self.on_canvas(me, floor + 170.0);
        if stuffed && ((self.t * 1.3).fract() < dt * 1.3) {
            self.fx.sweat(hx + 12.0, hy + 6.0);
        }
        for e in events {
            match e {
                Event::Served { belt, golden } => {
                    let Some(r) = &self.round else { break };
                    let b = r.layout.belts[belt];
                    let (x, y) = self.on_canvas(b.point(b.serve_s()).0, b.top() + 10.0);
                    if golden {
                        self.fx.banner(tx::GOLDEN.get(lang), tx::GOLDEN_SUB.get(lang), rgb(0xF4C430), 1.8);
                        self.fx.sparkles(x, y, rgb(0xFFE08A), 16, 26.0);
                    } else {
                        self.fx.puffs(x, y, 2);
                    }
                }
                Event::Ate { dish, points, combo, at } => {
                    let (x, y) = self.on_canvas(at, floor + 150.0);
                    self.fx.crumbs(x, y, rgb(DISHES[dish].crumb), 7);
                    self.fx.crumbs(x, y, rgb(0xFFFDF5), 4);
                    crate::assets::play(|s| s.bite);
                    self.fx.float(x, y - 24.0, format!("+{points}"), pal::MUSTARD, 24.0);
                    if combo >= 2 {
                        let m = (1.0 + RULES.combo_step * (combo - 1) as f32).min(RULES.combo_max);
                        self.fx.float(x, y - 50.0, format!("{} {combo} ×{m:.2}", tx::COMBO.get(lang)), pal::BLUSH, 17.0);
                    }
                    if dish == GOLDEN {
                        self.fx.hearts(x, y, 5);
                        self.fx.sparkles(x, y, rgb(0xFFE08A), 14, 24.0);
                    }
                    self.bump_score = 1.0;
                    self.fx.bump_plates = 1.0;
                    self.fx.shake(0.15);
                }
                Event::Full => self.fx.float(hx, hy - 20.0, tx::FULL.get(lang).into(), rgb(0xFFB45E), 20.0),
                Event::Danger => {
                    crate::assets::play(|s| s.red);
                    self.fx.banner(tx::DANGER.get(lang), tx::DANGER_SUB.get(lang), rgb(0xFF6B6B), 2.0);
                    self.fx.shake(0.6);
                }
                Event::Recovered => {
                    crate::assets::play(|s| s.recover);
                    self.fx.float(hx, hy - 20.0, tx::PHEW.get(lang).into(), pal::MINT, 20.0);
                }
                Event::MainDone => {
                    crate::assets::play(|s| s.card_reveal);
                    self.fx.banner(tx::TASK_DONE.get(lang), tx::TASK_DONE_SUB.get(lang), pal::MINT, 2.6);
                    self.fx.sparkles(hx, hy, pal::MINT, 20, 50.0);
                }
                Event::GoalDone(_) => {
                    self.fx.float(hx, hy - 44.0, tx::fill(tx::GOAL_DONE.get(lang), RULES.bonus), pal::MINT, 18.0);
                    self.bump_score = 1.0;
                }
                Event::GoalFailed(_) => {
                    self.fx.float(hx, hy - 44.0, tx::GOAL_FAILED.get(lang).into(), rgb(0xFF8A80), 16.0);
                }
                Event::Collapse => {
                    self.fx.banner(tx::COLLAPSE.get(lang), "", rgb(0xE9E2C8), 1.4);
                    self.fx.shake(0.5);
                }
                Event::Lean(_) => {
                    self.fx.float(320.0, 120.0, tx::LEAN.get(lang).into(), rgb(0x9FD8FF), 20.0);
                }
                Event::Outcome(o) => {
                    crate::assets::play(|s| s.closing);
                    let (title, sub, color) = match o {
                        Outcome::Success => (tx::SUCCESS, tx::SUCCESS_SUB, pal::MUSTARD),
                        Outcome::Closed => (tx::CLOSED, tx::CLOSED_SUB, pal::CREAM),
                        Outcome::Broke => (tx::BROKE, tx::BROKE_SUB, rgb(0xFF9A6B)),
                        Outcome::Hospital => (tx::HOSPITAL, tx::HOSPITAL_SUB, rgb(0xFF6B6B)),
                    };
                    self.fx.banner(title.get(lang), sub.get(lang), color, 2.4);
                    if o == Outcome::Success {
                        self.fx.sparkles(hx, hy, pal::MUSTARD, 24, 60.0);
                        self.fx.hearts(hx, hy - 20.0, 6);
                    } else {
                        self.fx.shake(0.5);
                    }
                }
            }
        }
        if let Some((_, t)) = outro {
            if t > 2.6 {
                self.finish_meal();
            }
        }
    }

    fn finish_meal(&mut self) {
        let Some(round) = &self.round else { return };
        let level = round.level;
        let id = LEVELS[level].id;
        let earned = round.cards_earned();
        let new_cards: Vec<usize> =
            earned.iter().filter_map(|id| card_by_id(id)).filter(|&i| !self.save.has(CARDS[i].id)).collect();
        let unlocked_before: Vec<bool> = (0..HEROES.len()).map(|i| self.save.hero_unlocked(i)).collect();
        for &i in &new_cards {
            self.save.cards.insert(CARDS[i].id.to_string());
        }
        let outcome = round.outcome.unwrap_or(Outcome::Closed);
        let stars = round.stars();
        let best_before = self.save.best_of(level);
        let score = if outcome == Outcome::Hospital { 0 } else { round.score() };
        if outcome.won() {
            let s = self.save.stars.entry(id.to_string()).or_insert(0);
            *s = (*s).max(stars);
            let b = self.save.best.entry(id.to_string()).or_insert(0);
            *b = (*b).max(score);
        }
        if outcome == Outcome::Hospital {
            self.save.hospital += 1;
        }
        self.save.meals += 1;
        self.save.store();
        let new_heroes = (0..HEROES.len()).filter(|&i| !unlocked_before[i] && self.save.hero_unlocked(i)).collect();
        let food = round.food_points();
        self.summary = Some(Summary {
            level,
            outcome,
            score,
            food,
            combo_bonus: round.tally.score - food,
            goals: round.goals.iter().copied().zip(round.goal_state.iter().copied()).collect(),
            bonus: round.bonus,
            stars,
            best_before,
            plates: round.tally.plates,
            peak: round.tally.max_fullness,
            new_cards,
            new_heroes,
            t: 0.0,
        });
        self.fx.clear();
        self.go(Screen::Result);
    }

    fn update_result(&mut self, dt: f32, c: &Controls) {
        if let Some(r) = &mut self.round {
            r.update(dt, crate::game::Controls::default());
            r.events.clear();
        }
        let Some(s) = &mut self.summary else { return };
        let before = s.t;
        s.t += dt;
        let after = s.t;
        let cards = s.new_cards.clone();
        for (i, &card) in cards.iter().enumerate() {
            let turn = Summary::reveal_at(i) + 0.25;
            let flip = Summary::reveal_at(i) + 0.45;
            if before < turn && after >= turn {
                crate::assets::play(|s| s.card_flip);
            }
            if before < flip && after >= flip {
                crate::assets::play(|s| s.card_reveal);
                let (x, y) = crate::screens::rack_slot(i, cards.len());
                let (w, h, _) = crate::screens::rack_layout(cards.len());
                let rare = CARDS[card].rarity >= Rarity::Rare;
                self.fx.sparkles(x + w / 2.0, y + h / 2.0, if rare { pal::GOLD } else { pal::CREAM }, if rare { 22 } else { 10 }, 44.0);
            }
        }
        for k in 0..3u32 {
            let at = 0.6 + k as f32 * 0.3;
            if before < at && after >= at && k < self.summary.as_ref().map_or(0, |s| s.stars) {
                crate::assets::play(|s| s.card_flip);
                self.fx.sparkles(560.0 + k as f32 * 0.0, 60.0, pal::GOLD, 8, 20.0);
            }
        }
        let s = self.summary.as_mut().unwrap();
        if c.pressed(Button::A) {
            if s.all_revealed() {
                self.level_sel = s.level;
                self.roll_ticket();
                self.start_meal();
            } else {
                s.t = Summary::reveal_at(s.new_cards.len()) + 0.4;
            }
        } else if c.pressed(Button::B) {
            let lv = s.level;
            let won = s.outcome.won();
            // After a first clear, offer the next room.
            self.level_sel = if won && lv + 1 < LEVELS.len() && self.save.level_open(lv + 1) && self.save.stars_of(lv + 1) == 0 {
                lv + 1
            } else {
                lv
            };
            self.level_pos = self.level_sel as f32;
            self.go(Screen::Levels);
        } else if c.pressed(Button::X) {
            self.go(Screen::Album);
        }
    }

    /// Cards of the open book, by index into `CARDS`.
    pub fn book_cards(&self) -> Vec<usize> {
        let book = BOOKS[self.album_book];
        (0..CARDS.len()).filter(|&i| CARDS[i].book == book).collect()
    }

    /// Columns of the open book's grid.
    pub fn book_cols(&self) -> usize {
        match BOOKS[self.album_book] {
            Book::Levels => 8,
            Book::Heroes => 6,
            _ => 4,
        }
    }

    fn update_album(&mut self, dt: f32, c: &Controls) {
        if let Some(t) = &mut self.album_open {
            *t += dt;
            if c.pressed(Button::A) || c.pressed(Button::B) {
                self.album_open = None;
            }
            return;
        }
        let n = self.book_cards().len();
        let cols = self.book_cols();
        if c.pressed(Button::L1) || c.pressed(Button::R1) {
            let d = if c.pressed(Button::L1) { BOOKS.len() - 1 } else { 1 };
            self.album_book = (self.album_book + d) % BOOKS.len();
            self.album_sel = 0;
            return;
        }
        if c.nav(Button::Left) {
            if self.album_sel % cols == 0 {
                self.album_book = (self.album_book + BOOKS.len() - 1) % BOOKS.len();
                self.album_sel = 0;
                return;
            }
            self.album_sel -= 1;
        }
        if c.nav(Button::Right) {
            if self.album_sel + 1 >= n || (self.album_sel + 1) % cols == 0 {
                self.album_book = (self.album_book + 1) % BOOKS.len();
                self.album_sel = 0;
                return;
            }
            self.album_sel += 1;
        }
        if c.nav(Button::Down) && self.album_sel + cols < n {
            self.album_sel += cols;
        }
        if c.nav(Button::Up) && self.album_sel >= cols {
            self.album_sel -= cols;
        }
        if c.pressed(Button::A) {
            self.album_open = Some(0.0);
        } else if c.pressed(Button::B) {
            let back = self.album_back;
            self.go(back);
        } else if c.pressed(Button::Select) {
            self.toggle_lang();
        }
    }

    /// The card open in the album, if any.
    pub fn open_card(&self) -> Option<usize> {
        self.album_open.and_then(|_| self.book_cards().get(self.album_sel).copied())
    }

    /// Renders textures needed this frame; call before drawing starts.
    pub fn prepare(&mut self) {
        if self.screen == Screen::Album {
            if let (Some(i), Some(t)) = (self.open_card(), self.album_open) {
                if self.save.has(CARDS[i].id) {
                    let turn = (t * 0.9).sin() * 0.9 + t * 0.3;
                    self.portraits.render_detail(&self.gl, &self.heroes, CARDS[i].art, turn, self.t);
                }
            }
        }
    }

    /// For captures: plays a meal with the autopilot to `at` seconds, or to its end.
    pub fn fast_forward(&mut self, at: f32, c: &Controls) {
        let auto = self.autopilot;
        self.autopilot = true;
        let dt = 1.0 / 60.0;
        while self.screen == Screen::Dining && self.round.as_ref().is_some_and(|r| r.t < at && r.outcome.is_none()) {
            self.update(dt, c);
        }
        self.autopilot = auto;
    }
}

/// Simplified Chinese unless Gizlet asks for English.
fn default_lang() -> Lang {
    let var = std::env::var("GIZLET_LANG").unwrap_or_default();
    if var.to_ascii_lowercase().starts_with("en") {
        Lang::En
    } else {
        Lang::Zh
    }
}
