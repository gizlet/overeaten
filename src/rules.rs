//! What the restaurants serve and ask. Data only: `game.rs` applies it.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Zh,
    En,
}

/// A string in both languages.
#[derive(Clone, Copy, Debug)]
pub struct Text {
    pub zh: &'static str,
    pub en: &'static str,
}

impl Text {
    pub const fn new(zh: &'static str, en: &'static str) -> Text {
        Text { zh, en }
    }

    pub fn get(self, lang: Lang) -> &'static str {
        match lang {
            Lang::Zh => self.zh,
            Lang::En => self.en,
        }
    }
}

pub struct Dish {
    pub name: Text,
    /// File stem of its paintings in `assets/art/dishes/*/`.
    pub art: &'static str,
    /// Color of the crumbs it leaves, `0xRRGGBB`.
    pub crumb: u32,
    /// What the dish is worth toward the task.
    pub coins: i32,
    /// Percent of a full stomach.
    pub fullness: i32,
    /// Relative chance of being served; 0 for specials served on a schedule.
    pub weight: u32,
}

pub const KINDS: usize = 7;
/// The six everyday dishes come first; the golden special is last.
pub const EVERYDAY: usize = 6;
pub const GOLDEN: usize = 6;

pub const DISHES: [Dish; KINDS] = [
    Dish {
        name: Text::new("炙烤三文鱼握", "Seared Salmon"),
        art: "salmon",
        crumb: 0xF4845F,
        coins: 18,
        fullness: 22,
        weight: 3,
    },
    Dish {
        name: Text::new("金枪鱼握", "Tuna Nigiri"),
        art: "tuna",
        crumb: 0xC8303F,
        coins: 17,
        fullness: 20,
        weight: 3,
    },
    Dish {
        name: Text::new("鲜虾握", "Shrimp Nigiri"),
        art: "shrimp",
        crumb: 0xFF9A5C,
        coins: 13,
        fullness: 17,
        weight: 3,
    },
    Dish {
        name: Text::new("玉子烧握", "Tamago Nigiri"),
        art: "tamago",
        crumb: 0xF7CF4E,
        coins: 10,
        fullness: 18,
        weight: 3,
    },
    Dish {
        name: Text::new("黄瓜细卷", "Cucumber Maki"),
        art: "maki",
        crumb: 0x6BBF59,
        coins: 6,
        fullness: 8,
        weight: 3,
    },
    Dish {
        name: Text::new("豆皮寿司", "Inari Pocket"),
        art: "inari",
        crumb: 0xD4924F,
        coins: 7,
        fullness: 16,
        weight: 2,
    },
    Dish {
        name: Text::new("金盘大腹", "Golden Toro"),
        art: "tuna",
        crumb: 0xF4C430,
        coins: 30,
        fullness: 18,
        weight: 0,
    },
];

/// The numbers of a meal at the sushi carousel.
pub struct Rules {
    /// Seconds until closing.
    pub duration: f32,
    /// Coins paid at the door; eating that much food completes the task.
    pub admission: i32,
    /// Percent that is "full".
    pub capacity: f32,
    /// Above this the stomach is in danger.
    pub red_line: f32,
    /// Seconds above the red line before the hospital.
    pub hospital_s: f32,
    /// Percent digested per second.
    pub digest: f32,
    /// Different dishes eaten within this many seconds build a combo.
    pub combo_window: f32,
    /// How long one plate takes: the reach, the bite and the chewing.
    pub eat_s: f32,
    /// When in `eat_s` the food is in the mouth and counts.
    pub bite_s: f32,
    /// Belt speed in centimeters per second.
    pub belt_speed: f32,
    /// Seconds between plates from the chef.
    pub serve_every: f32,
    /// The chef waits for this much free belt before serving.
    pub min_gap: f32,
    /// Laps a plate rides before the kitchen takes it back.
    pub laps: f32,
    /// When golden specials are served.
    pub golden_at: [f32; 2],
    /// Walking speed in centimeters per second.
    pub walk: f32,
    /// Distance from the feet within which a plate can be taken, in centimeters.
    pub reach: f32,
}

pub const RULES: Rules = Rules {
    duration: 120.0,
    admission: 160,
    capacity: 100.0,
    red_line: 120.0,
    hospital_s: 5.0,
    digest: 1.0,
    combo_window: 5.0,
    eat_s: 0.85,
    bite_s: 0.32,
    belt_speed: 70.0,
    serve_every: 1.4,
    min_gap: 112.0,
    laps: 1.6,
    golden_at: [32.0, 78.0],
    walk: 380.0,
    reach: 112.0,
};

/// A restaurant on the street. Only the first is open in this version.
pub struct Restaurant {
    pub name: Text,
    /// Its way of eating.
    pub way: Text,
    /// Its task, shown on the ticket.
    pub task: Text,
    pub admission: i32,
    pub open: bool,
    /// Accent color, `0xRRGGBB`.
    pub accent: u32,
}

pub const RESTAURANTS: [Restaurant; 3] = [
    Restaurant {
        name: Text::new("转转寿司", "Carousel Sushi"),
        way: Text::new("从转台上抓盘子吃", "Grab plates off the belt"),
        task: Text::new("吃回本：吃够 160 币", "Eat 160 coins' worth"),
        admission: 160,
        open: true,
        accent: 0xE4572E,
    },
    Restaurant {
        name: Text::new("咕嘟火锅", "Bubbling Pot"),
        way: Text::new("下锅、等熟、及时捞", "Dip, wait, fish it out"),
        task: Text::new("八种食材各尝一口", "Taste all 8 ingredients"),
        admission: 98,
        open: false,
        accent: 0xC0392B,
    },
    Restaurant {
        name: Text::new("点心推车", "Steam Cart"),
        way: Text::new("追着推车满场跑", "Chase the carts"),
        task: Text::new("集齐 6 个点心印章", "Collect 6 dim sum stamps"),
        admission: 88,
        open: false,
        accent: 0x2F8F83,
    },
];

/// The picture on a card's face.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CardArt {
    FirstPlate,
    MoneysWorth,
    Rainbow,
    Golden,
    LightEater,
    Combo,
    Ambulance,
}

pub struct Card {
    /// Stable identifier, written to the save file.
    pub id: &'static str,
    pub name: Text,
    pub condition: Text,
    /// Hidden cards keep their condition secret until found.
    pub hidden: bool,
    pub art: CardArt,
    /// Card band color, `0xRRGGBB`.
    pub band: u32,
}

pub const CARDS: [Card; 7] = [
    Card {
        id: "sushi.first-plate",
        name: Text::new("第一盘", "First Plate"),
        condition: Text::new("从转台上吃下一盘。", "Eat a plate off the belt."),
        hidden: false,
        art: CardArt::FirstPlate,
        band: 0x7FC8A9,
    },
    Card {
        id: "sushi.moneys-worth",
        name: Text::new("吃回本", "Money's Worth"),
        condition: Text::new("打烊前吃够入场费。", "Eat your admission back before closing."),
        hidden: false,
        art: CardArt::MoneysWorth,
        band: 0xF2B134,
    },
    Card {
        id: "sushi.rainbow",
        name: Text::new("七彩转台", "Full Rainbow"),
        condition: Text::new("一餐里尝遍六种寿司。", "Taste all six everyday dishes in one meal."),
        hidden: false,
        art: CardArt::Rainbow,
        band: 0x4A6FA5,
    },
    Card {
        id: "sushi.golden",
        name: Text::new("金盘来了", "Golden Plate"),
        condition: Text::new("抢到金盘特供。", "Catch the golden special."),
        hidden: false,
        art: CardArt::Golden,
        band: 0xE0A526,
    },
    Card {
        id: "sushi.light-eater",
        name: Text::new("小鸟胃", "Light Eater"),
        condition: Text::new("吃回本，且从未超过 100%。", "Eat your money's worth without passing 100%."),
        hidden: false,
        art: CardArt::LightEater,
        band: 0x9BC1BC,
    },
    Card {
        id: "sushi.combo",
        name: Text::new("五连吃", "Five in a Row"),
        condition: Text::new("连吃五盘不同的寿司。", "Chain five different dishes."),
        hidden: false,
        art: CardArt::Combo,
        band: 0xE4572E,
    },
    Card {
        id: "sushi.ambulance",
        name: Text::new("吃到送医", "One Plate Too Many"),
        condition: Text::new("在红区待得太久。", "Stay in the red zone too long."),
        hidden: true,
        art: CardArt::Ambulance,
        band: 0xD7263D,
    },
];

pub fn card_index(art: CardArt) -> usize {
    CARDS.iter().position(|c| c.art == art).unwrap_or(0)
}
