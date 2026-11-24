# Overeaten · 吃吃吃: design

A reverse cooking game. Nobody cooks: you are the customer. You pay admission at the door, and before the restaurant closes you must finish its eating task, such as eating your money's worth or tasting every dish. Each restaurant eats differently and has its own cards to collect. A stomach gauge keeps greed honest: past its limit a short countdown starts, and if you stay there you go to hospital.

Everything here is original: the restaurants, the visitor, the look and the cards. Nothing comes from Overcooked or any other game. The sushi bar's paintings, sounds and starting numbers come from the same author's Tidewheel Sushi (and the GizOS Buffet before it), and were redesigned for this game.

## The core loop

1. **The street.** Pick a restaurant. Its ticket shows its way of eating, its task and its admission.
2. **Pay and enter.** The meal starts, with a clock running to closing time.
3. **Eat.** Every dish has a **value** in coins, which counts toward the task, and a **fullness** in percent. The stomach digests steadily.
   - Up to 100% you are fine.
   - Past 100% you are stuffed: you walk slower and sweat.
   - Past the **red line (120%)** a 5-second countdown starts. Get back under the line and you recover; stay over it and an ambulance takes you to hospital.
4. **End of the meal.** Finish the task and you win the round on the spot. If closing time comes first, the task is failed. If the countdown runs out, you end in hospital.
5. **The bill.** Food value against admission, plates eaten, your fullest moment and what you ate. New cards flip over one by one.
6. **The album.** Every card of every restaurant, kept between sessions.

The tension in every restaurant is the same: dense, valuable food against filling food, and eating now against waiting to digest while the clock runs.

## The restaurants

| | Restaurant | Way of eating | Task | Status |
| --- | --- | --- | --- | --- |
| 1 | **Carousel Sushi · 转转寿司** | Walk round a counter island while plates ride a belt, and take the one in reach; a bubble names it and its worth. Golden specials come out at set times. | Eat your money's worth: 160 coins. | **Playable** |
| 2 | **Bubbling Pot · 咕嘟火锅** | Drop ingredients into a shared pot, each with its own cooking time, and fish them out in time. Raw food is worth nothing and overcooked food is worth less. | Taste all 8 ingredients. | Planned |
| 3 | **Steam Cart · 点心推车** | Carts roam the hall. Catch one and choose a steamer before it rolls on. | Collect 6 dim sum stamps. | Planned |
| 4 | **Refill Noodles · 续面馆** | A server refills your small bowl as soon as it is empty. Hold A to slurp, and put the lid on to stop the refills. | Eat 30 bowls before closing. | Idea |
| 5 | **Ember Grill · 炭火烤肉** | Lay meat on the grill, flip it once, and eat it at the right doneness. | Eat 10 perfect pieces. | Idea |

### Carousel Sushi in numbers

| Rule | Value |
| --- | --- |
| Meal length | 120 s |
| Admission and task | 160 coins |
| Digestion | 1.0% per second |
| Red line and countdown | 120%, 5 s |
| Belt | 70 cm/s on a 15×10 m floor; a lap takes about 26 s; plates leave after 1.6 laps |
| Serving | One plate every 1.4 s while 112 cm of belt is free; new plates settle for 1 s before they can be taken |
| Golden plate | At 32 s and 78 s, on a gold-glazed plate: 30 coins, 18% |
| Combo | A different dish within 5 s adds 1 coin per link, up to 4 |

| Dish | Coins | Fullness |
| --- | --- | --- |
| Seared Salmon · 炙烤三文鱼握 | 18 | 22% |
| Tuna Nigiri · 金枪鱼握 | 17 | 20% |
| Shrimp Nigiri · 鲜虾握 | 13 | 17% |
| Tamago Nigiri · 玉子烧握 | 10 | 18% |
| Cucumber Maki · 黄瓜细卷 | 6 | 8% |
| Inari Pocket · 豆皮寿司 | 7 | 16% |
| Golden Toro · 金盘大腹 (special) | 30 | 18% |

Eating every plate in reach does not work: 160 coins at about one coin per percent is roughly 160% of food, so a player has to pause to digest, choose value-dense plates and catch the golden ones. The autopilot test (`cargo test`) checks that the task is reachable without the hospital. All numbers live in `src/rules.rs`.

## Cards

Each restaurant has its own set of about seven cards, mixing three kinds:

- **Milestones**, earned in an ordinary meal: First Plate, Money's Worth.
- **Skill cards**, for playing well: Full Rainbow (all six everyday dishes in one meal), Golden Plate, Light Eater (finish the task without passing 100%), Five in a Row (a five-dish combo).
- **Hidden cards**, whose conditions stay secret until found: One Plate Too Many (the hospital).

A card is earned once and kept for good. On the bill each new card slides in face down, turns over and settles in the rack. In the album the owned cards show their faces and the missing ones show their backs; choosing a card enlarges it with its name and condition, and hidden cards read "???" until found.

Cards are stored by stable IDs such as `sushi.golden`, one per line in `GIZLET_SAVE_DIR/progress.txt`, written atomically.

## Controls

| Button | Menus | In a meal |
| --- | --- | --- |
| D-pad or left stick | Choose | Walk |
| A | Confirm | Eat the plate in reach |
| B | Back | |
| X | Open the album | |
| Start | | Pause |
| Select | Switch 中文/English | |
| Select + Start, held 2 s | Quit (handled by Gizlet OS) | Quit |

The A and B labels follow `GIZLET_SWAP_AB`. On a Mac: arrows or WASD; Space, Enter or K for A; Backspace or J for B; I for X; Esc or P for Start; Tab for Select.

## Layout at 640×480

The room is painted for a 1600×960 stage in a fixed, high three-quarter view. The game shows its middle 1280 columns at half size on the 640×480 canvas and moves the painted side walls in to frame them, so the R36S shows the whole floor with no bars. The canvas is scaled to fit any other screen, never cropped. Text and controls stay inside the 16 px safe margin, at least 15 px, and numbers that matter during play at least 24 px.

| Where | What |
| --- | --- |
| Top | The back wall: the window, shelves of bowls and the kitchen noren |
| Top right | The clock to closing, which turns red in the last 15 s |
| Middle | The carousel island, its wooden top holding a tea tray and condiments; the booth by the door and the yuzu tree in the corners |
| Over the plate in reach | Pulsing gold rings and a bubble naming the dish and its worth |
| Bottom left | The task: coins eaten against admission, with a bar; the combo and its draining timer above |
| Bottom right | Plates eaten and the stomach gauge from 0 to 150%: a notch at 100%, a marker and a dark red zone from 120%, and a ghost segment for the plate in reach, red when it would cross the line |
| Danger | Red edges pulse slowly (1 Hz) and a countdown ring floats over the visitor |

The other screens: the **title** (the bouncing 吃吃吃 logo over the visitor at a table among circling plates), the **street** (restaurant tickets under string lights), the **bill** (new cards on the left, a paper receipt with a stamp on the right) and the **album** (a 4 + 3 grid of cards).

## Look and feel

Painted, warm and tactile: Tidewheel's honey wood, cream tiles, steel belt and glazed ceramics under late light, with deep teal and cream interface panels in the same palette. Cards, the street and the title screen are drawn from shapes in matching colors until they have paintings of their own.

- **The visitor:** the blue-haired visitor in a cream hoodie, with walking, reaching, biting and savoring frames. They hop as they walk, squash when they bite, grow rounder as they fill up, and flush red in the danger zone.
- **Motion:** plates drop onto the belt and fade back to the kitchen. The bite arcs from the plate to the mouth, golden plates glow, crumbs fly, coins float up, the HUD bumps, the screen shakes a little, and cards turn over. Slats travel round the belt and light breathes across the floor.
- **Sound:** Tidewheel's cues for admission, bites, the red zone, recovery, closing and the cards. Its music loop is not used yet.

## Platform

Rust with `raylib-sys` 6.0. On a Mac `cargo run` builds raylib for desktop and opens a window. On Gizlet OS the `gizlet-os` feature links Gizlet runtime 1's `libraylib.so.600` (DRM, OpenGL ES 2) and reads the handheld's controls from evdev (BTN_EAST is A, BTN_SOUTH is B), as the gizlet/os launcher does. The app ships as a folder with `gizlet.toml`, which `gizlet install` packs.
