//! Tasks: each room's main task, which decides the meal, and the side goals
//! drawn for it, which add points. Ten kinds test different things: eating a
//! lot, eating everything, eating in order, holding back, picking well,
//! timing, ending at the right fullness, daring the red line, moving about
//! the room and chaining dishes.

use crate::rules::{Hue, Lang, Text, DISHES, EVERYDAY, GOLDEN, KINDS};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Goal {
    /// Score this many points.
    Score(i32),
    /// Eat each everyday dish once.
    Collect,
    /// Eat red, white, yellow in order, this many rounds.
    Sequence(u32),
    /// Score this much within this many plates.
    Efficiency { score: i32, plates: u32 },
    /// Eat this many golden plates within `within` seconds of their serving.
    Golden { count: u32, within: f32 },
    /// Score this much without eating the same dish twice in a row.
    NoRepeat { score: i32 },
    /// End the meal between `lo` and `hi` percent full, with at least this many plates.
    Precision { lo: f32, hi: f32, plates: u32 },
    /// Spend `secs` above the red line in all and score this much.
    Risk { secs: f32, score: i32 },
    /// Eat this many plates.
    Plates(u32),
    /// Eat this many different dishes.
    Kinds(u32),
    /// Eat a dish of each colour.
    Colors,
    /// Reach this combo.
    Combo(u32),
    /// Never pass 100%.
    Light,
    /// Never eat this dish.
    Avoid(usize),
    /// Eat a plate in each quarter of the room.
    Zones,
    /// Finish the main task within this many seconds.
    Quick(f32),
    /// End the meal at least this full.
    Full(f32),
    /// Spend this many seconds above the red line in all.
    Brave(f32),
    /// Eat this many golden plates.
    GoldenAny(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Quantity,
    Collect,
    Order,
    Restraint,
    Efficiency,
    Timing,
    Precision,
    Risk,
    Space,
    Combo,
}

impl Kind {
    pub fn name(self) -> Text {
        match self {
            Kind::Quantity => Text::new("数量", "Quantity"),
            Kind::Collect => Text::new("收集", "Collect"),
            Kind::Order => Text::new("顺序", "Order"),
            Kind::Restraint => Text::new("限制", "Restraint"),
            Kind::Efficiency => Text::new("效率", "Efficiency"),
            Kind::Timing => Text::new("时机", "Timing"),
            Kind::Precision => Text::new("精准", "Precision"),
            Kind::Risk => Text::new("风险", "Risk"),
            Kind::Space => Text::new("空间", "Space"),
            Kind::Combo => Text::new("连击", "Combo"),
        }
    }

    pub fn color(self) -> u32 {
        match self {
            Kind::Quantity => 0xF2B134,
            Kind::Collect => 0x7FC8A9,
            Kind::Order => 0x4A90C2,
            Kind::Restraint => 0x9BC1BC,
            Kind::Efficiency => 0xE4572E,
            Kind::Timing => 0xFFC93C,
            Kind::Precision => 0x8E7CC3,
            Kind::Risk => 0xD7263D,
            Kind::Space => 0x5FA8A0,
            Kind::Combo => 0xFF8FA3,
        }
    }
}

/// The colours of the sequence, in order.
pub const SEQUENCE: [Hue; 3] = [Hue::Red, Hue::White, Hue::Yellow];

/// Everything about a meal the goals look at.
#[derive(Clone, Debug, Default)]
pub struct Tally {
    pub t: f32,
    /// Points from food, combos included.
    pub score: i32,
    pub plates: u32,
    pub eaten: [u32; KINDS],
    pub fullness: f32,
    pub max_fullness: f32,
    pub max_combo: u32,
    /// Golden plates eaten, and those eaten quickly enough for `Golden`.
    pub golden: u32,
    pub golden_quick: u32,
    /// Correct steps of the colour sequence.
    pub seq: u32,
    /// The same dish was eaten twice in a row.
    pub repeated: bool,
    /// Seconds spent above the red line in all.
    pub red_secs: f32,
    /// Quarters of the room eaten in, as bits.
    pub zones: u8,
    /// When the main task was done.
    pub main_done: Option<f32>,
}

impl Tally {
    pub fn kinds(&self) -> u32 {
        self.eaten[..EVERYDAY].iter().filter(|&&n| n > 0).count() as u32 + (self.eaten[GOLDEN] > 0) as u32
    }

    pub fn colors(&self) -> u32 {
        [Hue::Red, Hue::White, Hue::Yellow]
            .iter()
            .filter(|&&h| DISHES.iter().enumerate().any(|(i, d)| d.hue == h && self.eaten[i] > 0))
            .count() as u32
    }

    /// Counts a bite of `dish` toward the colour sequence.
    pub fn step_sequence(&mut self, dish: usize) {
        let hue = DISHES[dish].hue;
        if hue == Hue::Gold {
            return;
        }
        if hue == SEQUENCE[(self.seq % 3) as usize] {
            self.seq += 1;
        } else {
            self.seq -= self.seq % 3;
            if hue == SEQUENCE[0] {
                self.seq += 1;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Open,
    Done,
    Failed,
}

impl Goal {
    pub fn kind(&self) -> Kind {
        match self {
            Goal::Score(_) | Goal::Plates(_) => Kind::Quantity,
            Goal::Collect | Goal::Kinds(_) | Goal::Colors => Kind::Collect,
            Goal::Sequence(_) => Kind::Order,
            Goal::NoRepeat { .. } | Goal::Light | Goal::Avoid(_) => Kind::Restraint,
            Goal::Efficiency { .. } => Kind::Efficiency,
            Goal::Golden { .. } | Goal::GoldenAny(_) | Goal::Quick(_) => Kind::Timing,
            Goal::Precision { .. } | Goal::Full(_) => Kind::Precision,
            Goal::Risk { .. } | Goal::Brave(_) => Kind::Risk,
            Goal::Zones => Kind::Space,
            Goal::Combo(_) => Kind::Combo,
        }
    }

    /// Whether it is only decided when the meal ends.
    pub fn at_close(&self) -> bool {
        matches!(self, Goal::Precision { .. } | Goal::Full(_))
    }

    /// Where the goal stands. `closing` is true once the meal is over.
    pub fn state(&self, t: &Tally, closing: bool) -> State {
        use State::*;
        let done = |ok: bool| if ok { Done } else { Open };
        let s = match *self {
            Goal::Score(n) => done(t.score >= n),
            Goal::Collect => done(t.eaten[..EVERYDAY].iter().all(|&n| n > 0)),
            Goal::Sequence(n) => done(t.seq >= n * 3),
            Goal::Efficiency { score, plates } => {
                if t.score >= score && t.plates <= plates {
                    Done
                } else if t.plates >= plates {
                    Failed
                } else {
                    Open
                }
            }
            Goal::Golden { count, .. } => done(t.golden_quick >= count),
            Goal::NoRepeat { score } => {
                if t.repeated {
                    Failed
                } else {
                    done(t.score >= score)
                }
            }
            Goal::Precision { lo, hi, plates } => {
                if closing {
                    if t.fullness >= lo && t.fullness <= hi && t.plates >= plates {
                        Done
                    } else {
                        Failed
                    }
                } else {
                    Open
                }
            }
            Goal::Risk { secs, score } => done(t.red_secs >= secs && t.score >= score),
            Goal::Plates(n) => done(t.plates >= n),
            Goal::Kinds(n) => done(t.kinds() >= n),
            Goal::Colors => done(t.colors() >= 3),
            Goal::Combo(n) => done(t.max_combo >= n),
            Goal::Light => {
                if t.max_fullness > 100.0 {
                    Failed
                } else {
                    done(closing && t.plates > 0)
                }
            }
            Goal::Avoid(d) => {
                if t.eaten[d] > 0 {
                    Failed
                } else {
                    done(closing && t.plates > 0)
                }
            }
            Goal::Zones => done(t.zones == 0b1111),
            Goal::Quick(secs) => match t.main_done {
                Some(at) if at <= secs => Done,
                _ if t.t > secs => Failed,
                _ => Open,
            },
            Goal::Full(pct) => {
                if closing {
                    if t.fullness >= pct {
                        Done
                    } else {
                        Failed
                    }
                } else {
                    Open
                }
            }
            Goal::Brave(secs) => done(t.red_secs >= secs),
            Goal::GoldenAny(n) => done(t.golden >= n),
        };
        if s == Open && closing {
            Failed
        } else {
            s
        }
    }

    /// How far along it is, as (now, target), for the progress bar.
    pub fn progress(&self, t: &Tally) -> (f32, f32) {
        match *self {
            Goal::Score(n) => (t.score as f32, n as f32),
            Goal::Collect => (t.eaten[..EVERYDAY].iter().filter(|&&n| n > 0).count() as f32, EVERYDAY as f32),
            Goal::Sequence(n) => (t.seq as f32, (n * 3) as f32),
            Goal::Efficiency { score, .. } => (t.score as f32, score as f32),
            Goal::Golden { count, .. } => (t.golden_quick as f32, count as f32),
            Goal::NoRepeat { score } => (t.score as f32, score as f32),
            Goal::Precision { lo, .. } => (t.fullness.min(lo), lo),
            Goal::Risk { secs, score } => {
                let a = (t.red_secs / secs).min(1.0);
                let b = (t.score as f32 / score as f32).min(1.0);
                (a + b, 2.0)
            }
            Goal::Plates(n) => (t.plates as f32, n as f32),
            Goal::Kinds(n) => (t.kinds() as f32, n as f32),
            Goal::Colors => (t.colors() as f32, 3.0),
            Goal::Combo(n) => (t.max_combo as f32, n as f32),
            Goal::Light | Goal::Avoid(_) => (0.0, 1.0),
            Goal::Zones => (t.zones.count_ones() as f32, 4.0),
            Goal::Quick(_) => (if t.main_done.is_some() { 1.0 } else { 0.0 }, 1.0),
            Goal::Full(pct) => (t.fullness.min(pct), pct),
            Goal::Brave(secs) => (t.red_secs.min(secs), secs),
            Goal::GoldenAny(n) => (t.golden as f32, n as f32),
        }
    }

    /// A short line naming the goal.
    pub fn text(&self, lang: Lang) -> String {
        let zh = lang == Lang::Zh;
        match *self {
            Goal::Score(n) => if zh { format!("吃够 {n} 分") } else { format!("Score {n}") },
            Goal::Collect => if zh { "六种寿司各吃一盘".into() } else { "Eat each of the six sushi".into() },
            Goal::Sequence(n) => {
                if zh {
                    format!("按 红→白→黄 吃 {n} 轮")
                } else {
                    format!("Eat red → white → yellow, {n} rounds")
                }
            }
            Goal::Efficiency { score, plates } => {
                if zh {
                    format!("{plates} 盘以内吃够 {score} 分")
                } else {
                    format!("Score {score} within {plates} plates")
                }
            }
            Goal::Golden { count, within } => {
                if zh {
                    format!("金盘出来 {within:.0} 秒内吃掉，{count} 次")
                } else {
                    format!("Eat {count} golden plates within {within:.0} s of serving")
                }
            }
            Goal::NoRepeat { score } => {
                if zh {
                    format!("不连吃同一种，吃够 {score} 分")
                } else {
                    format!("Score {score}, never the same dish twice in a row")
                }
            }
            Goal::Precision { lo, hi, plates } => {
                if zh {
                    format!("打烊时饱腹 {lo:.0}%~{hi:.0}%（至少 {plates} 盘）")
                } else {
                    format!("Close at {lo:.0}-{hi:.0}% full ({plates}+ plates)")
                }
            }
            Goal::Risk { secs, score } => {
                if zh {
                    format!("红线以上累计 {secs:.0} 秒，并吃够 {score} 分")
                } else {
                    format!("{secs:.0} s above the red line in all, and score {score}")
                }
            }
            Goal::Plates(n) => if zh { format!("吃满 {n} 盘") } else { format!("Eat {n} plates") },
            Goal::Kinds(n) => if zh { format!("吃到 {n} 种不同的") } else { format!("Eat {n} different dishes") },
            Goal::Colors => if zh { "红、白、黄三色都吃到".into() } else { "Eat red, white and yellow".into() },
            Goal::Combo(n) => if zh { format!("连吃达到 {n} 连") } else { format!("Reach a {n}-dish combo") },
            Goal::Light => if zh { "全程不超过 100%".into() } else { "Never pass 100%".into() },
            Goal::Avoid(d) => {
                let name = DISHES[d].name.get(lang);
                if zh {
                    format!("一盘{name}都不吃")
                } else {
                    format!("No {name} at all")
                }
            }
            Goal::Zones => if zh { "在房间四个角落各吃一盘".into() } else { "Eat in all four quarters of the room".into() },
            Goal::Quick(s) => if zh { format!("{s:.0} 秒内完成主任务") } else { format!("Finish the task within {s:.0} s") },
            Goal::Full(p) => if zh { format!("打烊时至少 {p:.0}% 饱") } else { format!("Close at least {p:.0}% full") },
            Goal::Brave(s) => if zh { format!("红线以上累计 {s:.0} 秒") } else { format!("{s:.0} s above the red line in all") },
            Goal::GoldenAny(n) => if zh { format!("吃到 {n} 盘金盘") } else { format!("Eat {n} golden plates") },
        }
    }
}

/// Whether two goals cannot both be met in one meal.
pub fn conflicts(a: &Goal, b: &Goal) -> bool {
    let one_way = |a: &Goal, b: &Goal| match (a, b) {
        (Goal::Light, Goal::Brave(_) | Goal::Risk { .. }) => true,
        (Goal::Avoid(d), Goal::Collect) => *d < EVERYDAY,
        // Avoiding one dish leaves six kinds to eat.
        (Goal::Avoid(_), Goal::Kinds(n)) => *n as usize >= KINDS,
        _ => false,
    };
    one_way(a, b) || one_way(b, a)
}

/// Draws two side goals of different kinds from `pool`, neither of the main
/// task's kind nor contradicting it or each other. `roll` gives random numbers.
pub fn draw_goals(main: &Goal, pool: &[Goal], mut roll: impl FnMut() -> u32) -> Vec<Goal> {
    let mut left: Vec<Goal> =
        pool.iter().copied().filter(|g| g.kind() != main.kind() && !conflicts(main, g)).collect();
    let mut out = Vec::new();
    while out.len() < 2 && !left.is_empty() {
        let g = left.remove(roll() as usize % left.len());
        left.retain(|o| o.kind() != g.kind() && !conflicts(&g, o));
        out.push(g);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sequence_restarts_its_round_on_a_wrong_colour() {
        let mut t = Tally::default();
        t.step_sequence(0);
        t.step_sequence(2);
        assert_eq!(t.seq, 2);
        t.step_sequence(1);
        assert_eq!(t.seq, 1, "red again starts the round over at one");
        t.step_sequence(3);
        t.step_sequence(4);
        assert_eq!(t.seq, 3);
        t.step_sequence(GOLDEN);
        assert_eq!(t.seq, 3, "gold does not count either way");
        t.step_sequence(4);
        assert_eq!(t.seq, 3);
    }

    #[test]
    fn efficiency_fails_when_the_plates_run_out() {
        let g = Goal::Efficiency { score: 100, plates: 3 };
        let mut t = Tally { plates: 2, score: 50, ..Tally::default() };
        assert_eq!(g.state(&t, false), State::Open);
        t.plates = 3;
        assert_eq!(g.state(&t, false), State::Failed);
        t.score = 100;
        assert_eq!(g.state(&t, false), State::Done);
    }

    #[test]
    fn precision_is_decided_at_closing() {
        let g = Goal::Precision { lo: 90.0, hi: 100.0, plates: 2 };
        let t = Tally { fullness: 95.0, plates: 4, ..Tally::default() };
        assert_eq!(g.state(&t, false), State::Open);
        assert_eq!(g.state(&t, true), State::Done);
        let t = Tally { fullness: 101.0, plates: 4, ..Tally::default() };
        assert_eq!(g.state(&t, true), State::Failed);
    }

    #[test]
    fn no_room_draws_contradictory_goals() {
        for def in crate::rules::LEVELS.iter() {
            let mut n = 7u32;
            for _ in 0..200 {
                let gs = draw_goals(&def.main, def.pool, || {
                    n ^= n << 13;
                    n ^= n >> 17;
                    n ^= n << 5;
                    n
                });
                assert_eq!(gs.len(), 2, "{}", def.id);
                for g in &gs {
                    assert!(!conflicts(&def.main, g), "{}: {g:?} against the task", def.id);
                }
                assert!(!conflicts(&gs[0], &gs[1]), "{}: {gs:?}", def.id);
            }
        }
        assert!(conflicts(&Goal::Avoid(4), &Goal::Collect));
        assert!(conflicts(&Goal::Brave(2.0), &Goal::Light));
        assert!(!conflicts(&Goal::Avoid(4), &Goal::Kinds(5)));
    }

    #[test]
    fn side_goals_differ_in_kind() {
        let mut n = 0u32;
        for _ in 0..50 {
            let gs = draw_goals(&Goal::Score(100), &[Goal::Plates(5), Goal::Kinds(3), Goal::Colors, Goal::Light], || {
                n = n.wrapping_mul(1103515245).wrapping_add(12345);
                n >> 8
            });
            assert_eq!(gs.len(), 2);
            assert_ne!(gs[0].kind(), gs[1].kind());
            assert!(gs.iter().all(|g| g.kind() != Kind::Quantity));
        }
    }
}
