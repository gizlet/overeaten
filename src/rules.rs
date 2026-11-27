//! What the restaurant serves and asks, who can eat there and the cards to
//! collect. Data only: `game.rs` and `tasks.rs` apply it.

use crate::tasks::Goal;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Zh,
    En,
}

/// A string in both languages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

// ---- Dishes --------------------------------------------------------------

/// The colour of a dish, for the sequence tasks; each rides its own plate.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hue {
    Red,
    White,
    Yellow,
    Gold,
}

impl Hue {
    /// The plate's glaze, `0xRRGGBB`.
    pub fn plate(self) -> u32 {
        match self {
            Hue::Red => 0xE0524A,
            Hue::White => 0xF4EFE6,
            Hue::Yellow => 0xF2C14E,
            Hue::Gold => 0xFFC93C,
        }
    }
}

pub struct Dish {
    pub name: Text,
    /// The model's name in `assets/models/props.mesh`.
    pub model: &'static str,
    pub hue: Hue,
    /// Points it scores, before the combo.
    pub value: i32,
    /// Percent of an ordinary stomach.
    pub fullness: i32,
    /// Crumb colour, `0xRRGGBB`.
    pub crumb: u32,
}

pub const KINDS: usize = 7;
/// The six everyday dishes come first; the golden special is last.
pub const EVERYDAY: usize = 6;
pub const GOLDEN: usize = 6;

pub const DISHES: [Dish; KINDS] = [
    Dish {
        name: Text::new("三文鱼握", "Salmon Nigiri"),
        model: "salmon_nigiri",
        hue: Hue::Red,
        value: 18,
        fullness: 22,
        crumb: 0xF4845F,
    },
    Dish {
        name: Text::new("金枪鱼握", "Tuna Nigiri"),
        model: "maguro_nigiri",
        hue: Hue::Red,
        value: 17,
        fullness: 20,
        crumb: 0xC8303F,
    },
    Dish {
        name: Text::new("章鱼握", "Octopus Nigiri"),
        model: "octopus_nigiri",
        hue: Hue::White,
        value: 12,
        fullness: 15,
        crumb: 0xE9C6D0,
    },
    Dish {
        name: Text::new("鲜虾握", "Shrimp Nigiri"),
        model: "ebi_nigiri",
        hue: Hue::White,
        value: 13,
        fullness: 17,
        crumb: 0xFF9A5C,
    },
    Dish {
        name: Text::new("玉子烧握", "Tamago Nigiri"),
        model: "tamago_nigiri",
        hue: Hue::Yellow,
        value: 9,
        fullness: 18,
        crumb: 0xF7CF4E,
    },
    Dish {
        name: Text::new("海胆军舰", "Uni Gunkan"),
        model: "uni_roll",
        hue: Hue::Yellow,
        value: 22,
        fullness: 18,
        crumb: 0xF2A93B,
    },
    Dish {
        name: Text::new("金盘刺身", "Golden Sashimi"),
        model: "salmon_sashimi",
        hue: Hue::Gold,
        value: 30,
        fullness: 16,
        crumb: 0xF4C430,
    },
];

/// The numbers every meal shares.
pub struct Rules {
    /// Percent of a full stomach past which you slow down.
    pub capacity: f32,
    /// Past this the hospital countdown runs.
    pub red_line: f32,
    /// Different dishes eaten within this many seconds build a combo.
    pub combo_window: f32,
    /// Each link adds this to the score multiplier, up to `combo_max`.
    pub combo_step: f32,
    pub combo_max: f32,
    /// How long one plate takes: the reach, the bite and the chewing.
    pub eat_s: f32,
    /// When in `eat_s` the food is in the mouth and counts.
    pub bite_s: f32,
    /// Belt speed, cm/s.
    pub belt_speed: f32,
    /// Seconds between plates from the kitchen.
    pub serve_every: f32,
    /// The kitchen waits for this much free belt before serving.
    pub min_gap: f32,
    /// Laps a plate rides before the kitchen takes it back.
    pub laps: f32,
    /// Reach from the feet to a plate, cm.
    pub reach: f32,
    /// Points each finished side goal adds.
    pub bonus: i32,
}

pub const RULES: Rules = Rules {
    capacity: 100.0,
    red_line: 120.0,
    combo_window: 5.0,
    combo_step: 0.25,
    combo_max: 2.0,
    eat_s: 0.85,
    bite_s: 0.32,
    belt_speed: 70.0,
    serve_every: 1.4,
    min_gap: 100.0,
    laps: 1.6,
    reach: 112.0,
    bonus: 40,
};

// ---- Heroes --------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unlock {
    Start,
    Stars(u32),
    /// After one trip to hospital.
    Hospital,
}

pub struct HeroDef {
    pub id: &'static str,
    /// The glTF file under `assets/heroes/`.
    pub model: &'static str,
    pub name: Text,
    pub perk: Text,
    /// How much a 100% dish fills them, as percent of their gauge.
    pub stomach: f32,
    /// Percent digested per second.
    pub digest: f32,
    /// Walking speed, cm/s.
    pub walk: f32,
    /// Seconds over the red line before the hospital.
    pub hospital_s: f32,
    /// Points scored per point of food.
    pub value: f32,
    /// Sees the next dish each kitchen will send out.
    pub foresight: bool,
    /// Does not slide on a leaning deck.
    pub steady: bool,
    /// Falls apart after three quick bites.
    pub fragile: bool,
    pub unlock: Unlock,
    /// Height in metres.
    pub height: f32,
    pub accent: u32,
    /// Ratings 1..5 for the select screen: stomach, digestion, speed, nerve.
    pub ratings: [u8; 4],
}

pub const HEROES: [HeroDef; 6] = [
    HeroDef {
        id: "traveller",
        model: "Rogue_Hooded",
        name: Text::new("旅人", "Traveller"),
        perk: Text::new("什么都不突出，也没有短板。", "Good at nothing in particular, bad at nothing."),
        stomach: 1.0,
        digest: 1.0,
        walk: 380.0,
        hospital_s: 5.0,
        value: 1.0,
        foresight: false,
        steady: false,
        fragile: false,
        unlock: Unlock::Start,
        height: 1.55,
        accent: 0x5FA8A0,
        ratings: [3, 3, 3, 3],
    },
    HeroDef {
        id: "knight",
        model: "Knight",
        name: Text::new("铁胃骑士", "Iron-Gut Knight"),
        perk: Text::new(
            "胃大 25%，红线倒计时 7 秒；走得慢，但在倾斜的甲板上站得稳。",
            "A quarter more stomach and 7 s in the red; slow, but steady on a leaning deck.",
        ),
        stomach: 0.8,
        digest: 0.9,
        walk: 320.0,
        hospital_s: 7.0,
        value: 1.0,
        foresight: false,
        steady: true,
        fragile: false,
        unlock: Unlock::Stars(6),
        height: 1.62,
        accent: 0x6C8EBF,
        ratings: [5, 2, 2, 5],
    },
    HeroDef {
        id: "mage",
        model: "Mage",
        name: Text::new("消化法师", "Digestion Mage"),
        perk: Text::new(
            "胃小，但消化快一半；能看见厨房下一盘出什么。",
            "A small stomach that digests half again as fast; sees the kitchen's next plate.",
        ),
        stomach: 1.18,
        digest: 1.6,
        walk: 380.0,
        hospital_s: 5.0,
        value: 1.0,
        foresight: true,
        steady: false,
        fragile: false,
        unlock: Unlock::Stars(12),
        height: 1.58,
        accent: 0x9B6FC2,
        ratings: [2, 5, 3, 3],
    },
    HeroDef {
        id: "skeleton",
        model: "Skeleton_Minion",
        name: Text::new("骷髅", "Skeleton"),
        perk: Text::new(
            "吃下去的从肋骨漏出来：胃只涨一半，分也只算一半。吃太快会散架。",
            "Food falls through the ribs: half as filling, half the points. Eat too fast and it falls apart.",
        ),
        stomach: 0.5,
        digest: 1.0,
        walk: 400.0,
        hospital_s: 5.0,
        value: 0.5,
        foresight: false,
        steady: false,
        fragile: true,
        unlock: Unlock::Hospital,
        height: 1.55,
        accent: 0xC9C2A7,
        ratings: [5, 3, 4, 1],
    },
    HeroDef {
        id: "office",
        model: "Woman",
        name: Text::new("上班族", "Office Worker"),
        perk: Text::new("午休就一会儿：走得快，但消化慢。", "On a lunch break: walks fast, digests slowly."),
        stomach: 1.0,
        digest: 0.85,
        walk: 450.0,
        hospital_s: 5.0,
        value: 1.0,
        foresight: false,
        steady: false,
        fragile: false,
        unlock: Unlock::Start,
        height: 1.6,
        accent: 0xE07A5F,
        ratings: [3, 2, 5, 3],
    },
    HeroDef {
        id: "student",
        model: "Student",
        name: Text::new("学生", "Student"),
        perk: Text::new("正在长身体：每盘多 10% 分，但更容易撑。", "Still growing: a tenth more points a plate, and fills up faster."),
        stomach: 1.12,
        digest: 1.0,
        walk: 390.0,
        hospital_s: 5.0,
        value: 1.1,
        foresight: false,
        steady: false,
        fragile: false,
        unlock: Unlock::Start,
        height: 1.5,
        accent: 0x4A90C2,
        ratings: [2, 3, 3, 4],
    },
];

// ---- Levels --------------------------------------------------------------

pub struct LevelDef {
    /// Stable identifier for the save file.
    pub id: &'static str,
    pub name: Text,
    /// What makes the room different.
    pub gimmick: Text,
    pub duration: f32,
    pub admission: i32,
    pub main: Goal,
    /// Side goals one meal may draw two of.
    pub pool: &'static [Goal],
    /// Scores for two and three stars.
    pub stars: [i32; 2],
    /// When golden plates come out.
    pub golden: Golden,
}

#[derive(Clone, Copy, Debug)]
pub enum Golden {
    At(&'static [f32]),
    /// From `start`, every `every` seconds.
    Every { start: f32, every: f32 },
}

use Goal as G;

pub const LEVELS: [LevelDef; 8] = [
    LevelDef {
        id: "sushi.1",
        name: Text::new("开张", "Opening Day"),
        gimmick: Text::new("一条椭圆转台，先熟悉一下。", "One oval belt to get the feel of it."),
        duration: 90.0,
        admission: 120,
        main: G::Score(120),
        pool: &[G::Plates(10), G::Kinds(4), G::Combo(3), G::Light, G::GoldenAny(1), G::Zones, G::Quick(60.0)],
        stars: [220, 300],
        golden: Golden::At(&[30.0, 62.0]),
    },
    LevelDef {
        id: "sushi.2",
        name: Text::new("柱廊", "Colonnade"),
        gimmick: Text::new("柱子和凳子挡住走位和视线。", "Pillars and stools block your way and your view."),
        duration: 100.0,
        admission: 130,
        main: G::Collect,
        pool: &[G::Plates(12), G::Combo(4), G::Light, G::GoldenAny(1), G::Avoid(4), G::Full(70.0), G::Quick(70.0)],
        stars: [240, 320],
        golden: Golden::At(&[35.0, 72.0]),
    },
    LevelDef {
        id: "sushi.3",
        name: Text::new("双环", "Twin Loops"),
        gimmick: Text::new("两条转台反向转动，中间只有一座窄桥。", "Two belts turning opposite ways, joined by one narrow bridge."),
        duration: 110.0,
        admission: 140,
        main: G::Sequence(3),
        pool: &[G::Plates(12), G::Kinds(5), G::Colors, G::Light, G::GoldenAny(1), G::Brave(2.0)],
        stars: [260, 340],
        golden: Golden::At(&[40.0, 80.0]),
    },
    LevelDef {
        id: "sushi.4",
        name: Text::new("榻榻米", "Tatami"),
        gimmick: Text::new(
            "抬高的榻榻米平台；上层转台要站上去才够得着。",
            "A raised tatami floor: step up to reach the upper belt.",
        ),
        duration: 110.0,
        admission: 150,
        main: G::Efficiency { score: 150, plates: 10 },
        pool: &[G::Kinds(5), G::Combo(4), G::Colors, G::GoldenAny(1), G::Zones, G::Full(80.0)],
        stars: [280, 360],
        golden: Golden::At(&[45.0, 85.0]),
    },
    LevelDef {
        id: "sushi.5",
        name: Text::new("金盘时刻", "Golden Hour"),
        gimmick: Text::new("推拉门定时开合，挡住一段转台；金盘每 10 秒一盘。", "Sliding doors shut off the belt; a golden plate every 10 s."),
        duration: 100.0,
        admission: 150,
        main: G::Golden { count: 3, within: 4.0 },
        pool: &[G::Plates(12), G::Kinds(5), G::Combo(4), G::Light, G::Avoid(4), G::Brave(2.0)],
        stars: [280, 370],
        golden: Golden::Every { start: 6.0, every: 10.0 },
    },
    LevelDef {
        id: "sushi.6",
        name: Text::new("暗夜", "Lantern Night"),
        gimmick: Text::new(
            "只有灯笼照到的地方看得见；同一种不能连着吃。",
            "Only what the lanterns light can be seen, and no dish twice in a row.",
        ),
        duration: 110.0,
        admission: 140,
        main: G::NoRepeat { score: 140 },
        pool: &[G::Plates(12), G::Kinds(6), G::Colors, G::GoldenAny(1), G::Light, G::Zones],
        stars: [260, 340],
        golden: Golden::At(&[38.0, 76.0]),
    },
    LevelDef {
        id: "sushi.7",
        name: Text::new("渡船", "Ferry"),
        gimmick: Text::new(
            "甲板每隔一会儿就倾斜，人会滑；打烊时要吃得刚刚好。",
            "The deck keeps leaning and you slide; be just full at closing.",
        ),
        duration: 90.0,
        admission: 130,
        main: G::Precision { lo: 90.0, hi: 100.0, plates: 8 },
        pool: &[G::Kinds(5), G::Combo(3), G::GoldenAny(1), G::Avoid(4), G::Zones, G::Plates(10)],
        stars: [210, 270],
        golden: Golden::At(&[28.0, 60.0]),
    },
    LevelDef {
        id: "sushi.8",
        name: Text::new("大满贯", "Grand Slam"),
        gimmick: Text::new(
            "窄桥、柱子、推拉门和昏暗的灯光，全部一起来。",
            "The bridge, the pillars, a sliding door and dim lanterns, all at once.",
        ),
        duration: 150.0,
        admission: 200,
        main: G::Risk { secs: 6.0, score: 200 },
        pool: &[G::Plates(18), G::Kinds(6), G::Colors, G::Combo(5), G::GoldenAny(2), G::Zones],
        stars: [300, 380],
        golden: Golden::At(&[30.0, 70.0, 110.0]),
    },
];

/// A restaurant on the street. Only Carousel Sushi is open in this version.
pub struct Restaurant {
    pub name: Text,
    pub open: bool,
    pub accent: u32,
}

pub const RESTAURANTS: [Restaurant; 2] = [
    Restaurant { name: Text::new("转转寿司", "Carousel Sushi"), open: true, accent: 0xE4572E },
    Restaurant { name: Text::new("下一家", "Next door"), open: false, accent: 0x2F8F83 },
];

// ---- Cards ---------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Book {
    Levels,
    Heroes,
    Dishes,
    Secrets,
}

pub const BOOKS: [Book; 4] = [Book::Levels, Book::Heroes, Book::Dishes, Book::Secrets];

impl Book {
    pub fn name(self) -> Text {
        match self {
            Book::Levels => Text::new("关卡", "Rooms"),
            Book::Heroes => Text::new("主角", "Heroes"),
            Book::Dishes => Text::new("寿司", "Sushi"),
            Book::Secrets => Text::new("隐藏", "Secrets"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Rarity {
    Common,
    Rare,
    Holo,
    Hidden,
}

impl Rarity {
    pub fn name(self) -> Text {
        match self {
            Rarity::Common => Text::new("普通", "Common"),
            Rarity::Rare => Text::new("稀有", "Rare"),
            Rarity::Holo => Text::new("闪卡", "Holo"),
            Rarity::Hidden => Text::new("隐藏", "Hidden"),
        }
    }
}

/// What a card's face shows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Art {
    /// A room's signature piece, with a plate.
    Level(usize),
    Hero(usize),
    Dish(usize),
    Ambulance,
    Exact,
    Combo,
    Empty,
}

pub struct Card {
    /// Stable identifier, written to the save file.
    pub id: &'static str,
    pub name: Text,
    pub condition: Text,
    pub book: Book,
    pub rarity: Rarity,
    pub art: Art,
}

const fn card(id: &'static str, name: Text, condition: Text, book: Book, rarity: Rarity, art: Art) -> Card {
    Card { id, name, condition, book, rarity, art }
}

use Book as B;
use Rarity as R;

pub const CARDS: [Card; 39] = [
    card("sushi.1.clear", Text::new("开张大吉", "Grand Opening"), Text::new("通过第 1 关「开张」。", "Clear room 1, Opening Day."), B::Levels, R::Common, Art::Level(0)),
    card("sushi.1.ace", Text::new("开张满堂彩", "Opening Ace"), Text::new("三星通过第 1 关。", "Three stars in room 1."), B::Levels, R::Holo, Art::Level(0)),
    card("sushi.2.clear", Text::new("绕柱而行", "Round the Pillars"), Text::new("通过第 2 关「柱廊」。", "Clear room 2, Colonnade."), B::Levels, R::Common, Art::Level(1)),
    card("sushi.2.ace", Text::new("柱廊满堂彩", "Colonnade Ace"), Text::new("三星通过第 2 关。", "Three stars in room 2."), B::Levels, R::Holo, Art::Level(1)),
    card("sushi.3.clear", Text::new("过桥", "Over the Bridge"), Text::new("通过第 3 关「双环」。", "Clear room 3, Twin Loops."), B::Levels, R::Common, Art::Level(2)),
    card("sushi.3.ace", Text::new("双环满堂彩", "Twin Loops Ace"), Text::new("三星通过第 3 关。", "Three stars in room 3."), B::Levels, R::Holo, Art::Level(2)),
    card("sushi.4.clear", Text::new("登堂上榻", "Up on the Tatami"), Text::new("通过第 4 关「榻榻米」。", "Clear room 4, Tatami."), B::Levels, R::Common, Art::Level(3)),
    card("sushi.4.ace", Text::new("榻榻米满堂彩", "Tatami Ace"), Text::new("三星通过第 4 关。", "Three stars in room 4."), B::Levels, R::Holo, Art::Level(3)),
    card("sushi.5.clear", Text::new("金光一闪", "Golden Flash"), Text::new("通过第 5 关「金盘时刻」。", "Clear room 5, Golden Hour."), B::Levels, R::Common, Art::Level(4)),
    card("sushi.5.ace", Text::new("金盘满堂彩", "Golden Hour Ace"), Text::new("三星通过第 5 关。", "Three stars in room 5."), B::Levels, R::Holo, Art::Level(4)),
    card("sushi.6.clear", Text::new("提灯夜宴", "Lantern Feast"), Text::new("通过第 6 关「暗夜」。", "Clear room 6, Lantern Night."), B::Levels, R::Common, Art::Level(5)),
    card("sushi.6.ace", Text::new("暗夜满堂彩", "Lantern Night Ace"), Text::new("三星通过第 6 关。", "Three stars in room 6."), B::Levels, R::Holo, Art::Level(5)),
    card("sushi.7.clear", Text::new("稳如船长", "Sea Legs"), Text::new("通过第 7 关「渡船」。", "Clear room 7, Ferry."), B::Levels, R::Common, Art::Level(6)),
    card("sushi.7.ace", Text::new("渡船满堂彩", "Ferry Ace"), Text::new("三星通过第 7 关。", "Three stars in room 7."), B::Levels, R::Holo, Art::Level(6)),
    card("sushi.8.clear", Text::new("大满贯", "Grand Slam"), Text::new("通过第 8 关「大满贯」。", "Clear room 8, Grand Slam."), B::Levels, R::Rare, Art::Level(7)),
    card("sushi.8.ace", Text::new("满贯满堂彩", "Grand Slam Ace"), Text::new("三星通过第 8 关。", "Three stars in room 8."), B::Levels, R::Holo, Art::Level(7)),
    card("hero.traveller.first", Text::new("旅人上路", "On the Road"), Text::new("用旅人通过任意一关。", "Clear any room as the Traveller."), B::Heroes, R::Rare, Art::Hero(0)),
    card("hero.traveller.feat", Text::new("轻装旅人", "Travelling Light"), Text::new("用旅人过关，且从未超过 100%。", "Clear a room as the Traveller without passing 100%."), B::Heroes, R::Holo, Art::Hero(0)),
    card("hero.knight.first", Text::new("铁胃初试", "First Joust"), Text::new("用铁胃骑士通过任意一关。", "Clear any room as the Knight."), B::Heroes, R::Rare, Art::Hero(1)),
    card("hero.knight.feat", Text::new("铁壁之胃", "Fortress Stomach"), Text::new("用骑士过关，且在红线以上累计待满 8 秒。", "Clear a room as the Knight after 8 s in the red."), B::Heroes, R::Holo, Art::Hero(1)),
    card("hero.mage.first", Text::new("法师入席", "The Mage Sits Down"), Text::new("用消化法师通过任意一关。", "Clear any room as the Mage."), B::Heroes, R::Rare, Art::Hero(2)),
    card("hero.mage.feat", Text::new("消化大师", "Master of Digestion"), Text::new("用法师一顿吃满 20 盘。", "Eat 20 plates in one meal as the Mage."), B::Heroes, R::Holo, Art::Hero(2)),
    card("hero.skeleton.first", Text::new("有骨气", "Backbone"), Text::new("用骷髅通过任意一关。", "Clear any room as the Skeleton."), B::Heroes, R::Rare, Art::Hero(3)),
    card("hero.skeleton.feat", Text::new("散而不倒", "Falls Apart, Carries On"), Text::new("用骷髅散架 3 次，仍然过关。", "Fall apart three times as the Skeleton and still clear the room."), B::Heroes, R::Holo, Art::Hero(3)),
    card("hero.office.first", Text::new("午休开饭", "Lunch Break"), Text::new("用上班族通过任意一关。", "Clear any room as the Office Worker."), B::Heroes, R::Rare, Art::Hero(4)),
    card("hero.office.feat", Text::new("准时回工位", "Back on Time"), Text::new("用上班族在 60 秒内完成主任务并过关。", "Clear a room as the Office Worker with the task done within 60 s."), B::Heroes, R::Holo, Art::Hero(4)),
    card("hero.student.first", Text::new("放学开饭", "After School"), Text::new("用学生通过任意一关。", "Clear any room as the Student."), B::Heroes, R::Rare, Art::Hero(5)),
    card("hero.student.feat", Text::new("六连大胃", "Six in a Row"), Text::new("用学生过关，且连吃达到 6 连。", "Clear a room as the Student with a six-dish combo."), B::Heroes, R::Holo, Art::Hero(5)),
    card("dish.salmon", Text::new("三文鱼握", "Salmon Nigiri"), Text::new("吃到一盘三文鱼握。", "Eat a salmon nigiri."), B::Dishes, R::Common, Art::Dish(0)),
    card("dish.tuna", Text::new("金枪鱼握", "Tuna Nigiri"), Text::new("吃到一盘金枪鱼握。", "Eat a tuna nigiri."), B::Dishes, R::Common, Art::Dish(1)),
    card("dish.octopus", Text::new("章鱼握", "Octopus Nigiri"), Text::new("吃到一盘章鱼握。", "Eat an octopus nigiri."), B::Dishes, R::Common, Art::Dish(2)),
    card("dish.shrimp", Text::new("鲜虾握", "Shrimp Nigiri"), Text::new("吃到一盘鲜虾握。", "Eat a shrimp nigiri."), B::Dishes, R::Common, Art::Dish(3)),
    card("dish.tamago", Text::new("玉子烧握", "Tamago Nigiri"), Text::new("吃到一盘玉子烧握。", "Eat a tamago nigiri."), B::Dishes, R::Common, Art::Dish(4)),
    card("dish.uni", Text::new("海胆军舰", "Uni Gunkan"), Text::new("吃到一盘海胆军舰。", "Eat an uni gunkan."), B::Dishes, R::Rare, Art::Dish(5)),
    card("dish.golden", Text::new("金盘刺身", "Golden Sashimi"), Text::new("抢到一盘金盘刺身。", "Catch a golden sashimi."), B::Dishes, R::Holo, Art::Dish(6)),
    card("secret.hospital", Text::new("吃到送医", "One Plate Too Many"), Text::new("在红区待得太久。", "Stay in the red too long."), B::Secrets, R::Hidden, Art::Ambulance),
    card("secret.exact", Text::new("刚刚好", "Just Right"), Text::new("打烊时饱腹正好在 99% 到 101% 之间。", "Close the meal between 99% and 101% full."), B::Secrets, R::Hidden, Art::Exact),
    card("secret.combo8", Text::new("八连吃", "Eight in a Row"), Text::new("连吃达到 8 连。", "Reach an eight-dish combo."), B::Secrets, R::Hidden, Art::Combo),
    card("secret.empty", Text::new("白交门票", "Paid for Nothing"), Text::new("一盘不吃，等到打烊。", "Eat nothing until closing."), B::Secrets, R::Hidden, Art::Empty),
];

/// Stars a finished book adds toward unlocking heroes.
pub const BOOK_BONUS: u32 = 2;

pub fn card_by_id(id: &str) -> Option<usize> {
    CARDS.iter().position(|c| c.id == id)
}
