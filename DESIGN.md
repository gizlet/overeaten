# Overeaten · 吃吃吃: design

A reverse cooking game. Nobody cooks: you are the customer. You pay admission at the door, and before the restaurant closes you must finish its eating task, such as eating your money's worth or tasting every dish. Each restaurant eats differently and has its own cards to collect. A stomach gauge keeps greed honest: past its limit a short countdown starts, and if you stay there you go to hospital.

Everything here is original: the restaurants, the visitor 小馋, the chef, the look and the cards. Nothing comes from Overcooked or any other game. Rules and numbers for the sushi bar started from the user's earlier Buffet (GizOS) and Tidewheel Sushi, and were redesigned for this game.

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
| 1 | **Carousel Sushi · 转转寿司** | Walk round a counter island while plates ride a belt. Take the one in reach. Rim color tells the price. A chef serves new plates from the middle, and golden specials come out at set times. | Eat your money's worth: 120 coins. | **Playable** |
| 2 | **Bubbling Pot · 咕嘟火锅** | Drop ingredients into a shared pot, each with its own cooking time, and fish them out in time. Raw food is worth nothing and overcooked food is worth less. | Taste all 8 ingredients. | Planned |
| 3 | **Steam Cart · 点心推车** | Carts roam the hall. Catch one and choose a steamer before it rolls on. | Collect 6 dim sum stamps. | Planned |
| 4 | **Refill Noodles · 续面馆** | A server refills your small bowl as soon as it is empty. Hold A to slurp, and put the lid on to stop the refills. | Eat 30 bowls before closing. | Idea |
| 5 | **Ember Grill · 炭火烤肉** | Lay meat on the grill, flip it once, and eat it at the right doneness. | Eat 10 perfect pieces. | Idea |

### Carousel Sushi in numbers

| Rule | Value |
| --- | --- |
| Meal length | 120 s |
| Admission and task | 120 coins |
| Digestion | 1.1% per second |
| Red line and countdown | 120%, 5 s |
| Belt | 38 px/s; a lap takes about 24 s; plates leave after 1.6 laps |
| Serving | One plate every 1.6 s while 54 px of belt is free behind the chef |
| Golden plate | At 32 s and 78 s: 30 coins, 18% |
| Combo | A different dish within 5 s adds 1 coin per link, up to 4 |

| Dish | Rim | Coins | Fullness |
| --- | --- | --- | --- |
| Seared Salmon · 炙烤三文鱼握 | red | 18 | 22% |
| Tuna Nigiri · 金枪鱼握 | red | 17 | 20% |
| Shrimp Nigiri · 鲜虾握 | green | 13 | 17% |
| Tamago Nigiri · 玉子烧握 | green | 10 | 18% |
| Cucumber Maki · 黄瓜细卷 | white | 6 | 8% |
| Inari Pocket · 豆皮寿司 | white | 7 | 16% |
| Golden Ikura · 金盘鱼子军舰 | gold | 30 | 18% |

Eating every plate in reach does not work: 120 coins at an average of about 0.8 coins per percent is roughly 150% of food, so a player has to pause to digest, choose value-dense plates and catch the golden ones. The autopilot test (`cargo test`) checks that the task is reachable without the hospital. All numbers live in `src/rules.rs`.

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

Everything is drawn on a 640×480 canvas, scaled to fit the screen with bars and never cropped. Text and controls stay inside the 16 px safe margin. Text is at least 16 px; numbers that matter during play are at least 20 px.

| Band (y) | Contents |
| --- | --- |
| 0–48 | The task chip in the middle, over the back wall |
| 0–136 | The back wall: price board, two lanterns, the kitchen pass with its noren, a window on the evening street; then the wainscot |
| 136–436 | The floor. The counter island is centered at (320, 284), 440×160, with the belt round its top and the chef inside. 小馋 walks the floor all round it. Potted plants stand in the corners. |
| 436–474 | The clock (left), the stomach gauge (middle) and the plate count (right), with the combo pill above the plate count |

- **Top middle:** the task, coins eaten against admission, with a bar that fills.
- **Bottom left:** a clock to closing, which turns red in the last 15 s.
- **Bottom middle:** the stomach gauge from 0 to 150%, with a white notch at 100% and a hatched red zone from 120%. When a plate is in reach, a ghost segment shows what it would add, and turns red if it would cross the red line.
- **Bottom right:** plates eaten, with the combo and its draining timer above.
- **Over the plate in reach:** a pulsing ring and a bubble naming the dish and its worth.
- **Danger:** red edges pulse slowly (never above 3 Hz) and a countdown ring floats over the visitor's head.

The other screens: the **title** (the bouncing 吃吃吃 logo, with 小馋 eating at a table among circling plates), the **street** (restaurant tickets under string lights), the **bill** (new cards on the left, a paper receipt with a stamp on the right) and the **album** (a 4 + 3 grid of cards).

## Look and feel

A warm evening palette: tomato red, mustard, mint and cream over honey wood and walnut, with ink-dark panels for the interface. Everything is drawn from shapes, with no image files, so it stays sharp at any window size and runs easily on the R36S's Mali-G31.

- **小馋**, the visitor: a small round eater in an orange hoodie and a bib, with a bun held by a pair of chopsticks. Their belly grows with fullness, their cheeks puff while chewing, their eyes turn to stars for golden plates and to crosses in the red zone.
- **Motion:** plates pop onto the belt and fade back into the kitchen. Food arcs from the plate to the mouth. Crumbs fly, coins float up, the HUD bumps, the screen shakes a little, and cards flip. Lanterns swing, the noren sways and steam rises from the kitchen.
- **Sound:** none yet. Confirm, bite, warning and fanfare effects are planned.

## Platform

Rust with `raylib-sys` 6.0. On a Mac `cargo run` builds raylib for desktop and opens a window. On Gizlet OS the `gizlet-os` feature links Gizlet runtime 1's `libraylib.so.600` (DRM, OpenGL ES 2) and reads the handheld's controls from evdev (BTN_EAST is A, BTN_SOUTH is B), as the gizlet/os launcher does. The app ships as a folder with `gizlet.toml`, which `gizlet install` packs.
