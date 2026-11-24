//! Screens and what moves between them: the title, the street of
//! restaurants, a meal, its bill and the card album.

use crate::art::pal;
use crate::fx::Fx;
use crate::game::{self, belt_point, Event, Outcome, Round};
use crate::gfx::rgb;
use crate::input::{Button, Controls};
use crate::rules::*;
use crate::save::Save;
use crate::text as tx;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Title,
    Street,
    Dining,
    Result,
    Album,
}

/// The bill of a finished meal.
pub struct Summary {
    pub outcome: Outcome,
    pub coins: i32,
    pub plates: u32,
    pub peak: f32,
    pub eaten: [u32; KINDS],
    /// Cards this meal earned for the first time.
    pub new_cards: Vec<usize>,
    /// Seconds since the bill came.
    pub t: f32,
}

impl Summary {
    /// When card `i` of the rack starts turning over.
    pub fn reveal_at(i: usize) -> f32 {
        0.9 + i as f32 * 0.55
    }

    pub fn all_revealed(&self) -> bool {
        self.t >= Summary::reveal_at(self.new_cards.len()) + 0.2
    }
}

pub struct App {
    pub lang: Lang,
    pub screen: Screen,
    pub t: f32,
    pub screen_t: f32,
    /// 1 right after a screen change, fading to 0.
    pub fade: f32,
    pub street_sel: usize,
    /// Eases toward `street_sel`, for sliding the tickets.
    pub street_pos: f32,
    pub round: Option<Round>,
    pub fx: Fx,
    pub save: Save,
    pub paused: Option<usize>,
    pub summary: Option<Summary>,
    pub album_sel: usize,
    /// Seconds the selected card has been open, while open.
    pub album_open: Option<f32>,
    album_back: Screen,
    pub autopilot: bool,
    /// Draws the app icon instead of a screen, for `OVEREATEN_SCENE=icon`.
    pub icon: bool,
    /// The gauge and counters ease toward the real numbers.
    pub shown_fullness: f32,
    pub shown_coins: f32,
    seed: u32,
    /// Labels of the A, B and X buttons after the A/B swap.
    pub labels: [&'static str; 3],
}

impl App {
    pub fn new(save: Save, controls: &Controls) -> App {
        let lang = match save.lang.as_str() {
            "en" => Lang::En,
            "zh" => Lang::Zh,
            _ => default_lang(),
        };
        App {
            lang,
            screen: Screen::Title,
            t: 0.0,
            screen_t: 0.0,
            fade: 1.0,
            street_sel: 0,
            street_pos: 0.0,
            round: None,
            fx: Fx::default(),
            save,
            paused: None,
            summary: None,
            album_sel: 0,
            album_open: None,
            album_back: Screen::Title,
            autopilot: false,
            icon: false,
            shown_fullness: 0.0,
            shown_coins: 0.0,
            seed: 0x2545_F491,
            labels: [controls.label(Button::A), controls.label(Button::B), controls.label(Button::X)],
        }
    }

    pub fn tr(&self, t: Text) -> &'static str {
        t.get(self.lang)
    }

    pub fn go(&mut self, s: Screen) {
        if s == Screen::Album && self.screen != Screen::Album {
            self.album_back = self.screen;
            self.album_open = None;
        }
        self.screen = s;
        self.screen_t = 0.0;
        self.fade = 1.0;
    }

    pub fn start_meal(&mut self) {
        self.seed = self.seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453) ^ (self.t * 1000.0) as u32;
        self.round = Some(Round::new(self.seed));
        self.fx.clear();
        self.shown_fullness = 0.0;
        self.shown_coins = 0.0;
        self.paused = None;
        self.summary = None;
        let lang = self.lang;
        crate::assets::play(|s| s.admission);
        self.fx.banner(tx::DIG_IN.get(lang), RESTAURANTS[0].task.get(lang), pal::MUSTARD, 2.2);
        self.go(Screen::Dining);
    }

    pub fn update(&mut self, dt: f32, c: &Controls) {
        self.t += dt;
        self.screen_t += dt;
        self.fade = (self.fade - dt * 4.0).max(0.0);
        self.fx.update(dt);
        match self.screen {
            Screen::Title => {
                if c.pressed(Button::A) {
                    self.go(Screen::Street);
                } else if c.pressed(Button::X) {
                    self.go(Screen::Album);
                } else if c.pressed(Button::Select) {
                    self.toggle_lang();
                }
            }
            Screen::Street => {
                if c.nav(Button::Left) && self.street_sel > 0 {
                    self.street_sel -= 1;
                }
                if c.nav(Button::Right) && self.street_sel + 1 < RESTAURANTS.len() {
                    self.street_sel += 1;
                }
                if c.pressed(Button::A) && RESTAURANTS[self.street_sel].open {
                    self.start_meal();
                } else if c.pressed(Button::B) {
                    self.go(Screen::Title);
                } else if c.pressed(Button::X) {
                    self.go(Screen::Album);
                } else if c.pressed(Button::Select) {
                    self.toggle_lang();
                }
            }
            Screen::Dining => self.update_dining(dt, c),
            Screen::Result => {
                let Some(s) = &mut self.summary else { return };
                let before = s.t;
                s.t += dt;
                let after = s.t;
                for (i, &card) in s.new_cards.clone().iter().enumerate() {
                    let flip = Summary::reveal_at(i) + 0.4;
                    let turn = Summary::reveal_at(i) + 0.25;
                    if before < turn && after >= turn {
                        crate::assets::play(|s| s.card_flip);
                    }
                    if before < flip && after >= flip {
                        crate::assets::play(|s| s.card_reveal);
                        let (x, y) = crate::screens::rack_slot(i, s.new_cards.len());
                        self.fx.sparkles(x + 48.0, y + 67.0, rgb(CARDS[card].band), 12, 40.0);
                    }
                }
                let s = self.summary.as_mut().unwrap();
                if c.pressed(Button::A) {
                    if s.all_revealed() {
                        self.start_meal();
                    } else {
                        s.t = Summary::reveal_at(s.new_cards.len()) + 0.3;
                    }
                } else if c.pressed(Button::B) {
                    self.go(Screen::Street);
                } else if c.pressed(Button::X) {
                    self.go(Screen::Album);
                }
            }
            Screen::Album => {
                if let Some(t) = &mut self.album_open {
                    *t += dt;
                    if c.pressed(Button::A) || c.pressed(Button::B) {
                        self.album_open = None;
                    }
                    return;
                }
                let n = CARDS.len();
                let row = |i: usize| if i < 4 { 0 } else { 1 };
                if c.nav(Button::Left) && self.album_sel > 0 {
                    self.album_sel -= 1;
                }
                if c.nav(Button::Right) && self.album_sel + 1 < n {
                    self.album_sel += 1;
                }
                if c.nav(Button::Down) && row(self.album_sel) == 0 {
                    self.album_sel = (self.album_sel + 4).min(n - 1);
                }
                if c.nav(Button::Up) && row(self.album_sel) == 1 {
                    self.album_sel -= 4;
                }
                if c.pressed(Button::A) {
                    self.album_open = Some(0.0);
                } else if c.pressed(Button::B) {
                    let back = self.album_back;
                    self.go(back);
                }
            }
        }
        self.street_pos += (self.street_sel as f32 - self.street_pos) * (1.0 - (-dt * 12.0).exp());
    }

    fn toggle_lang(&mut self) {
        self.lang = if self.lang == Lang::Zh { Lang::En } else { Lang::Zh };
        self.save.lang = if self.lang == Lang::Zh { "zh" } else { "en" }.into();
        self.save.store();
    }

    fn update_dining(&mut self, dt: f32, c: &Controls) {
        let lang = self.lang;
        if let Some(sel) = &mut self.paused {
            if c.nav(Button::Up) || c.nav(Button::Down) {
                *sel = 1 - *sel;
            }
            if c.pressed(Button::Start) || c.pressed(Button::B) || (c.pressed(Button::A) && *sel == 0) {
                self.paused = None;
            } else if c.pressed(Button::A) {
                self.paused = None;
                self.round = None;
                self.fx.clear();
                self.go(Screen::Street);
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
            game::Controls { mx, my, eat: c.pressed(Button::A) }
        };
        round.update(dt, controls);
        let k = 1.0 - (-dt * 8.0).exp();
        self.shown_fullness += (round.fullness - self.shown_fullness) * k;
        self.shown_coins += (round.coins as f32 - self.shown_coins) * k;
        if (round.coins as f32 - self.shown_coins).abs() < 0.5 {
            self.shown_coins = round.coins as f32;
        }

        // Stomach rumbles: an occasional sweat drop when stuffed.
        let p = {
            let (x, y) = crate::art::canvas(round.player.pos, 0.0);
            crate::game::V2::new(x, y + 20.0)
        };
        if round.fullness > RULES.capacity && ((self.t * 1.3).fract() < dt * 1.3) {
            self.fx.sweat(p.x + 14.0, p.y - 76.0);
        }

        let events: Vec<Event> = std::mem::take(&mut round.events);
        let outro = round.outcome.map(|o| (o, round.outro));
        for e in events {
            match e {
                Event::Served { golden } => {
                    let (x, y) = crate::art::canvas(belt_point(game::floor::SERVE_S).0, game::floor::COUNTER_H);
                    let at = crate::game::V2::new(x, y);
                    if golden {
                        self.fx.banner(tx::GOLDEN.get(lang), tx::GOLDEN_SUB.get(lang), rgb(0xF4C430), 1.8);
                        self.fx.sparkles(at.x, at.y, rgb(0xFFE08A), 16, 26.0);
                    } else {
                        self.fx.puffs(at.x, at.y, 3);
                    }
                }
                Event::Ate { dish, coins, combo, at } => {
                    let (x, y) = crate::art::canvas(at, 0.0);
                    let at = crate::game::V2::new(x, y - 50.0);
                    self.fx.crumbs(at.x, at.y + 12.0, rgb(DISHES[dish].crumb), 7);
                    self.fx.crumbs(at.x, at.y + 12.0, rgb(0xFFFDF5), 4);
                    crate::assets::play(|s| s.bite);
                    self.fx.float(at.x, at.y - 10.0, format!("+{coins}"), pal::MUSTARD, 24.0);
                    if combo >= 2 {
                        self.fx.float(
                            at.x,
                            at.y - 34.0,
                            format!("{} ×{combo}", tx::COMBO.get(lang)),
                            rgb(0xFF8FA3),
                            18.0,
                        );
                    }
                    if dish == GOLDEN {
                        self.fx.hearts(at.x, at.y, 5);
                        self.fx.sparkles(at.x, at.y, rgb(0xFFE08A), 14, 24.0);
                    }
                    self.fx.bump_coins = 1.0;
                    self.fx.bump_plates = 1.0;
                    self.fx.shake(0.18);
                }
                Event::Full => {
                    self.fx.float(p.x, p.y - 96.0, tx::FULL.get(lang).into(), rgb(0xFFB45E), 20.0);
                }
                Event::Danger => {
                    crate::assets::play(|s| s.red);
                    self.fx.banner(tx::DANGER.get(lang), tx::DANGER_SUB.get(lang), rgb(0xFF6B6B), 2.0);
                    self.fx.shake(0.6);
                }
                Event::Recovered => {
                    crate::assets::play(|s| s.recover);
                    self.fx.float(p.x, p.y - 96.0, tx::PHEW.get(lang).into(), pal::MINT, 20.0);
                }
                Event::Outcome(o) => {
                    crate::assets::play(|s| s.closing);
                    let (title, sub, color) = match o {
                        Outcome::Done => (tx::DONE, tx::DONE_SUB, pal::MUSTARD),
                        Outcome::Closed => (tx::CLOSED, tx::CLOSED_SUB, pal::CREAM),
                        Outcome::Hospital => (tx::HOSPITAL, tx::HOSPITAL_SUB, rgb(0xFF6B6B)),
                    };
                    self.fx.banner(title.get(lang), sub.get(lang), color, 2.4);
                    if o == Outcome::Done {
                        self.fx.sparkles(p.x, p.y - 50.0, pal::MUSTARD, 24, 60.0);
                        self.fx.hearts(p.x, p.y - 70.0, 6);
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
        let Some(round) = self.round.take() else { return };
        let earned = round.cards_earned();
        let new_cards: Vec<usize> = earned.iter().copied().filter(|&i| !self.save.has(CARDS[i].id)).collect();
        for &i in &new_cards {
            self.save.cards.insert(CARDS[i].id.to_string());
        }
        self.save.meals += 1;
        self.save.best = self.save.best.max(round.coins);
        self.save.store();
        self.summary = Some(Summary {
            outcome: round.outcome.unwrap_or(Outcome::Closed),
            coins: round.coins,
            plates: round.plates_eaten,
            peak: round.max_fullness,
            eaten: round.eaten,
            new_cards,
            t: 0.0,
        });
        self.fx.clear();
        self.go(Screen::Result);
    }

    /// For captures: plays a meal with the autopilot to `at` seconds, or to its end.
    pub fn fast_forward(&mut self, at: f32, c: &Controls) {
        let auto = self.autopilot;
        self.autopilot = true;
        let dt = 1.0 / 60.0;
        while self.screen == Screen::Dining && self.round.as_ref().is_some_and(|r| r.t < at) {
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
