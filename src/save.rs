//! Progress in `GIZLET_SAVE_DIR/progress.txt` (`./save` when run outside
//! Gizlet OS).
//!
//! The file is plain lines: `card <id>`, `stars <level> <n>`,
//! `best <level> <points>`, `hero <index>`, `hospital <n>`, `meals <n>` and
//! `lang <zh|en>`. Unknown lines (and the single-number `best` of version
//! 0.1) are skipped, so older and newer saves still load.

use crate::rules::{Unlock, BOOKS, BOOK_BONUS, CARDS, HEROES, LEVELS};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const FILE: &str = "progress.txt";

#[derive(Default)]
pub struct Save {
    dir: PathBuf,
    pub cards: BTreeSet<String>,
    pub stars: BTreeMap<String, u32>,
    pub best: BTreeMap<String, i32>,
    pub hero: usize,
    pub hospital: u32,
    pub meals: u32,
    /// `zh` or `en` once the player has chosen.
    pub lang: String,
    /// The last write failed; shown on the result screen.
    pub failed: bool,
}

fn unlock_all() -> bool {
    std::env::var("OVEREATEN_UNLOCK").is_ok_and(|v| v == "1")
}

impl Save {
    pub fn load() -> Save {
        let dir = std::env::var_os("GIZLET_SAVE_DIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("save"));
        let mut s = Save { dir, ..Save::default() };
        if let Ok(text) = std::fs::read_to_string(s.dir.join(FILE)) {
            s.parse(&text);
        }
        s
    }

    fn parse(&mut self, text: &str) {
        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            match parts.as_slice() {
                ["card", id] => {
                    self.cards.insert(id.to_string());
                }
                ["stars", id, n] => {
                    if let Ok(n) = n.parse() {
                        self.stars.insert(id.to_string(), n);
                    }
                }
                ["best", id, n] => {
                    if let Ok(n) = n.parse() {
                        self.best.insert(id.to_string(), n);
                    }
                }
                ["hero", n] => self.hero = n.parse().unwrap_or(0).min(HEROES.len() - 1),
                ["hospital", n] => self.hospital = n.parse().unwrap_or(0),
                ["meals", n] => self.meals = n.parse().unwrap_or(0),
                ["lang", l] => self.lang = l.to_string(),
                _ => {}
            }
        }
    }

    fn render(&self) -> String {
        let mut out = format!("meals {}\nhero {}\nhospital {}\n", self.meals, self.hero, self.hospital);
        if !self.lang.is_empty() {
            out.push_str(&format!("lang {}\n", self.lang));
        }
        for (id, n) in &self.stars {
            out.push_str(&format!("stars {id} {n}\n"));
        }
        for (id, n) in &self.best {
            out.push_str(&format!("best {id} {n}\n"));
        }
        for c in &self.cards {
            out.push_str(&format!("card {c}\n"));
        }
        out
    }

    pub fn has(&self, id: &str) -> bool {
        self.cards.contains(id)
    }

    pub fn stars_of(&self, level: usize) -> u32 {
        self.stars.get(LEVELS[level].id).copied().unwrap_or(0)
    }

    pub fn best_of(&self, level: usize) -> Option<i32> {
        self.best.get(LEVELS[level].id).copied()
    }

    /// Whether every card of a book is collected.
    pub fn book_done(&self, book: crate::rules::Book) -> bool {
        CARDS.iter().filter(|c| c.book == book).all(|c| self.has(c.id))
    }

    /// Stars from the rooms plus the bonus for each finished book.
    pub fn total_stars(&self) -> u32 {
        let rooms: u32 = (0..LEVELS.len()).map(|i| self.stars_of(i)).sum();
        rooms + BOOKS.iter().filter(|&&b| self.book_done(b)).count() as u32 * BOOK_BONUS
    }

    pub fn hero_unlocked(&self, i: usize) -> bool {
        if unlock_all() {
            return true;
        }
        match HEROES[i].unlock {
            Unlock::Start => true,
            Unlock::Stars(n) => self.total_stars() >= n,
            Unlock::Hospital => self.hospital > 0,
        }
    }

    /// A room opens once the one before it is cleared.
    pub fn level_open(&self, i: usize) -> bool {
        i == 0 || self.stars_of(i - 1) > 0 || unlock_all()
    }

    /// Writes atomically: a temporary file renamed over the old one.
    pub fn store(&mut self) {
        let tmp = self.dir.join(format!("{FILE}.tmp"));
        let ok = std::fs::create_dir_all(&self.dir).is_ok()
            && std::fs::write(&tmp, self.render()).is_ok()
            && std::fs::rename(&tmp, self.dir.join(FILE)).is_ok();
        self.failed = !ok;
        if !ok {
            eprintln!("overeaten: could not save to {}", self.dir.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut s = Save { meals: 3, hero: 2, hospital: 1, ..Save::default() };
        s.cards.insert("dish.golden".into());
        s.stars.insert("sushi.1".into(), 3);
        s.best.insert("sushi.1".into(), 251);
        let mut t = Save::default();
        t.parse(&(s.render() + "future line\nbest 131\n"));
        assert_eq!((t.meals, t.hero, t.hospital), (3, 2, 1));
        assert!(t.has("dish.golden"));
        assert_eq!(t.stars_of(0), 3);
        assert_eq!(t.best_of(0), Some(251));
        assert!(t.level_open(1) && !t.level_open(2));
    }

    #[test]
    fn heroes_unlock_with_stars_and_the_hospital() {
        let mut s = Save::default();
        assert!(s.hero_unlocked(0) && !s.hero_unlocked(1) && !s.hero_unlocked(3));
        for i in 0..2 {
            s.stars.insert(LEVELS[i].id.into(), 3);
        }
        assert!(s.hero_unlocked(1) && !s.hero_unlocked(2));
        s.hospital = 1;
        assert!(s.hero_unlocked(3));
    }
}
