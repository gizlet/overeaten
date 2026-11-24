//! Collected cards and a few records, in `GIZLET_SAVE_DIR/progress.txt`
//! (`./save` when run outside Gizlet OS).
//!
//! The file is plain lines, `card <id>`, `meals <n>`, `best <coins>` and
//! `lang <zh|en>`;
//! unknown lines are kept out of the way so newer saves still load.

use std::collections::BTreeSet;
use std::path::PathBuf;

const FILE: &str = "progress.txt";

#[derive(Default)]
pub struct Save {
    dir: PathBuf,
    pub cards: BTreeSet<String>,
    pub meals: u32,
    pub best: i32,
    /// `zh` or `en` once the player has chosen.
    pub lang: String,
    /// The last write failed; shown on the result screen.
    pub failed: bool,
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
            let Some((k, v)) = line.trim().split_once(' ') else { continue };
            match k {
                "card" => {
                    self.cards.insert(v.trim().to_string());
                }
                "meals" => self.meals = v.trim().parse().unwrap_or(self.meals),
                "best" => self.best = v.trim().parse().unwrap_or(self.best),
                "lang" => self.lang = v.trim().to_string(),
                _ => {}
            }
        }
    }

    fn render(&self) -> String {
        let mut out = format!("meals {}\nbest {}\n", self.meals, self.best);
        if !self.lang.is_empty() {
            out.push_str(&format!("lang {}\n", self.lang));
        }
        for c in &self.cards {
            out.push_str(&format!("card {c}\n"));
        }
        out
    }

    pub fn has(&self, id: &str) -> bool {
        self.cards.contains(id)
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
        let mut s = Save { meals: 3, best: 131, ..Save::default() };
        s.cards.insert("sushi.golden".into());
        let mut t = Save::default();
        t.parse(&(s.render() + "future line\n"));
        assert_eq!((t.meals, t.best), (3, 131));
        assert!(t.has("sushi.golden"));
    }
}
