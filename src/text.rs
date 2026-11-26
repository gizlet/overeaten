//! Interface strings in Simplified Chinese and English.

use crate::rules::Text;

pub const TITLE: Text = Text::new("自助大胃王", "Overeaten");
pub const TAGLINE: Text = Text::new("付一次门票，吃到打烊为止", "Pay once at the door, eat until closing");
pub const START: Text = Text::new("开始", "Start");
pub const ALBUM: Text = Text::new("图鉴", "Album");
pub const LANGUAGE: Text = Text::new("English", "中文");
pub const STATS: Text = Text::new("参数", "Stats");
pub const COLLECTED: Text = Text::new("卡片", "Cards");
pub const CREDITS: Text = Text::new(
    "模型：KayKit · Quaternius · Kenney（CC0）  材质：ambientCG（CC0）",
    "Models: KayKit · Quaternius · Kenney (CC0)  Materials: ambientCG (CC0)",
);

pub const PICK_HERO: Text = Text::new("今天谁去吃？", "Who's eating today?");
pub const CHOOSE: Text = Text::new("就你了", "Choose");
pub const BACK: Text = Text::new("返回", "Back");
pub const LOCKED: Text = Text::new("未解锁", "Locked");
pub const RATING: [Text; 4] = [
    Text::new("胃容量", "Stomach"),
    Text::new("消化", "Digestion"),
    Text::new("速度", "Speed"),
    Text::new("抗撑", "Nerve"),
];
pub const NEED_STARS: Text = Text::new("集齐 {} 颗星解锁", "Collect {} stars to unlock");
pub const NEED_HOSPITAL: Text = Text::new("被送医一次后解锁", "Unlocks after a trip to hospital");

pub const ROOM: Text = Text::new("第 {} 间", "Room {}");
pub const MAIN_TASK: Text = Text::new("主任务", "Task");
pub const SIDE_GOALS: Text = Text::new("附加目标", "Side goals");
pub const ADMISSION: Text = Text::new("门票", "Admission");
pub const CLOSING: Text = Text::new("打烊", "Closes in");
pub const BEST: Text = Text::new("最高", "Best");
pub const ENTER: Text = Text::new("付门票进店", "Pay and enter");
pub const ROOM_LOCKED: Text = Text::new("先通过上一间", "Clear the room before");
pub const SOON: Text = Text::new("敬请期待", "Coming soon");
pub const HERO: Text = Text::new("换人", "Hero");
pub const STAR_LINES: Text = Text::new("两星 {} · 三星 {}", "2 stars {} · 3 stars {}");
pub const MENU: Text = Text::new("今日菜单", "Today's menu");
pub const GOLDEN_IN: Text = Text::new("金盘 {}s", "Gold in {}s");
pub const EACH_GOAL: Text = Text::new("每个 +{}", "+{} each");

pub const DIG_IN: Text = Text::new("开吃！", "Dig in!");
pub const COMBO: Text = Text::new("连吃", "Combo");
pub const SCORE: Text = Text::new("得分", "Score");
pub const FULL: Text = Text::new("饱了！", "Full!");
pub const PHEW: Text = Text::new("呼～缓过来了", "Phew, better");
pub const DANGER: Text = Text::new("太撑了！", "Too full!");
pub const DANGER_SUB: Text = Text::new("别再吃了，等它消化", "Stop eating and let it settle");
pub const GOLDEN: Text = Text::new("金盘来了！", "Golden plate!");
pub const GOLDEN_SUB: Text = Text::new("30 分一盘，手快有", "Worth 30: be quick");
pub const TASK_DONE: Text = Text::new("任务完成！", "Task done!");
pub const TASK_DONE_SUB: Text = Text::new("继续吃能加分，或按 Start 结账", "Keep eating for points, or Start to settle up");
pub const GOAL_DONE: Text = Text::new("目标达成 +{}", "Goal +{}");
pub const GOAL_FAILED: Text = Text::new("目标失败", "Goal missed");
pub const COLLAPSE: Text = Text::new("散架了！", "Fell apart!");
pub const LEAN: Text = Text::new("船晃了！", "The deck leans!");
pub const UNKNOWN: Text = Text::new("看不清", "Can't see");
pub const NEXT: Text = Text::new("下一盘", "Next");
pub const PLATES: Text = Text::new("盘", "plates");
pub const SUCCESS: Text = Text::new("吃好了！", "Well fed!");
pub const SUCCESS_SUB: Text = Text::new("任务完成，结账", "Task done: the bill");
pub const CLOSED: Text = Text::new("打烊了", "Closing time");
pub const CLOSED_SUB: Text = Text::new("任务没完成", "The task wasn't done");
pub const BROKE: Text = Text::new("犯规了", "Rule broken");
pub const BROKE_SUB: Text = Text::new("任务失败", "The task failed");
pub const HOSPITAL: Text = Text::new("送医！", "Off to hospital!");
pub const HOSPITAL_SUB: Text = Text::new("吃太多了……", "That was one plate too many");

pub const PAUSED: Text = Text::new("暂停", "Paused");
pub const CONTINUE: Text = Text::new("继续吃", "Keep eating");
pub const SETTLE: Text = Text::new("结账离开", "Settle up and leave");
pub const LEAVE: Text = Text::new("放弃这顿", "Give up this meal");
pub const QUIT_HINT: Text = Text::new("按住 Select + Start 两秒退出游戏", "Hold Select + Start for 2 s to quit");

pub const NEW_CARDS: Text = Text::new("新卡片", "New cards");
pub const NO_NEW_CARDS: Text = Text::new("这顿没有新卡片", "No new cards this time");
pub const BILL: Text = Text::new("账单", "The bill");
pub const FOOD: Text = Text::new("吃下的分值", "Food eaten");
pub const COMBO_BONUS: Text = Text::new("连吃加成", "Combo bonus");
pub const PEAK: Text = Text::new("最撑", "Fullest");
pub const GOALS: Text = Text::new("附加目标", "Side goals");
pub const TOTAL: Text = Text::new("总分", "Score");
pub const NEW_BEST: Text = Text::new("新纪录！", "New best!");
pub const STAMP_WIN: Text = Text::new("吃好了", "WELL FED");
pub const STAMP_CLOSED: Text = Text::new("没完成", "UNFINISHED");
pub const STAMP_BROKE: Text = Text::new("犯规", "RULE BROKEN");
pub const STAMP_HOSPITAL: Text = Text::new("送医", "HOSPITAL");
pub const VOID: Text = Text::new("住院：分数作废", "Hospital: no score");
pub const AGAIN: Text = Text::new("再来一顿", "Eat again");
pub const ROOMS: Text = Text::new("选关", "Rooms");
pub const SAVE_FAILED: Text = Text::new("进度没能保存", "Progress could not be saved");
pub const NEW_HERO: Text = Text::new("新主角：{}", "New hero: {}");

pub const ALBUM_TITLE: Text = Text::new("卡片图鉴", "Card Album");
pub const VIEW: Text = Text::new("查看", "View");
pub const BOOK: Text = Text::new("换册", "Book");
pub const NOT_YET: Text = Text::new("还没收集", "Not collected yet");
pub const HIDDEN_NAME: Text = Text::new("？？？", "???");
pub const HIDDEN_COND: Text = Text::new("隐藏卡片：试试意想不到的吃法。", "A hidden card: eat in an unexpected way.");
pub const BOOK_DONE: Text = Text::new("集满了！+{} 星", "Complete! +{} stars");
pub const BOOK_REWARD: Text = Text::new("集满奖励 {} 星", "Fill it for {} stars");
pub const CLOSE: Text = Text::new("关闭", "Close");
pub const FULLNESS: Text = Text::new("饱腹", "Full");

/// `t` with `{}` replaced by `v`.
pub fn fill(t: &str, v: impl std::fmt::Display) -> String {
    t.replacen("{}", &v.to_string(), 1)
}
